# -*- coding: utf-8 -*-
"""Source-level expectation audit for CTP order/trade/position queries.

This does not call Core. It checks that real exported order/trade snapshots can
be joined deterministically and that settlement position detail/summary agree.
"""
from __future__ import annotations
import argparse, csv, glob, hashlib, io, json, os
from collections import Counter, defaultdict
from pathlib import Path


CSV_ENCODINGS = ("utf-8-sig", "gbk")


def read_csv(path):
    # Broker exports come as UTF-8 (BOM) or GBK; try in that order, same as
    # audit_three_way.py, instead of failing on the first non-UTF-8 byte.
    raw = Path(path).read_bytes()
    last = None
    for enc in CSV_ENCODINGS:
        try:
            text = raw.decode(enc)
        except UnicodeDecodeError as e:
            last = e; continue
        return list(csv.DictReader(io.StringIO(text, newline="")))
    raise ValueError("undecodable csv %s: %s" % (path, last))


def name_key(path, suffix):
    name = Path(path).name
    return name[:-len(suffix)] if name.endswith(suffix) else name


def anon(value):
    return hashlib.sha256(str(value).encode()).hexdigest()[:10]


def settlement_map(root):
    """(day, account) -> statement. Duplicates are **not** silently overwritten:
    the first file wins, every later one is listed in `duplicates` so the
    report can show which day/account pairs have more than one statement."""
    out = {}; duplicates = []
    for p in sorted(Path(root).rglob("*.json")):
        try:
            d = json.loads(p.read_text(encoding="utf-8")); m = d.get("meta_info") or {}
            day = str(m.get("结算日期") or ""); acc = str(m.get("资金账号") or m.get("客户号") or "")
            if not (day and acc): continue
            if (day, acc) in out:
                duplicates.append({"day": day, "investor": anon(acc), "file": anon(p.name)}); continue
            out[(day, acc)] = d
        except (OSError, ValueError): pass
    return out, duplicates


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
    ap.add_argument("--report", default="", help="可选：写入匿名 JSON 报告（计数、重复结算单、skip/fail 理由）")
    args = ap.parse_args(); st = Counter(); samples=[]; undecodable=[]
    smap, dup_settlements = settlement_map(args.settlement_dir) if os.path.isdir(args.settlement_dir) else ({}, [])
    st["duplicate_settlement"] = len(dup_settlements)
    files = sorted(glob.glob(os.path.join(args.export_dir, "*_account.csv"))) if os.path.isdir(args.export_dir) else []
    for account_path in files:
        base = name_key(account_path, "_account.csv")
        parts = base.rsplit("_", 1)
        if len(parts) != 2: st["bad_filename"] += 1; continue
        day, investor = parts
        if args.date and day != args.date or args.investor and investor != args.investor: continue
        order_path = account_path.replace("_account.csv", "_order.csv"); trade_path = account_path.replace("_account.csv", "_trade.csv")
        if not os.path.exists(order_path) or not os.path.exists(trade_path): st["missing_export_side"] += 1; continue
        try: orders, trades = read_csv(order_path), read_csv(trade_path)
        except ValueError as e: st["undecodable_csv"] += 1; undecodable.append({"day": day, "investor": anon(investor), "error": str(e)[:80]}); continue
        matched, bad = join_order_trade(orders, trades); st["order_rows"] += len(orders); st["trade_rows"] += len(trades); st["matched_trade_rows"] += matched; st.update(bad)
        cur = smap.get((day, investor)); prior_days = sorted(d for d,a in smap if a == investor and d < day)
        if cur is None: st["missing_same_day_settlement"] += 1
        else: st.update(position_consistency(cur)); st["same_day_settlement"] += 1
        if not prior_days: st["missing_prior_settlement"] += 1
        else: st["prior_settlement"] += 1
        st["account_days"] += 1
        if len(samples) < 5: samples.append((day, anon(investor), len(orders), len(trades), matched))
        if args.limit and st["account_days"] >= args.limit: break
    # Same 口径 as audit_three_way.py: a settlement that is missing (same-day
    # or prior) means the position tables were **not evaluated** for that
    # account day -- it is neither a pass nor a data failure. Bad data fails.
    fail_keys = ("trade_without_order", "ambiguous_order_key", "order_trade_mismatch", "position_summary_detail_mismatch",
                 "bad_position_number", "bad_summary_number", "undecodable_csv", "duplicate_settlement")
    not_evaluated = st["missing_same_day_settlement"] + st["missing_prior_settlement"]
    failed = any(st[k] for k in fail_keys)
    if not files: status = "skipped"
    elif failed: status = "fail"
    elif not_evaluated: status = "not_evaluated"
    else: status = "pass"
    print("account_days", st["account_days"])
    print("order_rows", st["order_rows"], "trade_rows", st["trade_rows"], "matched_trade_rows", st["matched_trade_rows"])
    print("position_summary_detail_checked", st["same_day_settlement"], "prior_settlement", st["prior_settlement"])
    print("position_tables_not_evaluated", not_evaluated, "(missing same-day %d / prior %d settlement)" % (st["missing_same_day_settlement"], st["missing_prior_settlement"]))
    print("duplicate_settlement", st["duplicate_settlement"], dup_settlements[:5])
    print("query_expectation_audit", dict(st))
    print("samples_anonymized", samples)
    print("status", status)
    print("scope", "source expectations only; actual Core query comparison and full account replay are not claimed")
    if args.report:
        report = {"status": status, "counts": dict(st), "position_tables_not_evaluated": not_evaluated,
                  "duplicate_settlements": dup_settlements, "undecodable_csv": undecodable,
                  "samples_anonymized": samples, "fail_keys": list(fail_keys),
                  "scope": "source expectations only; actual Core query comparison and full account replay are not claimed"}
        Path(args.report).write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return 1 if failed else 0

if __name__ == "__main__": raise SystemExit(main())
