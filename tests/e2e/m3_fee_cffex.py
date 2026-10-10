#!/usr/bin/env python
"""CFFEX 平今费时间序池 (知识库 §6.4/§10.4 #10；IM2410/20240924 生产判别 81/81).

中金所没有平今指令：持仓消耗一律先开先平，`Close`/`CloseToday` 在柜台侧
同义。但平今**手续费**走另一条轴——按成交时间序的开仓池「先平当日新开仓，
再平历史仓」，与被平明细的年龄互不参考。生产实锤（IM2410/20240924）：

  - 第 3 笔卖平：被平明细是 9/23 昨仓，池有余 → 收平今费 211.65；
  - 序号 1095839 卖平：被平明细是当日今仓，池已耗尽 → 收平昨费 21.41。

本 e2e 在单日内复刻这两个方向（bootstrap 昨仓 + 当日开仓 1 手，两笔平仓）：

  断言①  平 1 手先开先平吃掉**昨仓**，手续费却是平今 (1200.0)——
          年龄模型会收平昨 (120.0)，差 10 倍。
  断言②  再平 1 手吃掉**当日今仓**，池已耗尽 → 收平昨 (120.0)——
          年龄模型会收平今 (1200.0)。

费率经 scenario 自带 `refdata/commission_rates.jsonl` 注入（desk 数据集协议，
随包数据刻意不含手续费表），并断言查询面 `ReqQryInstrumentCommissionRate`
返回的就是这行——一份费率，查询面与计算面共用。「昨仓隔日不入池、日结清
池」由 ledger 单元测试锁定（当前场景 DSL 为单交易日）。

Usage:
    python tests/e2e/m3_fee_cffex.py
Env:
    CTPBUDDY_CORE   explicit path to the ctpbuddy-server binary
"""
from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from m3_refdata import BROKER, drain, find_core, free_port, wait_fills, wait_idx, wait_port  # noqa: E402

from ctpbuddy.sdk import Admin, Client  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
IF = "IF2509"          # CFFEX, mult 300, tick 0.2 — real contract from the bundled snapshot
TRADING_DAY = "20261009"
PRE_SETTLE = 4000.0
MULT = 300
CLOSE_YD_RATIO = 0.0001    # 平昨 万1 → 4000 × 300 × 0.0001 = 120.0 / 手
CLOSE_TD_RATIO = 0.001     # 平今 万10 → 4000 × 300 × 0.001  = 1200.0 / 手 (10×)
FEE_YD = PRE_SETTLE * MULT * CLOSE_YD_RATIO
FEE_TD = PRE_SETTLE * MULT * CLOSE_TD_RATIO


def scenario(path: str) -> None:
    """One flat print repeated: bid1 == ask1 == 4000 so both the opening buy
    and the closing sells fill at exactly 4000 (fees stay round numbers)."""
    rows = []
    for i, t in enumerate(("09:00:01", "09:00:02", "09:00:03")):
        row = {c: "" for c in CANONICAL_COLUMNS}
        row.update(
            instrument=IF, exchange="CFFEX", trading_day=TRADING_DAY,
            update_time=t, update_millisec="0",
            last_price="4000", volume="100", turnover="1200000", open_interest="5000",
            pre_settlement=str(PRE_SETTLE), settlement=str(PRE_SETTLE), pre_close="4000",
            open="4000", high="4000", low="4000", close="4000",
            upper="4400", lower="3600",
            pre_open_interest="4980", average="4000",
            bid1="4000", ask1="4000", bidvol1="20", askvol1="20",
        )
        rows.append(row)
    write_canonical(path, rows)


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ctpbuddy-feecffex-") as root:
        sdir = os.path.join(root, "scenario")
        os.makedirs(sdir)
        scenario(sdir)
        # desk 数据集协议：scenario/refdata = 随包三张表 + 自带手续费表。
        shutil.copytree(os.path.join(REPO, "refdata"), os.path.join(sdir, "refdata"))
        with open(os.path.join(sdir, "refdata", "commission_rates.jsonl"), "w",
                  encoding="utf-8") as f:
            f.write('{"instrument_id": "%s", "exchange_id": "CFFEX", '
                    '"close_ratio_by_money": %s, "close_today_ratio_by_money": %s}\n'
                    % (IF, CLOSE_YD_RATIO, CLOSE_TD_RATIO))

        td, ap = free_port(), free_port()
        data = os.path.join(root, "data")
        proc = subprocess.Popen(
            [find_core(), "--td", f"127.0.0.1:{td}", "--admin", f"127.0.0.1:{ap}",
             "--broker-id", BROKER, "--speed", "0", "--data-dir", data,
             "--scenario", sdir],
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        try:
            wait_port(ap)
            with Admin(f"127.0.0.1:{ap}") as admin:
                admin.start_scenario(sdir, paused=True, spec={
                    "name": "cffex-fee-pool-fictional",
                    "accounts": [{
                        "investor": "fee001", "balance": 2_000_000,
                        # 昨日 bootstrap 昨仓 1 手（open_date 昨日）：池不收纳它，
                        # 但先开先平会先消耗它——两轴分叉的支点。
                        "positions": [{
                            "instrument": IF, "exchange": "CFFEX", "direction": "long",
                            "open_date": "20261008", "trade_id": "YD1",
                            "open_price": 4000.0, "volume": 1,
                            "pre_settlement": 4000.0, "margin": 180_000.0,
                        }],
                    }],
                })
                cli = Client(f"127.0.0.1:{td}")
                cli.auth(BROKER, "fee001")
                cli.login(BROKER, "fee001", password="")
                cli.settle_confirm()
                cli.subscribe([IF])
                drain(cli)

                admin.step()
                wait_idx(admin, 1)
                drain(cli)

                # -- 0) 查询面 == 计算面: the injected row comes back ----------
                rows = cli.qry_instrument_commission_rate(IF)
                assert len(rows) == 1 and rows[0]["InstrumentID"] == IF, rows
                assert abs(rows[0]["CloseRatioByMoney"] - CLOSE_YD_RATIO) < 1e-12, rows
                assert abs(rows[0]["CloseTodayRatioByMoney"] - CLOSE_TD_RATIO) < 1e-12, rows
                print("[ok] ReqQryInstrumentCommissionRate returns the injected desk row")

                # -- 1) 买开 1 手 @4000: 池 = 1（昨仓不入池）-------------------
                cli.order_insert(IF, direction="0", offset="0", volume=1,
                                 limit_price=4000.0, exchange="CFFEX", order_ref="F1")
                assert len(wait_fills(cli, "F1", 1)) == 1, "F1 should fill"
                drain(cli)

                # -- 2) 断言①: 平 1 手吃昨仓，收平今费 -------------------------
                # 明细轴（先开先平）耗掉昨仓；费用轴（时间序池）池=1 有余 →
                # 平今费。IM2410/20240924 第 3 笔的同构。
                admin.step()
                wait_idx(admin, 2)
                drain(cli)
                cli.order_insert(IF, direction="1", offset="1", volume=1,
                                 limit_price=3999.0, exchange="CFFEX", order_ref="F2")
                assert len(wait_fills(cli, "F2", 1)) == 1, "F2 should fill"
                drain(cli)
                pos = cli.qry_investor_position(IF)[0]
                # YdPosition 是静态初值字段（官方语义：昨日结算时持仓），平仓
                # 不改它；剩余量看 Position/TodayPosition。CFFEX 单行投影。
                assert pos["Position"] == 1 and pos["TodayPosition"] == 1, pos
                acct = cli.qry_trading_account()
                assert abs(acct["Commission"] - FEE_TD) < 1e-6, (acct, FEE_TD)
                print("[ok] ① 昨仓被平收平今费: carried lot closed at today's rate %.1f (age model: %.1f)"
                      % (FEE_TD, FEE_YD))

                # -- 3) 断言②: 池耗尽后平今仓收平昨费 -------------------------
                # 明细轴只剩当日今仓；费用轴池已空 → 平昨费。IM2410 序号
                # 1095839 的同构（平当日开仓、收平昨费 21.41）。
                admin.step()
                wait_idx(admin, 3)
                drain(cli)
                before = cli.qry_trading_account()["Commission"]
                cli.order_insert(IF, direction="1", offset="1", volume=1,
                                 limit_price=3999.0, exchange="CFFEX", order_ref="F3")
                assert len(wait_fills(cli, "F3", 1)) == 1, "F3 should fill"
                drain(cli)
                acct = cli.qry_trading_account()
                assert abs(acct["Commission"] - before - FEE_YD) < 1e-6, (acct, FEE_YD)
                pos = cli.qry_investor_position(IF)
                # bootstrap 初仓全平后 CTP 可保留零 Position 行直至日结。
                assert len(pos) == 1 and pos[0]["Position"] == 0, pos
                print("[ok] ② 池耗尽后平今仓收平昨费: today lot closed at yesterday's rate %.1f (age model: %.1f)"
                      % (FEE_YD, FEE_TD))

                # -- 4) 明细轴自证: 两次平仓吃的是不同年龄的两笔明细 ------------
                # （先开先平的可见证据——若无昨仓，两笔平仓将吃同一笔明细。）
                print("[ok] 先开先平 axis: ① took the carried lot, ② the today lot")
                cli.close()
                admin.shutdown()
            proc.wait(timeout=5)
            assert proc.returncode == 0, proc.returncode
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.wait(timeout=5)
    print("M3 FEE CFFEX: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
