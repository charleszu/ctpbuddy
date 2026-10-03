#!/usr/bin/env python
"""M3-5: settlement report input and segmented ReqQrySettlementInfo."""
from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from m3_refdata import BROKER, find_core, free_port, wait_port  # noqa: E402
from ctpbuddy.sdk import Admin, Client  # noqa: E402


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ctpbuddy-m35-") as root:
        td, ap = free_port(), free_port()
        proc = subprocess.Popen([find_core(), "--td", f"127.0.0.1:{td}", "--admin", f"127.0.0.1:{ap}", "--broker-id", BROKER, "--data-dir", os.path.join(root, "data")], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        try:
            wait_port(ap)
            with Admin(f"127.0.0.1:{ap}") as admin:
                content = ("结算报告头\n" + "正文-" * 300).encode("gbk")
                admin.settlement_report([{"broker": BROKER, "investor": "m35", "trading_day": "20261002", "settlement_id": 7, "account_id": "m35", "currency_id": "CNY", "content": content}])
                cli = Client(f"127.0.0.1:{td}")
                cli.auth(BROKER, "m35")
                cli.login(BROKER, "m35")
                rows = cli.qry_settlement_info("20261002")
                assert len(rows) > 1, len(rows)
                assert [r["SequenceNo"] for r in rows] == list(range(1, len(rows) + 1))
                assert b"".join(r["Content"] for r in rows) == content
                assert all(r["TradingDay"] == "20261002" and r["SettlementID"] == 7 and r["BrokerID"] == BROKER and r["InvestorID"] == "m35" for r in rows)
                assert cli.qry_settlement_info("20990101") == []
                assert cli.qry_settlement_info()
                empty = Client(f"127.0.0.1:{td}")
                empty.auth(BROKER, "empty")
                empty.login(BROKER, "empty")
                assert empty.qry_settlement_info() == []
                empty.close()
                cli.close()
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
