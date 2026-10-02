#!/usr/bin/env python
"""One-shot generator for scenarios/dsl_demo/ticks.csv (M2-2 demo stream).

12 rb2601 (SHFE) ticks, 30s apart, 09:30:00 -> 09:35:30. Prices drift +1 per
tick; the five-level book is bid1=last-2/ask1=last+2 with static depth.

The scenario.yaml on top applies: freeze 09:32:00+30s (drops one tick),
liquidity x0.5 from 09:31:00, gap +2 from 09:33:00 — the e2e asserts the
transformed stream through the real wire (see tests/e2e/m2_scenario.py).
"""
from __future__ import annotations

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(HERE)), "py"))

from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402


def main() -> int:
    rows = []
    for i in range(12):
        minute = 30 + i // 2
        t = "09:%02d:%02d" % (minute, 30 * (i % 2))
        last = 3500 + i
        row = {c: "" for c in CANONICAL_COLUMNS}
        row.update(
            instrument="rb2601",
            exchange="SHFE",
            trading_day="20261002",
            update_time=t,
            update_millisec="0",
            last_price=str(last),
            volume=str(100 + 3 * i),
            turnover=str(350000 + 3000 * i),
            open_interest="5200",
            pre_settlement="3500",
            settlement="3500",
            pre_close="3498",
            open="3499",
            high="3508",
            low="3497",
            close="3501",
            upper="3850",
            lower="3150",
            pre_open_interest="5180",
            average=str(last + 2.5),
        )
        for k in range(5):
            row["bid%d" % (k + 1)] = str(last - 2 - k)
            row["ask%d" % (k + 1)] = str(last + 2 + k)
            row["bidvol%d" % (k + 1)] = str(10 - 2 * k)
            row["askvol%d" % (k + 1)] = str(12 - 2 * k)
        rows.append(row)
    write_canonical(HERE, rows)
    print("wrote %d ticks -> %s" % (len(rows), os.path.join(HERE, "ticks.csv")))
    return 0


if __name__ == "__main__":
    sys.exit(main())
