#!/usr/bin/env python
"""虚构 fixture 的真实 CTPBuddy TCP 服务器大单边保证金回归。"""
from __future__ import annotations
import json
import os
import subprocess
import sys
import tempfile

from m3_refdata import find_core, free_port, wait_port, wait_idx, wait_fills, close

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "py"))
from ctpbuddy.sdk import Admin, Client, CTPError
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical
from ctpbuddy import generated
from ctpbuddy.wire import REQ_QRY_INVESTOR_PRODUCT_GROUP_MARGIN, RSP_QRY_INVESTOR_PRODUCT_GROUP_MARGIN

# 合约、费率、初始资金全部是明确虚构的测试输入，不使用生产默认费率。
IDS = ("XA2601", "XA2602", "XB2601", "XC2601", "XC2602")


def fixture(root):
    ref = os.path.join(root, "ref")
    scenario = os.path.join(root, "scenario")
    os.makedirs(ref)
    os.makedirs(scenario)
    instruments, rates = [], []
    ticks = []
    for n, iid in enumerate(IDS):
        instruments.append(dict(instrument_id=iid, exchange_id="TEST", product_id=iid[:2],
                                volume_multiple=1, price_tick=1,
                                max_margin_side_algorithm="0" if iid.startswith("XC") else "1",
                                long_margin_ratio=0.01, short_margin_ratio=0.02))
        rates.append(dict(instrument_id=iid, long_margin_ratio_by_money=0.1,
                          short_margin_ratio_by_money=0.15))
        row = {c: "" for c in CANONICAL_COLUMNS}
        row.update(instrument=iid, exchange="TEST", trading_day="20261003",
                   update_time="09:00:%02d" % (n + 1), update_millisec="0",
                   last_price="100", pre_settlement="100", settlement="100", average="100",
                   volume="100", turnover="10000", open_interest="100", upper="300", lower="1",
                   bid1="100", ask1="100", bidvol1="100", askvol1="100")
        ticks.append(row)
    changed = dict(ticks[0], update_time="09:00:06", last_price="200", bid1="200", ask1="200", average="200")
    ticks.append(changed)
    for name, rows in [("instruments", instruments), ("margin_rates", rates),
                       ("trading_params", [dict(margin_price_type="2")])]:
        with open(os.path.join(ref, name + ".jsonl"), "w", encoding="utf-8") as f:
            for row in rows:
                f.write(json.dumps(row) + "\n")
    write_canonical(scenario, ticks)
    return ref, scenario


def main():
    with tempfile.TemporaryDirectory(prefix="ctpbuddy-margin-") as tmp:
        ref, scenario = fixture(tmp)
        td, ap = free_port(), free_port()
        proc = subprocess.Popen([find_core(), "--td", "127.0.0.1:%d" % td,
            "--admin", "127.0.0.1:%d" % ap, "--broker-id", "8888", "--refdata", ref,
            "--initial-funds", "35", "--speed", "0", "--qry-freq", "100", "--order-freq", "100"],
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        admin = None
        try:
            wait_port(ap)
            admin = Admin("127.0.0.1:%d" % ap)
            admin.start_scenario(scenario, paused=True)
            for idx in range(1, 6):
                admin.step()
                wait_idx(admin, idx)
            with Client("127.0.0.1:%d" % td) as cli:
                cli.auth("8888", "margin")
                cli.login("8888", "margin")
                cli.settle_confirm()

                def order(iid, direction, offset, volume, ref, price=100):
                    cli.order_insert(iid, direction, offset, volume, price, exchange="TEST", order_ref=ref)
                    assert len(wait_fills(cli, ref, 1)) == 1, ref

                def check(expected, frozen=0):
                    rows = cli.qry_investor_product_group_margin()
                    a = cli.qry_trading_account()
                    assert close(sum(r["UseMargin"] for r in rows), expected), (rows, expected)
                    assert close(a["CurrMargin"], expected), a
                    assert close(sum(r["FrozenMargin"] for r in rows), frozen), rows
                    assert close(a["FrozenMargin"], frozen), a
                    return rows

                assert cli.qry_investor_product_group_margin() == []
                order("XA2601", "0", "0", 3, "L")
                check(30)
                # 可用仅5元，对侧开仓15元仍应通过，因为取大后无增量。
                order("XA2602", "1", "0", 1, "S")
                rows = check(30)
                xa = next(r for r in rows if r["ProductGroupID"] == "XA")
                assert close(xa["LongUseMargin"], 30) and close(xa["ShortUseMargin"], 15), xa
                assert close(xa["ExchMargin"], 3), xa
                # 最新价模式按委托价估计：空边15+15.15略超多边30，只冻结0.15。
                cli.order_insert("XA2602", "1", "0", 1, 101, exchange="TEST", order_ref="REST")
                check(30, 0.15)
                # 再挂一手会把空边推至45，不能重复利用已有抵扣额度。
                try:
                    cli.order_insert("XA2602", "1", "0", 1, 101, exchange="TEST", order_ref="NO")
                    raise AssertionError("挂单累计风险应资金不足")
                except CTPError as e:
                    assert e.error_id == 31, e
                cli.order_action("XA2602", "REST")
                check(30, 0)
                # 减少多边后切换为空边15。
                order("XA2601", "1", "1", 2, "C")
                check(15)
                order("XB2601", "0", "0", 1, "B")
                check(25)
                assert len(cli.qry_investor_product_group_margin("XA")) == 1
                assert cli.qry_investor_product_group_margin("missing") == []
                payload = generated.pack("CThostFtdcQryInvestorProductGroupMarginField", BrokerID="8888", InvestorID="other")
                assert cli._query_stream(REQ_QRY_INVESTOR_PRODUCT_GROUP_MARGIN, RSP_QRY_INVESTOR_PRODUCT_GROUP_MARGIN, payload) == []
                # 行情变化只更新持仓盈亏；已入账保证金和大边聚合不重估。
                before_move = check(25)
                admin.step()
                wait_idx(admin, 6)
                after_move = check(25)
                assert close(sum(r["UseMargin"] for r in before_move),
                             sum(r["UseMargin"] for r in after_move))
                print("[ok] 跨合约优惠、异品种隔离、非对称、风控/挂单/撤单、平仓切边、行情不重估、查询一致")
            with Client("127.0.0.1:%d" % td) as cli:
                cli.auth("8888", "off")
                cli.login("8888", "off")
                cli.settle_confirm()
                for iid, side, refid in [("XC2601", "0", "N1"), ("XC2602", "1", "N2")]:
                    cli.order_insert(iid, side, "0", 1, 100, exchange="TEST", order_ref=refid)
                    assert len(wait_fills(cli, refid, 1)) == 1
                rows = cli.qry_investor_product_group_margin()
                assert close(sum(r["UseMargin"] for r in rows), 25), rows
                assert close(cli.qry_trading_account()["CurrMargin"], 25)
                print("[ok] 规则关闭保持多空求和")
            print("M3 MARGIN: PASS")
            return 0
        finally:
            if admin:
                try:
                    admin.shutdown()
                except Exception:
                    proc.terminate()
            else:
                proc.terminate()
            out, _ = proc.communicate(timeout=10)
            print(out)


if __name__ == "__main__":
    sys.exit(main())
