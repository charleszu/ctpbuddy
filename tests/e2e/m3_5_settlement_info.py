#!/usr/bin/env python
"""M3-5: supplied settlement reports and segmented ReqQrySettlementInfo."""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from m3_refdata import BROKER, find_core, free_port, wait_port  # noqa: E402
from ctpbuddy.sdk import Admin, Client  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DEFAULT_REAL_DIR = r"C:/workspace/src/CTP/ctp_settlement"


def start(core: str, td: int, ap: int, data: str) -> subprocess.Popen:
    proc = subprocess.Popen(
        [core, "--td", f"127.0.0.1:{td}", "--admin", f"127.0.0.1:{ap}",
         "--broker-id", BROKER, "--data-dir", data],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
    )
    wait_port(ap)
    return proc


def query(addr: str, investor: str, day: str, account: str) -> list[dict]:
    cli = Client(addr)
    cli.auth(BROKER, investor)
    cli.login(BROKER, investor)
    rows = cli.qry_settlement_info(day, account_id=account)
    cli.close()
    return rows


def shift_day(day: str, delta: int) -> str:
    import datetime
    return (datetime.date.fromisoformat(f"{day[:4]}-{day[4:6]}-{day[6:]}") + datetime.timedelta(days=delta)).strftime("%Y%m%d")


def load_real_sample() -> tuple[str, str, str, bytes] | None:
    """Load only a user-supplied paired JSON/TXT sample; never copy it into repo."""
    sample = os.environ.get("CTPBUDDY_SETTLEMENT_SAMPLE", "")
    root = os.environ.get("CTPBUDDY_SETTLEMENT_DIR", DEFAULT_REAL_DIR)
    if sample:
        json_path = sample
    elif os.path.isdir(root):
        candidates = []
        for base, _, names in os.walk(root):
            for name in names:
                if name.lower().endswith(".json"):
                    candidates.append(os.path.join(base, name))
        json_path = sorted(candidates)[0] if candidates else ""
    else:
        return None
    if not json_path or not os.path.isfile(json_path):
        return None
    txt_path = os.path.splitext(json_path)[0] + ".txt"
    if not os.path.isfile(txt_path):
        return None
    with open(json_path, "r", encoding="utf-8") as fh:
        meta = json.load(fh).get("meta_info", {})
    day = str(meta.get("结算日期") or meta.get("TradingDay") or "")
    investor = str(meta.get("资金账号") or meta.get("客户号") or meta.get("InvestorID") or "")
    if len(day) != 8 or not day.isdigit() or not investor:
        return None
    with open(txt_path, "rb") as fh:
        content = fh.read()
    if not content or b"\x00" in content:
        return None
    return day, investor, investor, content


def make_scenario(path: str, trading_day: str) -> None:
    rows = []
    for update_time, last_price in (("09:00:00", "3500"), ("15:00:00", "3501")):
        row = {c: "" for c in CANONICAL_COLUMNS}
        row.update(
            instrument="rb2601", exchange="SHFE", trading_day=trading_day,
            update_time=update_time, last_price=last_price, volume="100",
            turnover="350000", open_interest="5000", pre_settlement="3500",
            settlement="3500", pre_close="3499", open=last_price,
            high="3502", low="3498", close=last_price, upper="3850",
            lower="3150", pre_open_interest="4980", average=last_price,
            bid1=str(float(last_price) - 1), ask1=str(float(last_price) + 1),
            bidvol1="20", askvol1="20",
        )
        rows.append(row)
    write_canonical(path, rows)


def wait_idx(admin: Admin, n: int) -> None:
    deadline = time.time() + 5
    while time.time() < deadline:
        if admin.status()["playback"]["idx"] >= n:
            return
        time.sleep(0.02)
    raise AssertionError("playback did not reach tick %d" % n)


def main() -> int:
    real = load_real_sample()
    if real is None:
        print("[real sample] SKIP (paired JSON/TXT directory not available)")
        report_day, report_account, report_investor, content = "20261002", "m35", "m35", ("结算报告头\n" + "正文-" * 300).encode("gbk")
        report_id = 7
        real_mode = False
    else:
        report_day, report_account, report_investor, content = real
        report_id = int(os.environ.get("CTPBUDDY_SETTLEMENT_ID", "1"))
        real_mode = True
        print("[real sample] paired JSON/TXT loaded; raw bytes are supplied by user; sha256=%s" % hashlib.sha256(content).hexdigest())

    with tempfile.TemporaryDirectory(prefix="ctpbuddy-m35-") as root:
        scenario = os.path.join(root, "scenario")
        make_scenario(scenario, report_day)
        refdata = os.path.join(REPO, "refdata")
        if os.path.isdir(refdata):
            shutil.copytree(refdata, os.path.join(scenario, "refdata"))
        td, ap = free_port(), free_port()
        data = os.path.join(root, "data")
        core = find_core()
        proc = start(core, td, ap, data)
        td_addr, admin_addr = f"127.0.0.1:{td}", f"127.0.0.1:{ap}"
        try:
            with Admin(admin_addr) as admin:
                admin.start_scenario(scenario, paused=True)
                admin.settlement_report([
                    {"broker": BROKER, "investor": report_investor, "trading_day": report_day,
                     "settlement_id": report_id, "account_id": report_account,
                     "currency_id": "CNY", "content": content, "source": "user_supplied_raw_txt"},
                ])
                duplicate_day = shift_day(report_day, -1)
                old_content = b"older\x80\xff\n"
                admin.settlement_report([
                    {"broker": BROKER, "investor": report_investor, "trading_day": duplicate_day,
                     "settlement_id": 8, "account_id": report_account, "currency_id": "CNY", "content": old_content},
                ])
                try:
                    admin.settlement_report([{"broker": BROKER, "investor": report_investor, "trading_day": report_day, "settlement_id": 9, "content": b"overwrite"}])
                except RuntimeError as exc:
                    assert "已存在" in str(exc)
                else:
                    raise AssertionError("重复供给必须拒绝覆盖")
                for bad in (b"a\x00b", bytes([0])):
                    try:
                        admin.settlement_report([{"broker": BROKER, "investor": "bad", "trading_day": shift_day(report_day, -2), "settlement_id": 1, "content": bad}])
                    except RuntimeError as exc:
                        assert "字节" in str(exc) or "NUL" in str(exc)
                    else:
                        raise AssertionError("非法 bytes 必须拒绝")

                admin.step()
                wait_idx(admin, 1)
                admin.step()
                wait_idx(admin, 2)
                assert admin.status()["playback"]["virtual_time"] == "15:00:00"
                assert admin.status()["playback"]["idx"] == 2

            rows = query(td_addr, report_investor, report_day, report_account)
            assert len(rows) > 1, len(rows)
            joined = b"".join(r["Content"] for r in rows)
            assert [r["SequenceNo"] for r in rows] == list(range(1, len(rows) + 1))
            assert all(isinstance(r["Content"], bytes) for r in rows)
            assert joined == content
            assert hashlib.sha256(joined).digest() == hashlib.sha256(content).digest()
            assert all(r["TradingDay"] == report_day and r["SettlementID"] == report_id and r["BrokerID"] == BROKER and r["InvestorID"] == report_investor and r["AccountID"] == report_account for r in rows)
            assert query(td_addr, report_investor, "20990101", report_account) == []
            assert query(td_addr, "empty", report_day, "empty") == []

        finally:
            if proc.poll() is None:
                proc.kill()
                proc.wait(timeout=5)

        # Run the progression/restart portion with an explicit next day. The
        # report is already supplied for the prior day, so modeled minimal text
        # must not replace it.
        td, ap = free_port(), free_port()
        proc = start(core, td, ap, data)
        td_addr, admin_addr = f"127.0.0.1:{td}", f"127.0.0.1:{ap}"
        try:
            with Admin(admin_addr) as admin:
                admin.start_scenario(scenario, paused=True)
                next_day = shift_day(report_day, 1)
                admin.settle_day({}, next_day)
                assert admin.status()["playback"]["trading_day"] == next_day
            after_settle = query(td_addr, report_investor, report_day, report_account)
            assert b"".join(r["Content"] for r in after_settle) == content
            with Admin(admin_addr) as admin:
                admin.shutdown()
            proc.wait(timeout=5)
            assert proc.returncode == 0, proc.returncode

            td, ap = free_port(), free_port()
            proc = start(core, td, ap, data)
            td_addr, admin_addr = f"127.0.0.1:{td}", f"127.0.0.1:{ap}"
            restored = query(td_addr, report_investor, report_day, report_account)
            assert b"".join(r["Content"] for r in restored) == content
            assert hashlib.sha256(b"".join(r["Content"] for r in restored)).digest() == hashlib.sha256(content).digest()
            with Admin(admin_addr) as admin:
                admin.shutdown()
            proc.wait(timeout=5)
            assert proc.returncode == 0, proc.returncode
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.wait(timeout=5)
    print("M3-5 SETTLEMENT INFO E2E: PASS (real=%s; no private account/text printed)" % real_mode)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
