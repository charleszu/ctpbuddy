# -*- coding: utf-8 -*-
"""Deterministic source-alignment audit for real CTP exports.

This is deliberately not a simulator replay. It aligns prior settlement,
order/trade/account exports, and same-day settlement statements without
inventing option or unmodeled accounting rules.
"""
from __future__ import annotations
import argparse, csv, glob, hashlib, json, os
from collections import Counter
from pathlib import Path


def rows(path):
    with open(path, encoding="utf-8-sig", newline="") as f:
        return list(csv.DictReader(f))


def statements(root):
    out = {}
    for p in Path(root).rglob("*.json"):
        try:
            d = json.loads(p.read_text(encoding="utf-8"))
            m = d.get("meta_info") or {}
            day, account = str(m.get("结算日期") or ""), str(m.get("资金账号") or m.get("客户号") or "")
            if day and account:
                out[(day, account)] = d
        except (OSError, ValueError):
            pass
    return out


def key(name):
    stem = Path(name).stem
    if stem.endswith("_account"):
        stem = stem[:-len("_account")]
    return stem.rsplit("_", 1)


def join_trade_orders(trades, orders):
    by_sys, by_ref = {}, {}
    for o in orders:
        by_sys.setdefault(o.get("OrderSysID", "").strip(), []).append(o)
        by_ref.setdefault(o.get("OrderRef", "").strip(), []).append(o)
    matched = 0; bad = Counter()
    for t in trades:
        sysid = t.get("OrderSysID", "").strip(); ref = t.get("OrderRef", "").strip()
        candidates = by_sys.get(sysid, []) if sysid else []
        if not candidates: candidates = by_ref.get(ref, []) if ref else []
        if not candidates:
            bad["trade_without_order"] += 1; continue
        o = candidates[0]
        checks = ("InstrumentID", "ExchangeID", "Direction")
        if any(t.get(k, "").strip() != o.get(k, "").strip() for k in checks):
            bad["order_trade_field_mismatch"] += 1; continue
        matched += 1
    return matched, bad


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--export-dir", default=os.environ.get("CTPBUDDY_EXPORT_DIR", r"C:\workspace\src\CTP\ctp_export"))
    ap.add_argument("--settlement-dir", default=os.environ.get("CTPBUDDY_SETTLEMENT_DIR", r"C:\workspace\src\CTP\ctp_settlement"))
    ap.add_argument("--date", default=""); ap.add_argument("--investor", default=""); ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()
    files = sorted(glob.glob(os.path.join(args.export_dir, "*_account.csv"))) if os.path.isdir(args.export_dir) else []
    stmt = statements(args.settlement_dir) if os.path.isdir(args.settlement_dir) else {}
    summary = Counter(); samples=[]
    for apath in files:
        day, investor = key(apath)
        if args.date and day != args.date or args.investor and investor != args.investor: continue
        opath = apath.replace("_account.csv", "_order.csv"); tpath = apath.replace("_account.csv", "_trade.csv")
        if not os.path.exists(opath) or not os.path.exists(tpath): summary["missing_export_side"] += 1; continue
        account = rows(apath)[-1:] ; orders=rows(opath); trades=rows(tpath)
        # prior settlement is a baseline availability check only
        prior = sorted((d,a) for (d,a) in stmt if a == investor and d < day)
        current = stmt.get((day, investor))
        if not prior: summary["missing_prior_settlement"] += 1
        else: summary["prior_settlement_available"] += 1
        if current is None: summary["missing_current_settlement"] += 1
        else: summary["current_settlement_available"] += 1
        matched, bad = join_trade_orders(trades, orders); summary["trade_rows"] += len(trades); summary["matched_trade_rows"] += matched; summary.update(bad)
        if account and account[0].get("TradingDay", day).strip() not in ("", day): summary["account_trading_day_mismatch"] += 1
        summary["account_days"] += 1
        if len(samples) < 5: samples.append((day, investor, len(orders), len(trades), matched))
        if args.limit and summary["account_days"] >= args.limit: break
    print("account_days", summary["account_days"])
    print("trade_rows", summary["trade_rows"], "matched_trade_rows", summary["matched_trade_rows"])
    print("source_alignment", dict(summary))
    print("scope", "baseline and source-field alignment only; no Core ledger replay or option P/L inference")
    for x in samples: print("sample", x)
    return 0

if __name__ == "__main__": raise SystemExit(main())
