#!/usr/bin/env python
"""M3-5: settlement report input and segmented ReqQrySettlementInfo."""
from __future__ import annotations

import hashlib
import os
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from m3_refdata import BROKER, find_core, free_port, wait_port  # noqa: E402
from ctpbuddy.sdk import Admin, Client  # noqa: E402


def start(core: str, td: int, ap: int, data: str) -> subprocess.Popen:
    proc = subprocess.Popen(
        [core, "--td", f"127.0.0.1:{td}", "--admin", f"127.0.0.1:{ap}",
         "--broker-id", BROKER, "--data-dir", data],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
    )
    wait_port(ap)
    return proc


def query(addr: str, investor: str, day: str = "") -> list[dict]:
    cli = Client(addr)
    cli.auth(BROKER, investor)
    cli.login(BROKER, investor)
    rows = cli.qry_settlement_info(day)
    cli.close()
    return rows


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ctpbuddy-m35-") as root:
        td, ap = free_port(), free_port()
        data = os.path.join(root, "data")
        core = find_core()
        proc = start(core, td, ap, data)
        td_addr, admin_addr = f"127.0.0.1:{td}", f"127.0.0.1:{ap}"
        try:
            content = ("结算报告头\n" + "正文-" * 300).encode("gbk")
            report_day, report_account, report_id = "20261002", "m35", 7
            real_sample = os.environ.get("CTPBUDDY_SETTLEMENT_SAMPLE")
            if real_sample:
                import json
                with open(real_sample, "r", encoding="utf-8") as fh:
                    meta = json.load(fh).get("meta_info", {})
                txt_path = os.path.splitext(real_sample)[0] + ".txt"
                with open(txt_path, "rb") as fh:
                    content = fh.read()
                report_day = str(meta["结算日期"])
                report_account = str(meta["资金账号"])
                report_id = int(os.environ.get("CTPBUDDY_SETTLEMENT_ID", "1"))
                assert content and hashlib.sha256(content).hexdigest()
                print("[real sample] structure metadata + raw-text hash only: day=%s bytes=%d sha256=%s" % (report_day, len(content), hashlib.sha256(content).hexdigest()))
            old_content = b"older\x80\xff\n"
            duplicate_day = "20261004"
            with Admin(admin_addr) as admin:
                admin.settlement_report([
                    {"broker": BROKER, "investor": "m35", "trading_day": report_day, "settlement_id": report_id, "account_id": report_account, "currency_id": "CNY", "content": content},
                    {"broker": BROKER, "investor": "m35", "trading_day": duplicate_day, "settlement_id": 7, "account_id": "m35", "currency_id": "CNY", "content": b"duplicate-key"},
                    {"broker": BROKER, "investor": "m35", "trading_day": "20261003", "settlement_id": 8, "account_id": "m35", "currency_id": "CNY", "content": old_content},
                ])
                try:
                    admin.settlement_report([{"broker": BROKER, "investor": "m35", "trading_day": duplicate_day, "settlement_id": 9, "content": b"overwrite"}])
                except RuntimeError as exc:
                    assert "已存在" in str(exc)
                else:
                    raise AssertionError("重复供给必须拒绝覆盖")
                for bad in (b"a\x00b", bytes([0])):
                    try:
                        admin.settlement_report([{"broker": BROKER, "investor": "bad", "trading_day": "20261001", "settlement_id": 1, "content": bad}])
                    except RuntimeError as exc:
                        assert "字节" in str(exc) or "NUL" in str(exc)
                    else:
                        raise AssertionError("非法 bytes 必须拒绝")

            rows = query(td_addr, "m35", report_day)
            assert len(rows) > 1, len(rows)
            assert [r["SequenceNo"] for r in rows] == list(range(1, len(rows) + 1))
            assert all(isinstance(r["Content"], bytes) for r in rows)
            assert b"".join(r["Content"] for r in rows) == content
            assert hashlib.sha256(b"".join(r["Content"] for r in rows)).digest() == hashlib.sha256(content).digest()
            assert all(r["TradingDay"] == report_day and r["SettlementID"] == report_id and r["BrokerID"] == BROKER and r["InvestorID"] == "m35" for r in rows)
            assert query(td_addr, "m35", "20990101") == []
            assert query(td_addr, "empty") == []
            assert b"".join(r["Content"] for r in query(td_addr, "m35")) == old_content
            with Admin(admin_addr) as admin:
                admin.shutdown()
            proc.wait(timeout=5)
            assert proc.returncode == 0, proc.returncode

            # Restart the same core with the same data directory, then query
            # again after a fresh login. The bytes/hash must survive the restart.
            td, ap = free_port(), free_port()
            proc = start(core, td, ap, data)
            td_addr, admin_addr = f"127.0.0.1:{td}", f"127.0.0.1:{ap}"
            assert b"".join(r["Content"] for r in query(td_addr, "m35", report_day)) == content
            assert b"".join(r["Content"] for r in query(td_addr, "m35")) == old_content
            with Admin(admin_addr) as admin:
                admin.shutdown()
            proc.wait(timeout=5)
            assert proc.returncode == 0, proc.returncode
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.wait(timeout=5)
    print("M3-5 SETTLEMENT INFO E2E: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
