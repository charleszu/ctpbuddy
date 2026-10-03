#!/usr/bin/env python
"""任务43：OrderSysID 首条为空、accepted 后最终非空且关联稳定。"""
from __future__ import annotations

import os
import queue
import socket
import subprocess
import sys
import tempfile
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "py"))

from ctpbuddy.sdk import Admin, Client  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402
from ctpbuddy.wire import ERR_RTN_ORDER_INSERT, RTN_ORDER, RTN_TRADE  # noqa: E402
from ctpbuddy import generated  # noqa: E402

BROKER = "8888"
RB = "rb2601"
JD = "jd2602"
DAY = "20261002"


def free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def wait_port(port: int) -> None:
    deadline = time.time() + 10
    while time.time() < deadline:
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.5).close()
            return
        except OSError:
            time.sleep(0.05)
    raise SystemExit("server did not open port %d" % port)


def drain(cli: Client, quiet: float = 0.3):
    out = []
    while True:
        try:
            out.append(next(cli.events(timeout=quiet)))
        except (StopIteration, queue.Empty):
            return out


def orders(events, ref):
    return [f for k, f in events if k == RTN_ORDER and f["OrderRef"] == ref]


def trades(events, ref):
    return [f for k, f in events if k == RTN_TRADE and f["OrderRef"] == ref]


def make_scenario(path: str) -> None:
    rows = []

    def tick(instrument, exchange, last, bid, ask):
        r = {c: "" for c in CANONICAL_COLUMNS}
        r.update(
            instrument=instrument, exchange=exchange, trading_day=DAY,
            update_time="09:30:00", update_millisec="0", last_price=str(last),
            volume="100", turnover="350000", open_interest="5000",
            pre_settlement=str(last), settlement=str(last), pre_close=str(last),
            open=str(last), high=str(ask), low=str(bid), close=str(last),
            upper=str(last * 1.1), lower=str(last * 0.9), pre_open_interest="4980",
            average=str(last),
            bid1=str(bid), bid2=str(bid - 1), bid3=str(bid - 2), bid4=str(bid - 3), bid5=str(bid - 4),
            ask1=str(ask), ask2=str(ask + 1), ask3=str(ask + 2), ask4=str(ask + 3), ask5=str(ask + 4),
            bidvol1="10", bidvol2="8", bidvol3="6", bidvol4="4", bidvol5="2",
            askvol1="10", askvol2="8", askvol3="6", askvol4="4", askvol5="2",
        )
        rows.append(r)

    tick(RB, "SHFE", 3500, 3498, 3502)
    tick(JD, "DCE", 3000, 2998, 3002)
    write_canonical(path, rows)


def main() -> int:
    core = os.environ.get("CTPBUDDY_CORE")
    if not core:
        core = os.path.join(REPO, "core", "target", "task43", "debug", "ctpbuddy-server.exe")
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-order-sysid-")
    scenario = os.path.join(tmp, "scenario")
    os.makedirs(scenario)
    make_scenario(scenario)
    td, admin_port = free_port(), free_port()
    data = os.path.join(tmp, "data")
    proc = subprocess.Popen(
        [core, "--td", "127.0.0.1:%d" % td, "--admin", "127.0.0.1:%d" % admin_port,
         "--broker-id", BROKER, "--speed", "0", "--initial-funds", "2000000",
         "--qry-freq", "16", "--data-dir", data],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
    )
    try:
        wait_port(admin_port)
        admin = Admin("127.0.0.1:%d" % admin_port)
        started = admin.start_scenario(scenario, paused=False)
        assert started["ticks"] == 2, started
        time.sleep(0.15)
        wait_port(td)
        cli = Client("127.0.0.1:%d" % td)
        cli.auth(BROKER, "sysid01")
        login = cli.login(BROKER, "sysid01")
        cli.settle_confirm()
        cli.subscribe([RB, JD])
        drain(cli)

        # SHFE GFD：首条 a 为空，确认/撤单后非空；随后可用最终 sysid 撤单。
        cli.order_insert(RB, "0", "0", 1, 3490, exchange="SHFE", order_ref="G1")
        ev = drain(cli)
        g1 = orders(ev, "G1")
        assert len(g1) >= 2, g1
        assert g1[0]["OrderSysID"] == "", g1
        final_sys = g1[-1]["OrderSysID"]
        assert final_sys, g1
        assert chr(g1[0]["OrderStatus"]) == "a", g1
        assert chr(g1[-1]["OrderStatus"]) == "3", g1
        cli.order_action(RB, order_ref="", order_sys_id=final_sys, exchange="SHFE")
        cancel_ev = drain(cli)
        crows = orders(cancel_ev, "G1")
        assert crows and crows[-1]["OrderSysID"] == final_sys, crows
        assert chr(crows[-1]["OrderStatus"]) == "5", crows
        print("[ok] SHFE GFD: first a has empty sysid, accepted/cancel rows use %s" % final_sys)

        # 普通成交：Trade 只能带最终非空 OrderSysID，且 QryTrade/QryOrder 同键。
        cli.order_insert(RB, "0", "0", 1, 3502, exchange="SHFE", order_ref="T1")
        tev = drain(cli)
        trows = orders(tev, "T1")
        tf = trades(tev, "T1")
        assert trows[0]["OrderSysID"] == "", trows
        assert tf and tf[0]["OrderSysID"], tf
        trade_sys = tf[0]["OrderSysID"]
        assert trows[-1]["OrderSysID"] == trade_sys, (trows, tf)
        qtrade = [t for t in cli.qry_trade(RB) if t["OrderRef"] == "T1"]
        assert qtrade and qtrade[0]["OrderSysID"] == trade_sys, qtrade
        qorder = [o for o in cli.qry_order(RB) if o["OrderRef"] == "T1"]
        assert len(qorder) == 1 and qorder[0]["OrderSysID"] == trade_sys, qorder
        print("[ok] Trade/QryTrade/QryOrder use final sysid %s" % trade_sys)

        # 交易所层拒单：只有响应半面和错单回报，不生成任何 RTN_ORDER/sysid。
        cli.clear_late()
        cli.order_insert(RB, "0", "0", 1, 3500.5, exchange="SHFE", order_ref="R1")
        late = cli.wait_late(ERR_RTN_ORDER_INSERT)
        assert late is not None
        assert not orders(drain(cli), "R1")
        n = generated.SIZES["CThostFtdcInputOrderField"]
        assert generated.unpack("CThostFtdcInputOrderField", late.payload[:n])["OrderRef"] == "R1"
        print("[ok] exchange reject: no RTN_ORDER and no external sysid")

        # DCE FAK：布局确认行可先于成交，但仍不是“第2条绝对化”；Trade 最终非空。
        cli.order_insert(JD, "0", "0", 1, 3002, exchange="DCE", order_ref="D1",
                         time_condition="1", volume_condition="1")
        dev = drain(cli)
        drows, dtf = orders(dev, "D1"), trades(dev, "D1")
        assert drows and drows[0]["OrderSysID"] == "", drows
        assert dtf and dtf[0]["OrderSysID"], dtf
        assert drows[-1]["OrderSysID"] == dtf[0]["OrderSysID"], (drows, dtf)
        print("[ok] DCE/FAK layout: accepted boundary publishes final sysid")
        cli.close()
        print("\nM3 ORDER SYSID: PASS")
        return 0
    finally:
        try:
            Admin("127.0.0.1:%d" % admin_port).shutdown()
        except Exception:
            proc.terminate()
        try:
            out, _ = proc.communicate(timeout=5)
            print("---- server log ----")
            print(out.strip())
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.communicate()


if __name__ == "__main__":
    sys.exit(main())
