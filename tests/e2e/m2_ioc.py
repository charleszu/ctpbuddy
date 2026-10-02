#!/usr/bin/env python
"""M2 FAK report layout per exchange group (issue #43).

官方《报单回调规则》(docs/api-doc-html/pages/389-QTYWGZ-DBHB.html) 测试场景
8/9/10 规定同���笔「FAK 部分成交部分撤单」在三个所族给出**三种不同**的回调顺序：

===========  ==========================================================
所族          回报顺序（省略开头的 OnRtnOrder 未知单）
===========  ==========================================================
SHFE/INE/CFFEX '5'（已撤单，VolumeTraded 已有值） → 每笔成交 '5' + OnRtnTrade
DCE/GFEX      '3'（未成交）→ 每笔成交 **一行** '1' + OnRtnTrade → '5'
CZCE          '3'（未成交）→ 每笔成交 **前态 + '1'** + OnRtnTrade → '5'
===========  ==========================================================

只有两个所族会推 '3' 进簿确认，只有 CZCE 会为每笔成交重复推前一状态，只有
SHFE 把撤单行放在成交之前。一个按「OnRtnOrder 条数」或「'1' 的条数」统计
的下游客户端，在这三组上会得到三个不同的答案——这就是本套件存在的理由。

m2_book.py 已在上期所覆盖场景 8（FAK 15 手部成部撤）；本文件补齐大商所（场景 9）
与郑商所（场景 10），并把三组并排断言，防止有人「顺手统一」掉这个差异。

Usage:
    python tests/e2e/m2_ioc.py
Env:
    CTPBUDDY_CORE   explicit path to the ctpbuddy-server binary
"""
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
from ctpbuddy.wire import RTN_ORDER, RTN_TRADE  # noqa: E402

BROKER = "8888"
# one instrument per official report group (see core catalog::builtin)
RB = "rb2610"   # SHFE  场景 8：撤单先行
M = "m2609"     # DCE   场景 9：单行合成 '1'
TA = "TA609"    # CZCE  场景 10：前态+新态
SI = "si2610"   # GFEX  与大商所同组
GROUPS = [("SHFE", RB), ("DCE", M), ("CZCE", TA), ("GFEX", SI)]

MAKER = "iocmaker"    # rests the sell the FAK taker hits
TAKER = "ioctaker"    # sends the FAKs
INITIAL_FUNDS = 2_000_000.0

# per-instrument depth: bid1/ask1 with a wide enough spread that one
# tick-multiple sits strictly between them, so the FAK taker beats the tick on
# price and hits the **book** (a price tie hands the fill to the tick, §8.4).
DEPTH = {
    RB: ("SHFE", 3500.0, 3400.0, 3600.0, 1.0),
    M: ("DCE", 3000.0, 2900.0, 3100.0, 1.0),
    TA: ("CZCE", 4000.0, 3800.0, 4200.0, 2.0),
    SI: ("GFEX", 5000.0, 4800.0, 5200.0, 5.0),
}
# the resting price and the FAK limit: the same value, one tick-multiple above
# bid1 and below ask1, so the taker beats the tick and still leaves a leftover.
MAKER_PRICE = {RB: 3501.0, M: 3001.0, TA: 4002.0, SI: 5005.0}


def find_core() -> str:
    env = os.environ.get("CTPBUDDY_CORE")
    if env:
        return env
    for prof in ("debug", "release"):
        for name in ("ctpbuddy-server", "ctpbuddy-server.exe"):
            cand = os.path.join(REPO, "core", "target", prof, name)
            if os.path.exists(cand):
                return cand
    raise SystemExit("ctpbuddy-server binary not found; run `cargo build` in core/ or set CTPBUDDY_CORE")


def free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


def wait_port(port: int, timeout: float = 10.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.5).close()
            return
        except OSError:
            time.sleep(0.05)
    raise SystemExit("server did not open port %d" % port)


def make_scenario(dirpath: str) -> None:
    rows = []

    def tick(instrument, exchange, last, bid1, bid1v, ask1, ask1v, upper, lower):
        row = {c: "" for c in CANONICAL_COLUMNS}
        row.update(
            instrument=instrument,
            exchange=exchange,
            trading_day="20261002",
            update_time="09:30:00",
            update_millisec="0",
            last_price=str(last),
            volume="100",
            turnover="350000",
            open_interest="5000",
            pre_settlement=str(last),
            settlement=str(last),
            pre_close=str(last),
            open=str(last),
            high=str(ask1),
            low=str(bid1),
            close=str(last),
            upper=str(upper),
            lower=str(lower),
            pre_open_interest="4980",
            average=str(last),
            bid1=str(bid1), bid2=str(bid1 - 1), bid3=str(bid1 - 2),
            bid4=str(bid1 - 3), bid5=str(bid1 - 4),
            ask1=str(ask1), ask2=str(ask1 + 1), ask3=str(ask1 + 2),
            ask4=str(ask1 + 3), ask5=str(ask1 + 4),
            bidvol1=str(bid1v), bidvol2="8", bidvol3="6", bidvol4="4", bidvol5="2",
            askvol1=str(ask1v), askvol2="10", askvol3="8", askvol4="6", askvol5="4",
        )
        rows.append(row)

    for instrument, (exchange, last, bid1, ask1, _tick) in DEPTH.items():
        # ask1 is deep enough that a FAK for more than it leaves a leftover
        # even if it also drains the book — the 「部成部撤」 shape.
        tick(instrument, exchange, last, bid1, 10, ask1, 12, bid1 * 2, ask1 / 2)
    write_canonical(dirpath, rows)


def close(a: float, b: float, tol: float = 1e-6) -> bool:
    return abs(a - b) <= tol


def drain(cli: Client, quiet: float = 0.35):
    """Collect push events until the queue stays empty for `quiet` seconds."""
    out = []
    while True:
        try:
            out.append(next(cli.events(timeout=quiet)))
        except StopIteration:
            break
        except queue.Empty:
            break
    return out


def st(evts, ref: str):
    """OrderStatus chars of every RTN_ORDER for `ref`, in arrival order."""
    return [chr(f["OrderStatus"]) for k, f in evts if k == RTN_ORDER and f["OrderRef"] == ref]


def tr(evts, ref: str):
    """(Volume, Price) of every RTN_TRADE for `ref`, in arrival order."""
    return [(f["Volume"], round(f["Price"], 6)) for k, f in evts if k == RTN_TRADE and f["OrderRef"] == ref]


def rows(evts, ref: str):
    """Every RTN_ORDER dict for `ref`, in arrival order."""
    return [f for k, f in evts if k == RTN_ORDER and f["OrderRef"] == ref]


def wait_idx(admin: Admin, n: int, timeout: float = 5.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if admin.status()["playback"]["idx"] >= n:
            return
        time.sleep(0.02)
    raise AssertionError("playback did not reach tick %d" % n)


def main() -> int:
    core = find_core()
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-m2ioc-")
    scenario = os.path.join(tmp, "scenario")
    os.makedirs(scenario)
    make_scenario(scenario)
    data_dir = os.path.join(tmp, "data")
    td_port, admin_port = free_port(), free_port()

    proc = subprocess.Popen(
        [
            core,
            "--td", "127.0.0.1:%d" % td_port,
            "--admin", "127.0.0.1:%d" % admin_port,
            "--broker-id", BROKER,
            "--speed", "0",
            "--initial-funds", str(INITIAL_FUNDS),
            "--qry-freq", "16",
            "--data-dir", data_dir,
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        run_ioc(td_port, admin_port, scenario)
        print("\nM2 IOC LAYOUT: PASS")
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
            out, _ = proc.communicate()
            print("---- server log (killed) ----")
            print(out.strip())


def run_ioc(td_port: int, admin_port: int, scenario: str) -> None:
    wait_port(admin_port)
    admin = Admin("127.0.0.1:%d" % admin_port)
    assert admin.ping()["cmd"] == "ping"
    stt = admin.status()
    # SHFE×3 / DCE / CFFEX / CZCE / GFEX — one per report group, plus the
    # extras the builtin catalog carries.
    assert stt["broker_id"] == BROKER and stt["instruments"] == 7, stt
    started = admin.start_scenario(scenario, paused=False)
    assert started["ticks"] == 4, started
    print("[ok] scenario loaded: %d instruments, day %s" % (started["ticks"], started["trading_day"]))
    wait_idx(admin, 4)

    wait_port(td_port)
    clients = {}
    try:
        for name in (MAKER, TAKER):
            cli = Client("127.0.0.1:%d" % td_port)
            cli.auth(BROKER, name)
            cli.login(BROKER, name, password="")
            cli.settle_confirm()
            clients[name] = cli
        for cli in clients.values():
            cli.subscribe([i for _, i in GROUPS])
        drain(clients[MAKER])
        drain(clients[TAKER])
        print("[ok] 2 sessions logged in, %d instruments subscribed" % len(GROUPS))

        shapes = {}
        for idx, (exchange, instrument) in enumerate(GROUPS):
            price = MAKER_PRICE[instrument]
            # the maker rests 1 lot; the taker FAKs for 10 -> 部成部撤 on all groups
            clients[MAKER].order_insert(
                instrument, direction="1", offset="0", volume=1, limit_price=price,
                exchange=exchange, order_ref="MK%d" % idx)
            maker_log = drain(clients[MAKER])
            assert st(maker_log, "MK%d" % idx) == ["a", "3"], maker_log

            clients[TAKER].order_insert(
                instrument, direction="0", offset="0", volume=10, limit_price=price,
                exchange=exchange, order_ref="TK%d" % idx,
                time_condition="1", volume_condition="1")
            taker_log = drain(clients[TAKER])
            ref = "TK%d" % idx
            trades = tr(taker_log, ref)
            assert trades == [(1, price)], (exchange, trades)
            shapes[exchange] = st(taker_log, ref)
            print("[ok] %-4s FAK 10@%s: statuses %s, 1 fill @ %s"
                  % (exchange, price, "".join(shapes[exchange]), price))

        # 官方场景 8：撤单状态回报先于成交回报，每笔成交只有一行 '5'，无 '3'、无 '1'
        assert shapes["SHFE"] == ["a", "5", "5"], shapes["SHFE"]
        # 官方场景 9：大商所先给 '3' 进簿确认，每笔成交只推**一行**合成的 '1'
        assert shapes["DCE"] == ["a", "3", "1", "5"], shapes["DCE"]
        # 官方场景 10：郑商所先给 '3' 进簿确认，成交回报走一般的前态+新态
        assert shapes["CZCE"] == ["a", "3", "3", "1", "5"], shapes["CZCE"]
        # 广期所与大商所同组（场景 9 原文：「相同场景下，广期所FAK的回报与大商所一致」）
        assert shapes["GFEX"] == shapes["DCE"], shapes
        print("[ok] 场景 8/9/10 shapes reproduced per exchange group")

        # the three groups must NOT collapse into one shape — that is the bug
        # this suite exists to prevent.
        assert len({tuple(v) for v in shapes.values()}) == 3, shapes
        assert shapes["GFEX"] == shapes["DCE"], "GFEX follows DCE"
        print("[ok] three distinct layouts (SHFE / DCE+GFEX / CZCE)")

        # every FAK ended terminal '5' with the traded volume recorded once
        for idx, (exchange, instrument) in enumerate(GROUPS):
            got = clients[TAKER].qry_order(instrument)
            mine = [o for o in got if o["OrderRef"] == "TK%d" % idx]
            assert len(mine) == 1, (exchange, got)
            o = mine[0]
            assert chr(o["OrderStatus"]) == "5", (exchange, o)
            assert o["VolumeTraded"] == 1 and o["VolumeTotal"] == 9, (exchange, o)
        print("[ok] qry_order: each FAK terminal '5' with 1 traded / 9 left")
    finally:
        for cli in clients.values():
            cli.close()


if __name__ == "__main__":
    sys.exit(main())
