#!/usr/bin/env python
"""M4 受控真实期货子集：真实 order/trade 驱动的四查询验收。

这不是完整账户重演，也不把真实账号、OrderSysID 或 TradeID 写入场景。只选择
一笔可唯一关联、完全成交的真实普通期货开仓成交，在隔离的匿名 Core 账号中用
真实订单字段重发，并验收四个查询的核心字段。
"""
from __future__ import annotations

import csv
import glob
import json
import math
import os
import re
import socket
import subprocess
import sys
import tempfile
import time
from collections import Counter
from pathlib import Path

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "py"))
sys.path.insert(0, os.path.join(REPO, "tools"))

from ctpbuddy.sdk import Admin, Client  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402
from audit_three_way import canonical_hedge, compare_position_query  # noqa: E402

BROKER = os.environ.get("CTPBUDDY_M4_BROKER", "8888")
EXPORT = os.environ.get("CTPBUDDY_EXPORT_DIR", r"C:\workspace\src\CTP\ctp_export")
REFDATA = os.path.join(REPO, "refdata", "instruments.jsonl")
REPORT_DEFAULT = ""


def trim(value):
    return "" if value is None else str(value).strip()


def cvalue(value):
    return chr(value) if isinstance(value, int) and value >= 32 else trim(value)


def cflag(value):
    return chr(value) if isinstance(value, int) else trim(value)


def number(row, field):
    return float(trim(row.get(field)).replace(",", ""))


def integer(row, field):
    value = number(row, field)
    if not math.isfinite(value) or abs(value - round(value)) > 1e-9:
        raise ValueError("invalid integer field %s" % field)
    return int(round(value))


def option_like(value):
    return bool(re.search(r"(?:-C-|[-_]C|C)[0-9]+$", trim(value).upper()) or
                re.search(r"(?:-P-|[-_]P|P)[0-9]+$", trim(value).upper()))


def load_refdata():
    result = {}
    with open(REFDATA, encoding="utf-8") as source:
        for line in source:
            item = json.loads(line)
            result[(trim(item.get("exchange_id")), trim(item.get("instrument_id")))] = item
    return result


def eligible_ref(row, refdata):
    item = refdata.get((trim(row.get("ExchangeID")), trim(row.get("InstrumentID"))))
    if not item:
        return False, "refdata_missing"
    if option_like(row.get("InstrumentID")) or trim(item.get("product_class")) not in ("", "1"):
        return False, "option_or_non_future"
    day = trim(row.get("TradingDay"))
    expiry = trim(item.get("expire_date"))
    if not expiry or (len(day) == 8 and len(expiry) == 8 and expiry < day):
        return False, "expiry_not_covering_trading_day"
    return True, ""


def identity(row):
    return tuple(trim(row.get(k)) for k in ("BrokerID", "InvestorID", "TradingDay", "ExchangeID", "OrderSysID"))


def source_files():
    if not os.path.isdir(EXPORT):
        return []
    return sorted(glob.glob(os.path.join(EXPORT, "*_trade.csv")))


def choose_row():
    reasons = Counter()
    refdata = load_refdata()
    for trade_path in source_files():
        order_path = trade_path.replace("_trade.csv", "_order.csv")
        if not os.path.isfile(order_path):
            reasons["missing_order_snapshot"] += 1
            continue
        with open(order_path, encoding="utf-8-sig", newline="") as f:
            orders = list(csv.DictReader(f))
        with open(trade_path, encoding="utf-8-sig", newline="") as f:
            trades = list(csv.DictReader(f))
        for trade in trades:
            if trim(trade.get("OffsetFlag")) != "0":
                reasons["trade_not_open"] += 1; continue
            if trim(trade.get("HedgeFlag")) not in ("1", "投机"):
                reasons["trade_not_speculation"] += 1; continue
            if trim(trade.get("InvestUnitID")):
                reasons["trade_invest_unit_not_empty"] += 1; continue
            ok, reason = eligible_ref(trade, refdata)
            if not ok:
                reasons[reason] += 1; continue
            candidates = [o for o in orders if identity(o) == identity(trade)]
            if len(candidates) != 1:
                reasons["order_not_unique_by_broker_investor_day_exchange_sys"] += 1; continue
            order = candidates[0]
            if not controlled_open_pair(trade, order):
                reasons["order_trade_open_pair_mismatch"] += 1; continue
            if trim(order.get("CombOffsetFlag")) != "0":
                reasons["order_comb_offset_not_open"] += 1; continue
            if trim(order.get("InvestUnitID")):
                reasons["order_invest_unit_not_empty"] += 1; continue
            if trim(order.get("OrderStatus")) not in ("", "0", "AllTraded", "全部成交"):
                reasons["order_not_fully_filled"] += 1; continue
            if integer(order, "VolumeTotalOriginal") != integer(trade, "Volume"):
                reasons["order_qty_not_equal_trade_qty"] += 1; continue
            same_trades = [t for t in trades if identity(t) == identity(trade)]
            if len(same_trades) != 1:
                reasons["order_has_not_one_trade"] += 1; continue
            fields = ("LimitPrice", "VolumeTotalOriginal", "TimeCondition", "VolumeCondition", "MinVolume")
            if any(trim(order.get(f)) == "" for f in fields):
                reasons["order_request_field_missing"] += 1; continue
            return {"trade": trade, "order": order, "ref": refdata[(trim(trade.get("ExchangeID")), trim(trade.get("InstrumentID")))]}, reasons
    return None, reasons


def free_port():
    sock = socket.socket(); sock.bind(("127.0.0.1", 0)); port = sock.getsockname()[1]; sock.close(); return port


def wait_admin(port):
    deadline = time.time() + 10
    while time.time() < deadline:
        try:
            admin = Admin("127.0.0.1:%d" % port)
        except OSError:
            time.sleep(.05)
            continue
        admin.ping()
        return admin
    raise RuntimeError("admin connection not ready")


def controlled_open_pair(trade, order):
    return (trim(trade.get("OffsetFlag")) == "0" and
            trim(order.get("CombOffsetFlag")) == "0" and
            trim(trade.get("HedgeFlag")) in ("1", "投机") and
            not trim(trade.get("InvestUnitID")) and
            not trim(order.get("InvestUnitID")) and
            trim(trade.get("InstrumentID")) == trim(order.get("InstrumentID")) and
            trim(trade.get("ExchangeID")) == trim(order.get("ExchangeID")) and
            trim(trade.get("Direction")) == trim(order.get("Direction")) and
            integer(trade, "Volume") == integer(order, "VolumeTotalOriginal"))


def wait_idx(admin, expected):
    deadline = time.time() + 5
    while time.time() < deadline:
        if admin.status()["playback"]["idx"] >= expected:
            return
        time.sleep(.02)
    raise RuntimeError("playback index did not reach %d" % expected)


def controlled_quotes(direction, trade_price):
    # 卖开必须让 bid 命中真实成交价；bid=trade_price-.01 会永不成交。
    return ((trade_price, trade_price + .01) if direction == "1" else
            (trade_price - .01, trade_price))


def scenario(path, row):
    trade = row["trade"]
    price = number(trade, "Price")
    direction = trim(trade.get("Direction"))
    bid, ask = controlled_quotes(direction, price)
    cols = {c: "" for c in CANONICAL_COLUMNS}
    cols.update(instrument=trim(trade["InstrumentID"]), exchange=trim(trade["ExchangeID"]),
                trading_day=trim(trade["TradingDay"]), update_time="09:00:01",
                last_price=str(price), volume="100", turnover=str(price), open_interest="1000",
                pre_settlement=str(price), settlement=str(price), pre_close=str(price),
                open=str(price), high=str(price), low=str(price), close=str(price),
                upper=str(price * 1.2), lower=str(price * .8), average=str(price),
                bid1=str(bid), ask1=str(ask), bidvol1="100", askvol1="100")
    write_canonical(path, [cols])


def query_assertions(cli, selected):
    trade, order = selected["trade"], selected["order"]
    instrument, exchange = trim(trade["InstrumentID"]), trim(trade["ExchangeID"])
    direction = trim(trade["Direction"])
    price, volume = number(trade, "Price"), integer(trade, "Volume")
    checks = Counter()

    orders = cli.qry_order(instrument)
    matched_orders = [o for o in orders if (trim(cvalue(o.get("InstrumentID"))) == instrument and
                     trim(cvalue(o.get("ExchangeID"))) == exchange and
                     cvalue(o.get("Direction")) == direction and
                     abs(float(o["LimitPrice"]) - number(order, "LimitPrice")) < 1e-9 and
                     o["VolumeTotalOriginal"] == integer(order, "VolumeTotalOriginal") and
                     cvalue(o.get("CombOffsetFlag")) == "0" and
                     cvalue(o.get("CombHedgeFlag")) == "1" and
                     trim(cvalue(o.get("InvestUnitID"))) == "")]
    assert len(matched_orders) == 1, {"orders": len(matched_orders), "rows": orders}
    got_order = matched_orders[0]
    for field in ("TimeCondition", "VolumeCondition", "MinVolume"):
        assert cvalue(got_order[field]) == cvalue(order[field]), (field, got_order[field], order[field])
    checks["qry_order_core_fields"] += 1

    trades = cli.qry_trade(instrument)
    matched_trades = [t for t in trades if (trim(cvalue(t.get("InstrumentID"))) == instrument and
                      trim(cvalue(t.get("ExchangeID"))) == exchange and
                      cvalue(t.get("Direction")) == direction and
                      cvalue(t.get("OffsetFlag")) == "0" and
                      abs(float(t["Price"]) - price) < 1e-9 and t["Volume"] == volume)]
    assert len(matched_trades) == 1, {"trades": len(matched_trades), "rows": trades}
    checks["qry_trade_core_fields"] += 1

    positions = cli.qry_investor_position(instrument)
    assert positions, positions
    pos = [p for p in positions if trim(cvalue(p.get("InstrumentID"))) == instrument and
           trim(cvalue(p.get("ExchangeID"))) == exchange and
           cvalue(p.get("PosiDirection")) == ("2" if direction == "0" else "3")]
    assert len(pos) == 1, positions
    assert pos[0]["Position"] == volume and pos[0]["TodayPosition"] == volume, pos[0]
    checks["qry_investor_position_qty_today"] += 1

    details = cli.qry_investor_position_detail(instrument)
    assert len(details) == 1, details
    detail = details[0]
    assert detail["InstrumentID"] == instrument and detail["ExchangeID"] == exchange
    assert detail["OpenDate"] == trim(trade["TradingDay"])
    assert abs(detail["OpenPrice"] - price) < 1e-9 and detail["Volume"] == volume, detail
    checks["qry_investor_position_detail_open"] += 1
    expected = {( (exchange, instrument, "", canonical_hedge("1")), "long" if direction == "0" else "short"): volume}
    actual = [{"ExchangeID": exchange, "InstrumentID": instrument, "HedgeFlag": "1",
               "PosiDirection": str(pos[0]["PosiDirection"]), "Position": str(pos[0]["Position"])}]
    assert compare_position_query(actual, expected) == []
    return checks


def write_report(path, report):
    report_path = Path(path)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def make_report(status, checks, skips, errors, candidate_count):
    return {"status": status, "scope": "controlled_real_futures_open_subset; not_full_account_replay",
            "account_scope": "isolated_anonymous_fixture; no_claim_of_real_account_empty_baseline",
            "candidate_count": candidate_count, "checks": dict(checks), "skips": dict(skips),
            "errors": errors[:20], "error_count": len(errors)}


def main(argv=None):
    import argparse
    parser = argparse.ArgumentParser(description="四查询真实期货受控子集验收")
    parser.add_argument("--report", default=REPORT_DEFAULT, help="显式写入匿名 JSON 汇总报告")
    args = parser.parse_args(argv)
    selected, reasons = choose_row()
    if selected is None:
        status = "fail" if source_files() else "skip"
        report = make_report(status, Counter(), reasons, [], 0)
        print("M4 REAL CONTROLLED REPLAY: %s" % status.upper())
        print(json.dumps(report, ensure_ascii=False, indent=2))
        if args.report:
            write_report(args.report, report)
        return 1 if status == "fail" else 0

    checks, errors = Counter(), []
    proc = admin = cli = None
    try:
        with tempfile.TemporaryDirectory(prefix="ctpbuddy-real-replay-") as root:
            sdir = os.path.join(root, "scenario"); os.makedirs(sdir); scenario(sdir, selected)
            td, ap = free_port(), free_port()
            core = os.environ.get("CTPBUDDY_CORE", os.path.join(REPO, "core", "target", "debug", "ctpbuddy-server.exe"))
            proc = subprocess.Popen([core, "--td", "127.0.0.1:%d" % td, "--admin", "127.0.0.1:%d" % ap,
                                     "--broker-id", BROKER, "--initial-funds", "2000000", "--data-dir", os.path.join(root, "data"),
                                     "--scenario", sdir], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            admin = wait_admin(ap)
            admin.start_scenario(sdir, paused=True)
            admin.step(); wait_idx(admin, 1)
            cli = Client("127.0.0.1:%d" % td)
            cli.auth(BROKER, "m4-controlled"); cli.login(BROKER, "m4-controlled"); cli.settle_confirm()
            order, trade = selected["order"], selected["trade"]
            cli.order_insert(trim(trade["InstrumentID"]), direction=trim(trade["Direction"]), offset="0",
                             volume=integer(order, "VolumeTotalOriginal"), limit_price=number(order, "LimitPrice"),
                             exchange=trim(order["ExchangeID"]), order_ref="M4-REAL-1",
                             time_condition=trim(order["TimeCondition"]), volume_condition=trim(order["VolumeCondition"]),
                             min_volume=integer(order, "MinVolume"))
            deadline = time.time() + 5
            while time.time() < deadline:
                if any(t.get("OrderRef") == "M4-REAL-1" for t in cli.qry_trade()): break
                time.sleep(.05)
            checks.update(query_assertions(cli, selected))
            report = make_report("pass", checks, reasons, errors, 1)
            print("M4 REAL CONTROLLED REPLAY: PASS")
    except Exception as exc:
        errors.append(str(exc))
        report = make_report("fail", checks, reasons, errors, 1)
        print("M4 REAL CONTROLLED REPLAY: FAIL")
    finally:
        if cli is not None:
            try: cli.close()
            except Exception: pass
        if admin is not None:
            try: admin.shutdown()
            except Exception: pass
        if proc is not None and proc.poll() is None:
            proc.kill(); proc.wait(timeout=5)
    print(json.dumps(report, ensure_ascii=False, indent=2))
    if args.report:
        write_report(args.report, report)
    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
