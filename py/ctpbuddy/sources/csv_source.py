"""Market-data source plugin: turn a scenario directory into canonical ticks.

The Rust core owns the canonical `CsvSource` format (40 columns, see
core/ctpbuddy-market/src/lib.rs `CSV_COLUMNS`). This module is the Python-side
authoring/validation helper: it validates scenario dirs and normalizes
exchange-native dumps into the canonical CSV layout. Custom sources (CTP
md forwarder, parquet, database replay...) implement the same
`iter_ticks(dir)` contract and drop in without touching the core.
"""
from __future__ import annotations

import csv
import os
from typing import Any, Dict, Iterator, List

CANONICAL_COLUMNS = [
    "instrument", "exchange", "trading_day", "update_time", "update_millisec", "last_price",
    "volume", "turnover", "open_interest", "pre_settlement", "settlement", "pre_close", "open",
    "high", "low", "close", "upper", "lower", "pre_open_interest", "average", "bid1", "bid2",
    "bid3", "bid4", "bid5", "ask1", "ask2", "ask3", "ask4", "ask5", "bidvol1", "bidvol2",
    "bidvol3", "bidvol4", "bidvol5", "askvol1", "askvol2", "askvol3", "askvol4", "askvol5",
]

INSTRUMENT_COLUMNS = [
    "instrument", "exchange", "product", "volume_multiple", "price_tick",
    "margin_rate", "commission_rate", "open_fee_by_value", "min_volume", "max_volume",
]


def scenario_files(scenario_dir: str) -> Dict[str, str]:
    """Return {kind: path} for the files a scenario must/may contain."""
    return {
        "ticks": os.path.join(scenario_dir, "ticks.csv"),
        "instruments": os.path.join(scenario_dir, "instruments.csv"),
    }


def validate_scenario(scenario_dir: str) -> List[str]:
    """Return a list of problems; empty means loadable by the core."""
    problems: List[str] = []
    files = scenario_files(scenario_dir)
    if not os.path.isdir(scenario_dir):
        return ["scenario dir does not exist: %s" % scenario_dir]
    ticks = files["ticks"]
    if not os.path.exists(ticks):
        problems.append("ticks.csv is required")
        return problems
    with open(ticks, "r", encoding="utf-8", newline="") as f:
        for lineno, row in enumerate(csv.reader(f), start=1):
            if not row or (lineno == 1 and row[0] == "instrument") or row[0].startswith("#"):
                continue
            if len(row) < len(CANONICAL_COLUMNS):
                problems.append("ticks.csv line %d: %d columns, want %d" % (lineno, len(row), len(CANONICAL_COLUMNS)))
                if len(problems) > 5:
                    break
                continue
            try:
                float(row[5])
                int(row[6])
            except ValueError:
                problems.append("ticks.csv line %d: non-numeric last_price/volume" % lineno)
                if len(problems) > 5:
                    break
    instruments = files["instruments"]
    if os.path.exists(instruments):
        with open(instruments, "r", encoding="utf-8", newline="") as f:
            for lineno, row in enumerate(csv.reader(f), start=1):
                if not row or (lineno == 1 and row[0] == "instrument") or row[0].startswith("#"):
                    continue
                if len(row) < len(INSTRUMENT_COLUMNS):
                    problems.append("instruments.csv line %d: %d columns, want %d" % (lineno, len(row), len(INSTRUMENT_COLUMNS)))
                    if len(problems) > 5:
                        break
    return problems


def iter_ticks(scenario_dir: str) -> Iterator[Dict[str, Any]]:
    """Canonical tick stream for a scenario (what the core replays)."""
    with open(scenario_files(scenario_dir)["ticks"], "r", encoding="utf-8", newline="") as f:
        for lineno, row in enumerate(csv.reader(f), start=1):
            if not row or (lineno == 1 and row[0] == "instrument") or row[0].startswith("#"):
                continue
            row = row + [""] * (len(CANONICAL_COLUMNS) - len(row))
            yield dict(zip(CANONICAL_COLUMNS, (c.strip() for c in row)))


def write_canonical(scenario_dir: str, ticks: List[Dict[str, Any]]) -> None:
    """Write a canonical ticks.csv (authoring helper for tests/examples)."""
    os.makedirs(scenario_dir, exist_ok=True)
    with open(scenario_files(scenario_dir)["ticks"], "w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(CANONICAL_COLUMNS)
        for t in ticks:
            w.writerow([t.get(c, "") for c in CANONICAL_COLUMNS])
