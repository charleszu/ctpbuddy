# -*- coding: utf-8 -*-
"""Source-level expectation audit for CTP order/trade/position queries.

This does not call Core. It checks that real exported order/trade snapshots can
be joined deterministically and that settlement position detail/summary agree.
"""
from __future__ import annotations
import argparse, csv, glob, hashlib, json, os
from collections import Counter, defaultdict
from pathlib import Path


def read_csv(path):
    with open(path, encoding="utf-8-sig", newline="") as f:
        return list(csv.DictReader(f))


def name_key(path, suffix):
    name = Path(path).name
    return name[:-len(suffix)] if name.endswith(suffix) else name


def anon(value):
    return hashlib.sha256(str(value).encode()).hexdigest()[:10]


def settlement_map(root):
    out = {}
    for p in Path(root).rglob("*.json"):
        try:
            d = json.loads(p.read_text(encoding="utf-8")); m = d.get("meta_info") or {}
            day = str(m.get("结算日期") or ""); acc = str(m.get("资金账号") or m.get("客户号") or "")
            if day and acc: out[(day, acc)] = d
        except (OSError, ValueError): pass
    return out


def join_order_trade(orders, trades):
    by_sys = defaultdict(list); by_ref = defaultdict(list)
    for o in orders:
        by_sys[o.get("OrderSysID", "").strip()].append(o)
        by_ref[o.get("OrderRef", "").strip()].append(o)
    matched = 0; bad = Counter()
    for t in trades:
        sysid, ref = t.get("OrderSysID", "").strip(), t.get("OrderRef", "").strip()
        cand = by_sys.get(sysid, []) if sysid else []
        if not cand: cand = by_ref.get(ref, []) if ref else []
        # A fallback is only deterministic if all candidates have same core fields.
        if not cand: bad["trade_without_order"] += 1; continue
        sig = {(x.get("InstrumentID", "").strip(), x.get("ExchangeID", "").strip(), x.get("Direction", "").strip()) for x in cand}
        if len(sig) != 1: bad["ambiguous_order_key"] += 1; continue
        o = cand[0]; fields = ("InstrumentID", "ExchangeID", "Direction")
        if any(t.get(k, "").strip() != o.get(k, "").strip() for k in fields): bad["order_trade_mismatch"] += 1
        else: matched += 1
    return matched, bad


def position_consistency(doc):
    detail = doc.get("positions_detail") or []; summary = doc.get("positions_summary") or []
    d = defaultdict(lambda: {"买": 0, "卖": 0})
    for row in detail:
        c = str(row.get("合约") or ""); side = str(row.get("买卖") or "")
        try: vol = float(row.get("持仓量") or 0)
        except ValueError: return Counter(bad_position_number=1)
        if side in d[c]: d[c][side] += vol
    bad = Counter()
    for row in summary:
        c = str(row.get("合约") or "")
        for side, field in (("买", "买持"), ("卖", "卖持")):
            try: expected = float(row.get(field) or 0)
            except ValueError: bad["bad_summary_number"] += 1; continue
            if abs(expected - d[c][side]) > 1e-8: bad["position_summary_detail_mismatch"] += 1
    return bad


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--export-dir", default=os.environ.get("CTPBUDDY_EXPORT_DIR", r"C:\workspace\src\CTP\ctp_export"))
    ap.add_argument("--settlement-dir", default=os.environ.get("CTPBUDDY_SETTLEMENT_DIR", r"C:\workspace\src\CTP\ctp_settlement"))
    ap.add_argument("--date", default=""); ap.add_argument("--investor", default=""); ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args(); st = Counter(); samples=[]
    smap = settlement_map(args.settlement_dir) if os.path.isdir(args.settlement_dir) else {}
    files = sorted(glob.glob(os.path.join(args.export_dir, "*_account.csv"))) if os.path.isdir(args.export_dir) else []
    for account_path in files:
        base = name_key(account_path, "_account.csv")
        parts = base.rsplit("_", 1)
        if len(parts) != 2: st["bad_filename"] += 1; continue
        day, investor = parts
        if args.date and day != args.date or args.investor and investor != args.investor: continue
        order_path = account_path.replace("_account.csv", "_order.csv"); trade_path = account_path.replace("_account.csv", "_trade.csv")
        if not os.path.exists(order_path) or not os.path.exists(trade_path): st["missing_export_side"] += 1; continue
        orders, trades = read_csv(order_path), read_csv(trade_path)
        matched, bad = join_order_trade(orders, trades); st["order_rows"] += len(orders); st["trade_rows"] += len(trades); st["matched_trade_rows"] += matched; st.update(bad)
        cur = smap.get((day, investor)); prior_days = sorted(d for d,a in smap if a == investor and d < day)
        if cur is None: st["missing_same_day_settlement"] += 1
        else: st.update(position_consistency(cur)); st["same_day_settlement"] += 1
        if not prior_days: st["missing_prior_settlement"] += 1
        else: st["prior_settlement"] += 1
        st["account_days"] += 1
        if len(samples) < 5: samples.append((day, anon(investor), len(orders), len(trades), matched))
        if args.limit and st["account_days"] >= args.limit: break
    print("account_days", st["account_days"])
    print("order_rows", st["order_rows"], "trade_rows", st["trade_rows"], "matched_trade_rows", st["matched_trade_rows"])
    print("position_summary_detail_checked", st["same_day_settlement"], "prior_settlement", st["prior_settlement"])
    print("query_expectation_audit", dict(st))
    print("samples_anonymized", samples)
    print("scope", "source expectations only; actual Core query comparison and full account replay are not claimed")
    return 1 if any(st[k] for k in ("trade_without_order", "ambiguous_order_key", "order_trade_mismatch", "position_summary_detail_mismatch")) else 0

if __name__ == "__main__": raise SystemExit(main())
