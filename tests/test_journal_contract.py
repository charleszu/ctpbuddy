"""Focused contract artifact checks used by CI and local changes."""
from __future__ import annotations

import json
import os
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "py"))

from ctpbuddy.cli import main as cli_main  # noqa: E402
from ctpbuddy.journal_contract import (  # noqa: E402
    CONTRACT,
    CONTRACT_VERSION,
    validate_event_contract,
)


def main() -> int:
    assert CONTRACT_VERSION == CONTRACT["contract_version"]
    assert "order_insert" in CONTRACT["events"]
    event = {
        "seq": 1,
        "ts_wall": "2026-10-03T09:30:00+08:00",
        "trading_day": "20261003",
        "vt_ms": 1,
        "type": "order_insert",
        "data": {
            "order_ref": "r",
            "instrument": "rb",
            "exchange": "SHFE",
            "direction": 48,
            "offset": 48,
            "price_type": "2",
            "limit_price": 3500,
            "volume": 1,
            "time_condition": "3",
            "volume_condition": "1",
            "min_volume": 1,
            "outcome": {"accepted": True, "fills": [{"trade_id": "t", "price": 3500, "volume": 1}]},
        },
    }
    assert validate_event_contract(event) == []
    with open(os.path.join(REPO, "py", "ctpbuddy", "contracts", "journal_vectors.json"), encoding="utf-8") as handle:
        vectors = json.load(handle)["validation"]
    for case in vectors:
        problems = validate_event_contract(case["event"])
        assert (not problems) is case["event_valid"], (case["name"], problems)
    malformed = dict(event, type="made_up_event")
    assert any("unknown event type" in problem for problem in validate_event_contract(malformed))
    assert any("contract_version" in problem for problem in validate_event_contract(
        dict(event, contract_version=True)))
    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "journal.jsonl")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(json.dumps(malformed) + "\n")
        assert cli_main(["journal", "hash", path]) == 1
    print("journal contract: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
