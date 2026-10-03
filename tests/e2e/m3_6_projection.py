#!/usr/bin/env python
"""M3-6 e2e: JSONL journal -> SQLite projection -> read-only queries."""
from __future__ import annotations

import json
import os
import shutil
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "py"))

from ctpbuddy.cli import main as cli_main  # noqa: E402
from ctpbuddy.store import Projection, rebuild  # noqa: E402


def main() -> int:
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-m36-")
    try:
        journal = os.path.join(tmp, "journal")
        os.mkdir(journal)
        events = [
            {"seq": 1, "ts_wall": "2026-10-03T09:30:00+08:00", "trading_day": "20261003", "vt_ms": 1, "type": "session_login", "broker": "8888", "investor": "m36", "data": {}},
            {"seq": 2, "ts_wall": "2026-10-03T09:30:01+08:00", "trading_day": "20261003", "vt_ms": 2, "type": "order_insert", "broker": "8888", "investor": "m36", "data": {"order_ref": "M36-1", "order_sys_id": "1", "instrument": "rb2601", "exchange": "SHFE", "direction": 0, "offset": 0, "limit_price": 3500, "volume": 1, "outcome": {"accepted": True}}},
            {"seq": 3, "ts_wall": "2026-10-03T09:30:02+08:00", "trading_day": "20261003", "vt_ms": 3, "type": "fill", "broker": "8888", "investor": "m36", "data": {"trade_id": "T1", "order_sys_id": "1", "order_ref": "M36-1", "instrument": "rb2601", "direction": 0, "offset": 0, "price": 3500, "volume": 1}},
            {"seq": 4, "ts_wall": "2026-10-03T09:30:03+08:00", "trading_day": "20261003", "vt_ms": 4, "type": "settings_updated", "data": {"after": {"qry_freq": 2}}},
        ]
        with open(os.path.join(journal, "20261003.jsonl"), "w", encoding="utf-8") as fh:
            for event in events:
                fh.write(json.dumps(event, ensure_ascii=False) + "\n")
        db = os.path.join(tmp, "ctpbuddy.db")
        assert cli_main(["journal", "rebuild", journal, "--db", db]) == 0
        assert cli_main(["journal", "query", "trade_record", "--db", db, "--investor", "m36"]) == 0
        with Projection(db) as projection:
            assert projection.query("account", investor="m36")[0]["last_seq"] == 3
            assert projection.query("trade_record", investor="m36")[0]["trade_id"] == "T1"
            assert projection.query("audit_log")[0]["action"] == "settings_updated"
        with open(db, "wb") as fh:
            fh.write(b"corrupt")
        assert cli_main(["journal", "rebuild", journal, "--db", db]) == 0
        print("M3-6 PROJECTION: PASS")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
