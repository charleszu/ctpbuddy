#!/usr/bin/env python
"""任务42：显式日结基础闭环。"""
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from m3_refdata import BROKER, RB, M, RB_PRE_SETTLE, find_core, free_port, wait_fills, wait_port  # noqa: E402
from ctpbuddy.calendar import TradingCalendar  # noqa: E402
from ctpbuddy.sdk import Admin, Client, CTPError  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402


def scenario(path: str) -> None:
    row = {c: "" for c in CANONICAL_COLUMNS}
    row.update(
        instrument=RB, exchange="SHFE", trading_day="20261003", update_time="09:00:01",
        last_price="3500", volume="100", turnover="350000", open_interest="5000",
        pre_settlement=str(RB_PRE_SETTLE), settlement=str(RB_PRE_SETTLE), pre_close="3499",
        open="3500", high="3502", low="3498", close="3500", upper="3839", lower="3141",
        pre_open_interest="4980", average="3500", bid1="3498", ask1="3502",
        bidvol1="20", askvol1="20",
    )
    write_canonical(path, [row])


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="ctpbuddy-settlement-") as root:
        sdir = os.path.join(root, "scenario")
        os.makedirs(sdir)
        scenario(sdir)
        shutil.copytree(os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), "refdata"), os.path.join(sdir, "refdata"))
        td, ap = free_port(), free_port()
        data = os.path.join(root, "data")
        proc = subprocess.Popen([find_core(), "--td", f"127.0.0.1:{td}", "--admin", f"127.0.0.1:{ap}", "--broker-id", BROKER, "--speed", "0", "--data-dir", data, "--scenario", sdir], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        try:
            wait_port(ap)
            calendar = TradingCalendar.from_json({
                "schema": "ctpbuddy.trading-calendar/v1",
                "version": "e2e-2026-r1",
                "source": {"kind": "fixture", "name": "m3_settlement", "revision": "r1", "license": "test", "scope": "futures"},
                "days": [
                    {"date": "2026-10-02", "is_trading_day": True, "trading_day": "20261002"},
                    {"date": "2026-10-03", "is_trading_day": True, "trading_day": "20261003"},
                    {"date": "2026-10-04", "is_trading_day": False},
                    {"date": "2026-10-05", "is_trading_day": False},
                    {"date": "2026-10-06", "is_trading_day": True, "trading_day": "20261006"},
                ],
            })
            with Admin(f"127.0.0.1:{ap}", calendar=calendar) as admin:
                admin.start_scenario(sdir, paused=True, spec={"name": "settlement-fictional", "accounts": [{
                    "investor": "carry002", "balance": 1000000, "positions": [{
                        "instrument": M, "exchange": "DCE", "direction": "short",
                        "open_date": "20261002", "trade_id": "CARRY", "open_price": 3000,
                        "volume": 1, "pre_settlement": 2990, "margin": 1000,
                    }],
                }]})
                other = Client(f"127.0.0.1:{td}")
                other.auth(BROKER, "carry002")
                other.login(BROKER, "carry002")
                other.settle_confirm()
                cli = Client(f"127.0.0.1:{td}")
                cli.auth(BROKER, "settle001")
                cli.login(BROKER, "settle001", password="")
                cli.settle_confirm()
                admin.step()
                deadline = time.time() + 5
                while time.time() < deadline and admin.status()["playback"]["idx"] < 1:
                    time.sleep(0.02)
                cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3502, exchange="SHFE", order_ref="S1")
                assert wait_fills(cli, "S1", 1), "开仓未成交"
                before = cli.qry_trading_account()
                try:
                    admin.settle_day({RB: 3501.0}, "20261003")
                    raise AssertionError("非递增 next_trading_day 被接受")
                except RuntimeError:
                    pass
                unchanged = cli.qry_trading_account()
                assert unchanged["PreBalance"] == before["PreBalance"]
                admin.resume()
                try:
                    admin.settle_day({RB: 3501.0}, "20261004")
                    raise AssertionError("未暂停 playback 的日结被接受")
                except RuntimeError:
                    pass
                admin.pause()
                try:
                    admin.settle_day({}, "20261004")
                    raise AssertionError("缺价日结被接受")
                except RuntimeError:
                    pass
                snapshots = (cli.qry_trading_account(), cli.qry_investor_position(), cli.qry_investor_position_detail(), cli.qry_order(), cli.qry_trade(), other.qry_trading_account(), other.qry_investor_position_detail())
                for prices, day in [({RB: 3501}, "20261004"), ({RB: 0, M: 2980}, "20261004"), ({RB: -1, M: 2980}, "20261004"), ({RB: "bad", M: 2980}, "20261004"), ({RB: 3501, M: 2980}, "20260230"), ({RB: 3501, M: 2980}, "2026-10-04"), ({RB: 3501, M: 2980}, "20261003")]:
                    try:
                        admin.settle_day(prices, day)
                        raise AssertionError("非法/缺价日结被接受")
                    except RuntimeError:
                        pass
                    after = (cli.qry_trading_account(), cli.qry_investor_position(), cli.qry_investor_position_detail(), cli.qry_order(), cli.qry_trade(), other.qry_trading_account(), other.qry_investor_position_detail())
                    assert after == snapshots, (prices, day, snapshots, after)
                    assert admin.status()["playback"]["trading_day"] == "20261003"
                cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3490, exchange="SHFE", order_ref="REST")
                time.sleep(0.1)
                try:
                    admin.settle_day({RB: 3501.0}, "20261004")
                    raise AssertionError("活动订单检查未生效")
                except RuntimeError:
                    pass
                cli.order_action(RB, "REST")
                time.sleep(0.1)
                details_before = cli.qry_investor_position_detail()
                result = admin.settle_day({RB: 3501.0, M: 2980.0})
                assert result["trading_day"] == "20261006", result
                rows = cli.qry_investor_position(RB)
                assert rows and all(r["TodayPosition"] == 0 for r in rows), rows
                carry = other.qry_investor_position_detail(M)
                assert carry and carry[0]["OpenDate"] == "20261002", carry
                acct = cli.qry_trading_account()
                for key in ("Deposit", "Withdraw", "CloseProfit", "PositionProfit", "Commission", "FrozenMargin", "FrozenCommission"):
                    assert abs(acct[key]) < 1e-8, (key, acct)
                assert abs(acct["PreBalance"] - acct["Balance"]) < 1e-8, acct
                assert abs(acct["PreBalance"] - 1999990.0) < 1e-8, acct
                other_acct = other.qry_trading_account()
                assert abs(other_acct["PreBalance"] - 1000100.0) < 1e-8, other_acct
                assert rows[0]["YdPosition"] == 1 and rows[0]["PositionDate"] == ord("2"), rows
                details_after = cli.qry_investor_position_detail()
                for key in ("OpenDate", "TradeID", "OpenPrice", "Direction", "Volume"):
                    assert details_after[0][key] == details_before[0][key], details_after
                assert details_after[0]["LastSettlementPrice"] == 3501.0
                assert rows[0]["PreSettlementPrice"] == 3501.0 and rows[0]["SettlementPrice"] == 3501.0
                assert cli.qry_trade() == [] and cli.qry_order() == []
                try:
                    cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3502, exchange="SHFE", order_ref="S2")
                    raise AssertionError("下一交易日未重新确认仍可报单")
                except CTPError as e:
                    assert e.error_id == 42, e
                cli.settle_confirm()
                cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3502, exchange="SHFE", order_ref="S2")
                cli.close()
                other.close()
                admin.shutdown()
            proc.wait(timeout=5)
            events = []
            for name in os.listdir(os.path.join(data, "journal")):
                with open(os.path.join(data, "journal", name), encoding="utf-8") as f:
                    events.extend(json.loads(line) for line in f if line.strip())
            settlements = [e for e in events if e["type"] == "settlement"]
            assert len(settlements) == 1, settlements
            event = settlements[0]
            assert event["trading_day"] == "20261003"
            assert event["data"]["next_trading_day"] == "20261006"
            assert event["data"]["settlement_prices"] == {RB: 3501.0, M: 2980.0}
            assert {a["investor"] for a in event["data"]["accounts"]} == {"settle001", "carry002"}
            assert event["data"]["cleared_confirmations"] == 2
            assert event["data"]["cleared_trades"] == 1
            assert proc.returncode == 0, proc.returncode
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.wait(timeout=5)
    print("M3 SETTLEMENT E2E: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
