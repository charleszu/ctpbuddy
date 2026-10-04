# -*- coding: utf-8 -*-
"""Find real order/trade days that are safe candidates for CTPBuddy replay.

This tool is deliberately a gate, not a fake replay: a candidate is emitted
only when every traded instrument is present in the supplied RefData, every
trade type is in the modeled ordinary-futures set, and no option-looking
contract is present. It does not claim account-level equivalence.
"""
from __future__ import annotations

import argparse
import csv
import glob
import json
import os
import re
from collections import Counter

# No export directory / no trade files: not a pass. Exit 3 unless the caller
# opts in with --allow-skip (CI wiring decides, not the script).
EXIT_SKIPPED = 3


def read_rows(path):
    with open(path, encoding="utf-8-sig", newline="") as fh:
        return list(csv.DictReader(fh))


def ref_instruments(ref_dir):
    found = set()
    for path in glob.glob(os.path.join(ref_dir, "*.jsonl")):
        with open(path, encoding="utf-8") as fh:
            for line in fh:
                try:
                    item = json.loads(line)
                except ValueError:
                    continue
                for key in ("InstrumentID", "instrument_id", "Instrument"):
                    value = item.get(key)
                    if value:
                        found.add(str(value).strip())
                        break
    return found


def option_like(instrument):
    value = instrument.strip().upper()
    # CTP exports use several option spellings: MO2506-C-6100,
    # ag2507C8700, and ag2507P8700. A letter C/P followed by a strike
    # distinguishes these from ordinary futures symbols.
    return bool(re.search(r"(?:-C-|[-_]C|C)[0-9]+$", value) or
                re.search(r"(?:-P-|[-_]P|P)[0-9]+$", value))


def inspect(path, known):
    rows = read_rows(path)
    reasons = Counter()
    for row in rows:
        instrument = row.get("InstrumentID", "").strip()
        trade_type = row.get("TradeType", "").strip()
        if option_like(instrument):
            reasons["option_or_option_like"] += 1
        if instrument not in known:
            reasons["instrument_not_in_refdata"] += 1
        if trade_type not in ("", "0"):
            reasons["unsupported_trade_type"] += 1
    return len(rows), reasons


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--export-dir", default=os.environ.get("CTPBUDDY_EXPORT_DIR", r"C:\workspace\src\CTP\ctp_export"))
    ap.add_argument("--refdata-dir", default="refdata")
    ap.add_argument("--limit", type=int, default=20)
    ap.add_argument("--allow-skip", action="store_true",
                    help="无导出目录/无 *_trade.csv 时以 0 退出（默认退出码 %d）" % EXIT_SKIPPED)
    args = ap.parse_args()
    known = ref_instruments(args.refdata_dir)
    files = sorted(glob.glob(os.path.join(args.export_dir, "*_trade.csv")))
    if not files:
        print("SKIPPED: no *_trade.csv under %s (set CTPBUDDY_EXPORT_DIR / --export-dir)" % args.export_dir)
        return 0 if args.allow_skip else EXIT_SKIPPED
    candidates = []
    reasons = Counter()
    for path in files:
        rows, issue = inspect(path, known)
        if issue:
            reasons.update(issue.keys())
        else:
            candidates.append((os.path.basename(path), rows))
    print("real_trade_files", len(files))
    print("refdata_instruments", len(known))
    print("strict_replay_candidates", len(candidates))
    if candidates:
        for name, rows in candidates[: args.limit]:
            print("  candidate", name, "trades", rows)
    else:
        print("[skipped] no strict candidate: no synthetic replay was claimed")
    print("skip_reason_files", dict(reasons))
    print("note: m4_real_replay.py separately verifies one eligible row when broker exports are available")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
