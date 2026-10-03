#!/usr/bin/env python
"""任务41：真实 core 启动初仓、查询、平昨与流水边界。fixture 为虚构数据。"""
from __future__ import annotations

import json
import os
import queue
import shutil
import socket
import subprocess
import sys
import tempfile
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "py"))

from ctpbuddy.generated import structs  # noqa: E402
from ctpbuddy.scenario import ScenarioError, compile_scenario, normalize_spec, parse_yaml  # noqa: E402
from ctpbuddy.sdk import Client  # noqa: E402
from ctpbuddy.sdk.admin import Admin  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402

BROKER = "8888"
INVESTOR = "boot001"
RB = "rb2601"
DCE = "jd2602"
DAY = "20261003"
PRE_SETTLE = 3490.0

FIXTURE = {
    "name": "m3-bootstrap-fictional",
    "accounts": [{
        "investor": INVESTOR,
        "balance": 1000000,
        "positions": [{
            "instrument": RB, "exchange": "SHFE", "direction": "long",
            "open_date": "20261002", "trade_id": "BOOT-YD-1",
            "open_price": 3500.0, "volume": 1, "pre_settlement": PRE_SETTLE,
        }, {
            "instrument": DCE, "exchange": "DCE", "direction": "long",
            "open_date": "20261002", "trade_id": "BOOT-DCE-1",
            "open_price": 3000.0, "volume": 1, "pre_settlement": 2990.0,
            "margin": 1234.5,
        }],
    }],
}


def port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def wait_admin(addr: str) -> None:
    end = time.time() + 10
    while time.time() < end:
        try:
            with Admin(addr) as a:
                a.ping()
            return
        except OSError:
            time.sleep(0.05)
    raise AssertionError("ADMIN 未启动")


def make_ticks(path: str) -> None:
    row = {c: "" for c in CANONICAL_COLUMNS}
    row.update(
        instrument=RB, exchange="SHFE", trading_day=DAY,
        update_time="09:00:01", update_millisec="0", last_price="3500",
        volume="100", turnover="350000", open_interest="5000",
        pre_settlement=str(PRE_SETTLE), settlement=str(PRE_SETTLE),
        pre_close="3499", open="3500", high="3502", low="3498", close="3500",
        upper="3839", lower="3141", pre_open_interest="4980", average="3500",
        bid1="3498", bid2="3497", bid3="3496", bid4="3495", bid5="3494",
        ask1="3502", ask2="3503", ask3="3504", ask4="3505", ask5="3506",
        bidvol1="20", bidvol2="20", bidvol3="20", bidvol4="20", bidvol5="20",
        askvol1="20", askvol2="20", askvol3="20", askvol4="20", askvol5="20",
    )
    write_canonical(path, [row])


def schema_checks() -> None:
    spec = normalize_spec(FIXTURE)
    assert spec["accounts"][0]["positions"][0]["direction"] == "long"
    yaml = """name: m3-bootstrap-fictional
accounts:
  - investor: boot001
    balance: 1000000
    positions:
      - instrument: rb2601
        exchange: SHFE
        direction: long
        open_date: "20261002"
        trade_id: BOOT-YD-1
        open_price: 3500
        volume: 1
        pre_settlement: 3490
"""
    assert normalize_spec(parse_yaml(yaml))["accounts"][0]["positions"]
    for bad in (
        {"accounts": [{"investor": "x", "positions": [{**FIXTURE["accounts"][0]["positions"][0], "volume": 1.5}]}]},
        {"accounts": [{"investor": "x", "positions": [{**FIXTURE["accounts"][0]["positions"][0], "open_date": "2026-10-03"}]}]},
        {"accounts": [{"investor": "x", "positions": [{**FIXTURE["accounts"][0]["positions"][0]}, {**FIXTURE["accounts"][0]["positions"][0]}]}]},
        {"accounts": [{"investor": "x", "positions": [{**FIXTURE["accounts"][0]["positions"][0], "volume": True}]}]},
        {"accounts": [{"investor": "x", "positions": [{**FIXTURE["accounts"][0]["positions"][0], "open_date": "20260230"}]}]},
        {"accounts": [{"investor": "x", "positions": {}}]},
        {"accounts": [{"investor": "x", "positions": [{**FIXTURE["accounts"][0]["positions"][0], "margin": "bad"}]}]},
        {"accounts": [{"investor": "x", "positions": [{**FIXTURE["accounts"][0]["positions"][0], "volume": 2147483648}]}]},
    ):
        try:
            normalize_spec(bad)
        except ScenarioError:
            pass
        else:
            raise AssertionError("非法初仓被接受: %r" % bad)


def main() -> int:
    schema_checks()
    with tempfile.TemporaryDirectory(prefix="ctpbuddy-bootstrap-e2e-") as root:
        scenario = os.path.join(root, "scenario")
        os.makedirs(scenario)
        make_ticks(scenario)
        shutil.copytree(os.path.join(REPO, "refdata"), os.path.join(scenario, "refdata"))
        with open(os.path.join(scenario, "refdata", "trading_params.jsonl"), "w", encoding="utf-8") as f:
            f.write(json.dumps({"margin_price_type": "4"}) + "\n")
        with open(os.path.join(scenario, "scenario.yaml"), "w", encoding="utf-8") as f:
            f.write("""name: m3-bootstrap-fictional
source:
  kind: csv
  path: ticks.csv
clock:
  time_scale: 0
accounts:
  - investor: boot001
    balance: 1000000
    positions:
      - instrument: rb2601
        exchange: SHFE
        direction: long
        open_date: "20261002"
        trade_id: BOOT-YD-1
        open_price: 3500
        volume: 1
        pre_settlement: 3490
      - instrument: jd2602
        exchange: DCE
        direction: long
        open_date: "20261002"
        trade_id: BOOT-DCE-1
        open_price: 3000
        volume: 1
        pre_settlement: 2990
        margin: 1234.5
""")
        compile_scenario(scenario)
        td, ap = port(), port()
        data = os.path.join(root, "data")
        core = os.environ.get("CTPBUDDY_CORE", os.path.join(REPO, "target", "task41", "debug", "ctpbuddy-server.exe"))
        proc = subprocess.Popen([
            core, "--td", f"127.0.0.1:{td}", "--admin", f"127.0.0.1:{ap}",
            "--broker-id", BROKER, "--speed", "0", "--initial-funds", "2000000",
            "--qry-freq", "32", "--data-dir", data, "--scenario", scenario,
        ], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        try:
            admin_addr = f"127.0.0.1:{ap}"
            wait_admin(admin_addr)
            with Admin(admin_addr) as admin:
                before = admin.status()
                bad_spec = {"name": "bad", "accounts": [{"investor": INVESTOR, "positions": [{"instrument": RB, "exchange": "SHFE", "direction": "long", "open_date": "20260230", "trade_id": "BAD", "open_price": 1, "volume": 1, "pre_settlement": 1}]}]}
                try:
                    admin.start_scenario(scenario, paused=True, spec=bad_spec)
                except RuntimeError:
                    pass
                else:
                    raise AssertionError("非法初仓整批被接受")
                after = admin.status()
                assert after["scenario"] == before["scenario"] and after["playback"]["idx"] == before["playback"]["idx"], (before, after)
                print("[ok] invalid bootstrap batch rejected with world state unchanged")
            cli = Client(f"127.0.0.1:{td}")
            cli.auth(BROKER, INVESTOR)
            cli.login(BROKER, INVESTOR, password="")
            cli.settle_confirm()

            rows = cli.qry_investor_position(RB)
            assert len(rows) == 1, rows
            row = rows[0]
            assert row["Position"] == 1 and row["YdPosition"] == 1, row
            assert row["TodayPosition"] == 0 and row["PositionDate"] == ord("2"), row
            details = cli.qry_investor_position_detail(RB)
            assert len(details) == 1 and details[0]["Volume"] == 1, details
            assert cli.qry_trade() == [], "初仓不能制造 Trade 流水"
            assert cli.qry_order() == [], "初仓不能制造 Order 流水"
            dce_rows = cli.qry_investor_position(DCE)
            assert len(dce_rows) == 1 and dce_rows[0]["PositionDate"] == ord("1") and dce_rows[0]["YdPosition"] == 1, dce_rows
            assert dce_rows[0]["UseMargin"] == 1234.5, dce_rows
            print("[ok] startup bootstrap: SHFE Position=1 YdPosition=1 PositionDate=2")
            print("[ok] DCE single row: PositionDate=1 YdPosition=1 and explicit margin preserved")
            print("[ok] bootstrap detail exists; QryTrade/QryOrder empty")

            cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3502.0, exchange="SHFE", order_ref="OPEN-TODAY")
            deadline = time.time() + 5
            today_trades = []
            while time.time() < deadline:
                today_trades = [t for t in cli.qry_trade() if t["OrderRef"] == "OPEN-TODAY"]
                if today_trades:
                    break
                time.sleep(0.05)
            assert len(today_trades) == 1, today_trades
            mixed = cli.qry_investor_position(RB)
            assert {r["PositionDate"] for r in mixed} == {ord("1"), ord("2")}, mixed
            yd_row = next(r for r in mixed if r["PositionDate"] == ord("2"))
            td_row = next(r for r in mixed if r["PositionDate"] == ord("1"))
            assert yd_row["YdPosition"] == 1 and td_row["YdPosition"] == 0
            assert yd_row["PositionCost"] == 34900.0 and td_row["PositionCost"] == 35020.0, mixed
            assert abs(yd_row["UseMargin"] - 5584.0) < 1e-6 and abs(td_row["UseMargin"] - 5603.2) < 1e-6, mixed
            print("[ok] mixed SHFE query: detail-age rows keep distinct PositionCost and static YdPosition")

            cli.order_insert(RB, direction="1", offset="1", volume=1, limit_price=3498.0, exchange="SHFE", order_ref="CLOSE-YD")
            deadline = time.time() + 5
            trades = []
            while time.time() < deadline:
                trades = [t for t in cli.qry_trade() if t["OrderRef"] == "CLOSE-YD"]
                if trades:
                    break
                time.sleep(0.05)
            assert len(trades) == 1, trades
            after = cli.qry_investor_position(RB)
            assert len(after) == 2, after
            yd_after = next(r for r in after if r["PositionDate"] == ord("2"))
            td_after = next(r for r in after if r["PositionDate"] == ord("1"))
            assert yd_after["Position"] == 0 and yd_after["YdPosition"] == 1
            assert td_after["Position"] == 1 and td_after["YdPosition"] == 0
            details_after = cli.qry_investor_position_detail(RB)
            assert len(details_after) == 1 and details_after[0]["Volume"] == 1, details_after
            print("[ok] close yesterday: current yd Position decreases while static YdPosition=1 remains")
            print("[ok] zero-volume yd detail is filtered; today detail remains visible")
            cli.order_insert(RB, direction="1", offset="3", volume=1, limit_price=3498.0, exchange="SHFE", order_ref="CLOSE-TODAY")
            deadline = time.time() + 5
            while time.time() < deadline and not [t for t in cli.qry_trade() if t["OrderRef"] == "CLOSE-TODAY"]:
                time.sleep(0.05)
            final_rows = cli.qry_investor_position(RB)
            final_yd = next(r for r in final_rows if r["PositionDate"] == ord("2"))
            assert final_yd["Position"] == 0 and final_yd["YdPosition"] == 1
            assert cli.qry_investor_position_detail(RB) == [], "全平后零余量 detail 不返回"
            print("[ok] full close keeps static SHFE row and removes all zero-volume details")
            cli.close()
            with Admin(admin_addr) as a:
                a.shutdown()
            proc.wait(timeout=5)
            journal = []
            jdir = os.path.join(data, "journal")
            for name in os.listdir(jdir):
                with open(os.path.join(jdir, name), encoding="utf-8") as f:
                    journal.extend(json.loads(line) for line in f if line.strip())
            bootstrap_fills = [e for e in journal if e["type"] == "fill" and e.get("data", {}).get("trade_id") == "BOOT-YD-1"]
            assert not bootstrap_fills
        finally:
            if proc.poll() is None:
                proc.terminate()
                proc.wait(timeout=5)
    print("M3 BOOTSTRAP E2E: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
