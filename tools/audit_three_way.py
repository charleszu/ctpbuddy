# -*- coding: utf-8 -*-
"""真实 CTP order/trade/settlement 查询期望审计。

本模块只构建可解释的查询期望，不把 order/trade CSV 当作完整历史回报，
也不声称已经重演 Core 账本。真实查询回报若未单独提供，结果为 not_evaluated。
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import os
import sys
from collections import Counter, defaultdict
from pathlib import Path

DEFAULT_EXPORT_DIR = os.environ.get("CTPBUDDY_EXPORT_DIR", r"C:\workspace\src\CTP\ctp_export")
DEFAULT_SETTLEMENT_DIR = os.environ.get("CTPBUDDY_SETTLEMENT_DIR", r"C:\workspace\src\CTP\ctp_settlement")
EPS = 1e-9


class BadData(ValueError):
    pass


def trim(value):
    return "" if value is None else str(value).strip()


def anon(value):
    return hashlib.sha256(trim(value).encode("utf-8")).hexdigest()[:12] if trim(value) else ""


def read_csv(path):
    raw = Path(path).read_bytes()
    for encoding in ("gbk", "utf-8-sig", "utf-8"):
        try:
            text = raw.decode(encoding)
            break
        except UnicodeDecodeError:
            continue
    else:
        raise BadData("cannot decode CSV")
    reader = csv.DictReader(io.StringIO(text.lstrip("\ufeff")))
    def clean_key(key):
        key = trim(key).lstrip("\ufeff")
        return {"锘緼ccountID": "AccountID", "锘緽rokerID": "BrokerID"}.get(key, key)
    return [{clean_key(k): trim(v) for k, v in row.items()} for row in reader]


def number(row, field, *, required=False):
    value = trim(row.get(field))
    if not value:
        if required:
            raise BadData("missing numeric field " + field)
        return 0.0
    try:
        result = float(value.replace(",", ""))
    except ValueError as exc:
        raise BadData("invalid numeric field " + field) from exc
    if result != result or result in (float("inf"), float("-inf")):
        raise BadData("non-finite numeric field " + field)
    return result


def integer(row, field, *, required=False):
    value = number(row, field, required=required)
    if abs(value - round(value)) > EPS:
        raise BadData("non-integer quantity field " + field)
    return int(round(value))


def file_identity(path, kind):
    stem = Path(path).stem
    suffix = "_" + kind
    if not stem.endswith(suffix):
        raise BadData("unexpected file name")
    base = stem[: -len(suffix)]
    day, investor = base.rsplit("_", 1)
    return day, investor


def settlement_documents(root):
    result = {}
    root = Path(root)
    if not root.is_dir():
        return result
    for path in sorted(root.rglob("*.json")):
        try:
            doc = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, json.JSONDecodeError):
            continue
        meta = doc.get("meta_info") or {}
        day = trim(meta.get("结算日期"))
        investor = trim(meta.get("资金账号") or meta.get("客户号"))
        if day and investor:
            result.setdefault((day, investor), []).append(doc)
    return result


def one_settlement(documents, day, investor):
    docs = documents.get((day, investor), [])
    if not docs:
        return None
    if len(docs) > 1:
        raise BadData("duplicate settlement for exact day/account")
    return docs[0]


def prior_settlement(documents, day, investor):
    candidates = sorted(d for d, i in documents if i == investor and d < day)
    if not candidates:
        return None
    prior_day = candidates[-1]
    return prior_day, one_settlement(documents, prior_day, investor)


def row_identity(row, kind):
    return (
        trim(row.get("BrokerID")),
        trim(row.get("InvestorID")),
        trim(row.get("TradingDay")),
        trim(row.get("ExchangeID")),
        trim(row.get("OrderSysID")),
    )


def reference_identity(row):
    return (
        trim(row.get("BrokerID")),
        trim(row.get("InvestorID")),
        trim(row.get("TradingDay")),
        trim(row.get("ExchangeID")),
        trim(row.get("OrderRef")),
        trim(row.get("FrontID")),
        trim(row.get("SessionID")),
    )


def validate_scope(row, day, investor):
    if trim(row.get("TradingDay")) != day or trim(row.get("InvestorID")) != investor:
        raise BadData("export row day/account mismatch")
    if not trim(row.get("BrokerID")) or not trim(row.get("ExchangeID")):
        raise BadData("export row missing broker/exchange")


def expected_order_rows(orders):
    """构建 ReqQryOrder 期望；保留 CombOffsetFlag，不把它改名成成交开平。"""
    result = []
    for row in orders:
        result.append({
            "BrokerID": trim(row.get("BrokerID")),
            "InvestorID": trim(row.get("InvestorID")),
            "TradingDay": trim(row.get("TradingDay")),
            "ExchangeID": trim(row.get("ExchangeID")),
            "InstrumentID": trim(row.get("InstrumentID")),
            "OrderSysID": trim(row.get("OrderSysID")),
            "OrderRef": trim(row.get("OrderRef")),
            "FrontID": trim(row.get("FrontID")),
            "SessionID": trim(row.get("SessionID")),
            "Direction": trim(row.get("Direction")),
            "CombOffsetFlag": trim(row.get("CombOffsetFlag")),
            "VolumeTotalOriginal": integer(row, "VolumeTotalOriginal", required=True),
            "VolumeTraded": integer(row, "VolumeTraded"),
        })
    return result


def expected_trade_rows(trades):
    """构建 ReqQryTrade 期望；成交只使用 OffsetFlag。"""
    result = []
    for row in trades:
        result.append({
            "BrokerID": trim(row.get("BrokerID")),
            "InvestorID": trim(row.get("InvestorID")),
            "TradingDay": trim(row.get("TradingDay")),
            "TradeDate": trim(row.get("TradeDate")),
            "ExchangeID": trim(row.get("ExchangeID")),
            "InstrumentID": trim(row.get("InstrumentID")),
            "OrderSysID": trim(row.get("OrderSysID")),
            "OrderRef": trim(row.get("OrderRef")),
            "TradeID": trim(row.get("TradeID")),
            "Direction": trim(row.get("Direction")),
            "OffsetFlag": trim(row.get("OffsetFlag")),
            "HedgeFlag": trim(row.get("HedgeFlag")),
            "InvestUnitID": trim(row.get("InvestUnitID")),
            "Volume": integer(row, "Volume", required=True),
        })
    return result


def match_trades_orders(trades, orders):
    """唯一关联；sys/ref 键多候选均 ambiguity，禁止 first-candidate。"""
    by_sys = defaultdict(list)
    by_ref = defaultdict(list)
    for order in orders:
        by_sys[row_identity(order, "order")].append(order)
        by_ref[reference_identity(order)].append(order)
    matched = 0
    reasons = Counter()
    for trade in trades:
        sys_key = row_identity(trade, "trade")
        candidates = by_sys.get(sys_key, []) if sys_key[-1] else []
        if not candidates and sys_key[-1] == "":
            ref_key = reference_identity(trade)
            candidates = by_ref.get(ref_key, []) if ref_key[-2] else []
        if len(candidates) == 0:
            reasons["trade_without_unique_order"] += 1
            continue
        if len(candidates) > 1:
            reasons["ambiguous_order_match"] += 1
            continue
        order = candidates[0]
        for field in ("InstrumentID", "ExchangeID", "Direction", "InvestorID", "TradingDay"):
            if trim(trade.get(field)) != trim(order.get(field)):
                reasons["order_trade_field_mismatch"] += 1
                break
        else:
            matched += 1
    return matched, reasons


def is_option(row):
    instrument = trim(row.get("InstrumentID") or row.get("合约"))
    product = trim(row.get("品种"))
    return any(x in instrument.upper() for x in ("-C-", "-P-")) or "期权" in product


EXCHANGE_ALIASES = {
    "上期所": "SHFE", "上海期货交易所": "SHFE", "能源中心": "INE",
    "上海国际能源交易中心": "INE", "大商所": "DCE", "郑商所": "CZCE",
    "中金所": "CFFEX", "广期所": "GFEX",
}
HEDGE_ALIASES = {"1": "投机", "投机": "投机", "2": "套利", "套利": "套利", "3": "套保", "套保": "套保", "一般": "投机"}


def canonical_exchange(value):
    value = trim(value)
    return EXCHANGE_ALIASES.get(value, value)


def canonical_hedge(value):
    value = trim(value)
    return HEDGE_ALIASES.get(value, value)


def canonical_invest_unit(row):
    value = trim(row.get("投资单元") or row.get("InvestUnitID"))
    account = trim(row.get("资金账号") or row.get("AccountID"))
    # Broker settlement exports repeat the account in 投资单元 for the default
    # unit, while CTP query fields correctly leave InvestUnitID empty.
    return "" if value and account and value == account else value


def position_key(row):
    return (
        canonical_exchange(row.get("交易所") or row.get("ExchangeID")),
        trim(row.get("合约") or row.get("InstrumentID")),
        canonical_invest_unit(row),
        canonical_hedge(row.get("投保") or row.get("HedgeFlag") or row.get("CombHedgeFlag")),
    )


def position_side(row):
    value = trim(row.get("买卖") or row.get("Direction"))
    if value in ("买", "0", "long", "LONG"):
        return "long"
    if value in ("卖", "1", "short", "SHORT"):
        return "short"
    raise BadData("unknown position direction")


def settlement_positions(doc, source):
    """从结算单同时读取 detail/summary，负数与 detail-summary 不一致均为坏数据。"""
    details = doc.get("positions_detail") or []
    summaries = doc.get("positions_summary") or []
    detail_totals = defaultdict(float)
    for row in details:
        qty = number(row, "持仓量", required=True)
        if qty < -EPS:
            raise BadData(source + " negative detail position")
        detail_totals[(position_key(row), position_side(row))] += qty
    summary_totals = defaultdict(float)
    detail_exchanges = defaultdict(set)
    for key, _side in detail_totals:
        detail_exchanges[key[1:]].add(key[0])
    for row in summaries:
        key = position_key(row)
        if not key[0]:
            exchanges = detail_exchanges.get(key[1:], set())
            if len(exchanges) == 1:
                key = (next(iter(exchanges)), *key[1:])
        for side, field in (("long", "买持"), ("short", "卖持")):
            qty = number(row, field)
            if qty < -EPS:
                raise BadData(source + " negative summary position")
            summary_totals[(key, side)] += qty
    if details and summaries:
        keys = set(detail_totals) | set(summary_totals)
        for key in keys:
            if abs(detail_totals[key] - summary_totals[key]) > EPS:
                raise BadData(source + " detail/summary total mismatch")
    return detail_totals or summary_totals


def direction(row):
    value = trim(row.get("Direction"))
    if value in ("0", "买"):
        return "long"
    if value in ("1", "卖"):
        return "short"
    raise BadData("unknown trade direction")


def offset(row, field):
    value = trim(row.get(field))
    if value in ("0", "开"):
        return "open"
    if value in ("1", "平"):
        return "close"
    if value in ("3", "平今"):
        return "close_today"
    if value in ("4", "平昨"):
        return "close_yesterday"
    raise BadData("unknown offset flag")


def expected_positions(prior, trades, day):
    """由前一结算持仓 + 当日成交得到可靠数量；年龄不充分时标记 ambiguity。"""
    positions = defaultdict(float)
    if prior is not None:
        positions.update(settlement_positions(prior, "prior settlement"))
    ambiguities = Counter()
    for trade in trades:
        qty = integer(trade, "Volume", required=True)
        if qty < 0:
            raise BadData("negative trade volume")
        if not trim(trade.get("InstrumentID")) or not trim(trade.get("ExchangeID")):
            raise BadData("trade missing instrument/exchange")
        key = (
            canonical_exchange(trade.get("ExchangeID")), trim(trade.get("InstrumentID")),
            trim(trade.get("InvestUnitID")), canonical_hedge(trade.get("HedgeFlag")),
        )
        side = direction(trade)
        off = offset(trade, "OffsetFlag")
        if off == "open":
            positions[(key, side)] += qty
        else:
            close_side = "short" if side == "long" else "long"
            if off == "close_today":
                positions[(key, close_side)] -= qty
            elif off == "close_yesterday":
                positions[(key, close_side)] -= qty
            else:
                positions[(key, close_side)] -= qty
                if trim(trade.get("ExchangeID")) not in ("SHFE", "INE"):
                    ambiguities["non_shfe_ine_close_age"] += 1
                else:
                    ambiguities["shfe_ine_close_age_not_inferred"] += 1
    for key, qty in positions.items():
        if qty < -EPS:
            raise BadData("position quantity became negative")
        if abs(qty) <= EPS:
            positions[key] = 0.0
    return positions, ambiguities


def compare_position_query(actual, expected):
    """仅比较数量、方向、合约、hedge、investunit；实际查询缺失不算通过。"""
    actual_map = defaultdict(float)
    for row in actual:
        key = position_key(row)
        for side, field in (("long", "Position"), ("short", "Position")):
            qty = number(row, field)
            if qty < -EPS:
                raise BadData("negative actual query position")
            if side == "long" and trim(row.get("PosiDirection")) in ("1", "卖", "short"):
                side = "short"
            actual_map[(key, side)] += qty
    diffs = []
    for key in set(actual_map) | set(expected):
        if abs(actual_map[key] - expected[key]) > EPS:
            diffs.append({"key": [*key[0], key[1]], "expected": expected[key], "actual": actual_map[key]})
    return diffs


def source_rows_for_day(export_dir, day, investor):
    files = sorted(Path(export_dir).glob(f"{day}_{investor}_*.csv"))
    paths = {p.stem.rsplit("_", 1)[-1]: p for p in files}
    required = ("order", "trade")
    if any(kind not in paths for kind in required):
        return None
    orders, trades = read_csv(paths["order"]), read_csv(paths["trade"])
    for row in orders + trades:
        validate_scope(row, day, investor)
    return orders, trades


def audit(export_dir, settlement_dir, date_filter="", investor_filter="", limit=0, actual_dir=""):
    summary = Counter()
    diffs = []
    skips = Counter()
    errors = []
    statements = settlement_documents(settlement_dir)
    export = Path(export_dir)
    if not export.is_dir():
        skips["export_directory_missing"] += 1
        return make_report(summary, skips, errors, diffs, actual_dir)
    account_files = sorted(export.glob("*_account.csv"))
    if not account_files:
        skips["account_snapshot_missing"] += 1
        return make_report(summary, skips, errors, diffs, actual_dir)
    seen = 0
    for account_path in account_files:
        try:
            day, investor = file_identity(account_path, "account")
        except BadData as exc:
            errors.append(str(exc)); continue
        if date_filter and day != date_filter or investor_filter and investor != investor_filter:
            continue
        if limit and seen >= limit:
            break
        seen += 1
        summary["account_days"] += 1
        try:
            source = source_rows_for_day(export, day, investor)
            if source is None:
                skips["missing_order_or_trade_snapshot"] += 1
                continue
            orders, trades = source
            summary["order_rows"] += len(orders); summary["trade_rows"] += len(trades)
            matched, match_reasons = match_trades_orders(trades, orders)
            summary["trade_order_matches"] += matched
            summary.update({"match_" + k: v for k, v in match_reasons.items()})
            if match_reasons:
                diffs.append({"day": day, "investor": anon(investor), "kind": "trade_order", "reasons": dict(match_reasons)})
            current = one_settlement(statements, day, investor)
            previous = prior_settlement(statements, day, investor)
            if current is None:
                skips["missing_same_day_settlement"] += 1
                continue
            if previous is None:
                skips["missing_previous_settlement"] += 1
                continue
            prior_day, prior_doc = previous
            expected, ambiguities = expected_positions(prior_doc, trades, day)
            summary["position_expectation_account_days"] += 1
            summary.update({"ambiguity_" + k: v for k, v in ambiguities.items()})
            current_totals = settlement_positions(current, "same-day settlement")
            for key in set(expected) | set(current_totals):
                if abs(expected[key] - current_totals[key]) > EPS:
                    diffs.append({"day": day, "investor": anon(investor), "kind": "settlement_position", "key": [*key[0], key[1]], "expected": expected[key], "actual": current_totals[key]})
                    summary["settlement_position_differences"] += 1
            actual_path = Path(actual_dir) / f"{day}_{investor}_position.csv" if actual_dir else None
            if actual_path and actual_path.exists():
                actual = read_csv(actual_path)
                pdiffs = compare_position_query(actual, expected)
                summary["actual_position_query_checks"] += len(expected)
                summary["actual_position_query_differences"] += len(pdiffs)
                if pdiffs:
                    diffs.extend({"day": day, "investor": anon(investor), "kind": "actual_position_query", **d} for d in pdiffs)
            else:
                skips["actual_query_not_available"] += 1
        except (BadData, OSError, csv.Error) as exc:
            errors.append({"day": day, "investor": anon(investor), "reason": str(exc)})
    return make_report(summary, skips, errors, diffs, actual_dir)


def make_report(summary, skips, errors, diffs, actual_dir=""):
    status = "fail" if errors else ("not_evaluated" if skips or diffs else "pass")
    return {
        "status": status,
        "scope": "source_alignment_and_query_expectations_only; core_ledger_replay_not_completed",
        "actual_query_status": "available" if actual_dir else "not_evaluated",
        "checks": dict(summary),
        "differences": diffs[:100],
        "difference_count": len(diffs),
        "skips": dict(skips),
        "errors": errors[:100],
        "error_count": len(errors),
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description="审计真实 order/trade/settlement 查询期望，不保存真实正文")
    parser.add_argument("--export-dir", default=DEFAULT_EXPORT_DIR)
    parser.add_argument("--settlement-dir", default=DEFAULT_SETTLEMENT_DIR)
    parser.add_argument("--actual-query-dir", default="", help="可选的真实查询快照目录；没有则 not_evaluated")
    parser.add_argument("--date", default="")
    parser.add_argument("--investor", default="")
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--report", default="", help="写入新的 JSON 汇总文件")
    args = parser.parse_args(argv)
    report = audit(args.export_dir, args.settlement_dir, args.date, args.investor, args.limit, args.actual_query_dir)
    if args.report:
        report_path = Path(os.path.expandvars(args.report))
        report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 1 if report["status"] == "fail" else 0


if __name__ == "__main__":
    sys.exit(main())
