#!/usr/bin/env python
"""M2-1 end-to-end: the limit-order-book matching engine.

Runs the real Rust `ctpbuddy-server` against the real wire protocol with six
investor sessions and drives the whole M2-1 matching surface:

- order-vs-order matching at arrival: price priority, then time priority
  (fill price = the resting maker's limit price);
- the incoming order takes the better of the book and the current tick's
  five-level depth (on a price tie the tick depth wins: its volume queued
  before the just-parked order);
- FAK / FOK / FAK-with-MinVolume exact semantics under the official CTP
  encodings (TC_IOC='1'; VC_AV='1', VC_MV='2', VC_CV='3');
- self-trade prevention: resting orders of the same (broker, investor) never
  match each other;
- tick-driven matching of resting orders (`on_tick`);
- 成交开平归一化: SHFE keeps 平今 on the trade report, DCE reports 平仓
  (the order report still carries the requested 平今);
- the DESIGN §8.9 return sequence (initial 'a', rest confirmation as a single
  '3', 前态+新态 around every fill/cancel) and the frozen-funds release on
  the terminal '5'.

Usage:
    python tests/e2e/m2_book.py
Env:
    CTPBUDDY_CORE   explicit path to the ctpbuddy-server binary
"""
from __future__ import annotations

import json
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
RB = "rb2601"   # SHFE: mult 10, tick 1, margin 0.16
M = "jd2602"     # DCE:  mult 10, tick 1, margin 0.15
# six investors: A/B/C walk the rb2601 book, D the DCE offset normalization,
# E self-trade prevention, F tick-driven matching
INVESTORS = ["smoke001", "smoke002", "smoke003", "smoke004", "smoke005", "smoke006"]
INITIAL_FUNDS = 2_000_000.0

# rb2601 book economics used by the assertions below. The margin rate is the
# **company** rate from refdata/margin_rates.jsonl (notes/04 C3: the rate the
# counter charges is the one ReqQryInstrumentMarginRate returns), and the
# basis is 昨结算价 because the bundled MarginPriceType is '1' — so a parked
# order's frozen amount is PRE_SETTLE * mult * rate, independent of its
# limit price.
RB_MULT = 10
RB_MARGIN = 0.16
RB_PRE_SETTLE = 3500.0


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
    """Eight time-ordered ticks across rb2601 (SHFE) and jd2602 (DCE).

    Ticks 1-6 replay before the order flow (stepped while paused); ticks 7-8
    replay afterwards so a resting rb2601 sell is crossed by a later tick.
    """
    rows = []

    def tick(instrument, exchange, t, ms, last, bid1, bid1v, ask1, ask1v,
             upper, lower, bid2v="8", ask2v="10", pre_settle=None):
        row = {c: "" for c in CANONICAL_COLUMNS}
        row.update(
            instrument=instrument,
            exchange=exchange,
            trading_day="20261002",
            update_time=t,
            update_millisec=str(ms),
            last_price=str(last),
            volume="100",
            turnover="350000",
            open_interest="5000",
            # Every contract carries its **own** previous settlement, as a real
            # feed does. Hardcoding one value for both exchanges would leave
            # the DCE leg with a SHFE settlement price — and since 昨结算 is the
            # margin basis, that silently misprices the whole DCE position.
            pre_settlement=str(pre_settle if pre_settle is not None else last),
            settlement=str(pre_settle if pre_settle is not None else last),
            pre_close=str(bid1),
            open="3499",
            high="3505",
            low="3497",
            close="3501",
            upper=str(upper),
            lower=str(lower),
            pre_open_interest="4980",
            average="3500.25",
            bid1=str(bid1), bid2=str(bid1 - 1), bid3=str(bid1 - 2),
            bid4=str(bid1 - 3), bid5=str(bid1 - 4),
            ask1=str(ask1), ask2=str(ask1 + 1), ask3=str(ask1 + 2),
            ask4=str(ask1 + 3), ask5=str(ask1 + 4),
            bidvol1=str(bid1v), bidvol2=bid2v, bidvol3="6", bidvol4="4", bidvol5="2",
            askvol1=str(ask1v), askvol2=ask2v, askvol3="8", askvol4="6", askvol5="4",
        )
        rows.append(row)

    # the pre-flow market: rb bid1 3498x10 / ask1 3502x12, m bid1 3000 / ask1 3002
    for t, ms in (("09:30:00", 0), ("09:30:00", 500), ("09:31:00", 0)):
        tick(RB, "SHFE", t, ms, 3500, 3498, 10, 3502, 12, 3850, 3150)
        tick(M, "DCE", t, ms, 3000, 3000, 10, 3002, 12, 3300, 2700, pre_settle=2990)
    # post-flow: bid1 jumps to 3500 (crosses a resting 3499 sell), then calm
    tick(RB, "SHFE", "09:32:00", 0, 3499, 3500, 5, 3502, 12, 3850, 3150, bid2v="7")
    tick(RB, "SHFE", "09:33:00", 0, 3500, 3490, 5, 3510, 10, 3850, 3150)
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


def tids(evts, ref: str):
    return [f["TradeID"] for k, f in evts if k == RTN_TRADE and f["OrderRef"] == ref]


def rows(evts, ref: str):
    """Every RTN_ORDER dict for `ref`, in arrival order (statuses + volumes)."""
    return [f for k, f in evts if k == RTN_ORDER and f["OrderRef"] == ref]


def wait_idx(admin: Admin, n: int, timeout: float = 5.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if admin.status()["playback"]["idx"] >= n:
            return
        time.sleep(0.02)
    raise AssertionError("playback did not reach tick %d" % n)


class Feed:
    """Per-investor accumulation of push events across the whole flow.

    A drain consumes the queue, so asserting on a single step's slice would
    miss the earlier pushes of the same order; the feed keeps the full
    sequence per investor and the assertions read like the documented
    §8.9 return sequences.
    """

    def __init__(self, clients):
        self.clients = clients
        self.log = {name: [] for name in clients}

    def pump(self, *names):
        for name in names:
            self.log[name].extend(drain(self.clients[name]))

    def st(self, name, ref):
        return st(self.log[name], ref)

    def tr(self, name, ref):
        return tr(self.log[name], ref)

    def tids(self, name, ref):
        return tids(self.log[name], ref)

    def trades(self, name, ref):
        return [f for k, f in self.log[name] if k == RTN_TRADE and f["OrderRef"] == ref]

    def rows(self, name, ref):
        return rows(self.log[name], ref)


def main() -> int:
    core = find_core()
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-m2book-")
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
        run_book(td_port, admin_port, data_dir, scenario)
        print("\nM2 BOOK: PASS")
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


def run_book(td_port: int, admin_port: int, data_dir: str, scenario: str) -> None:
    # -- admin control plane ------------------------------------------------
    wait_port(admin_port)
    admin = Admin("127.0.0.1:%d" % admin_port)
    assert admin.ping()["cmd"] == "ping"
    stt = admin.status()
    # 7 builtin contracts: SHFE×3 / DCE / CFFEX / CZCE / GFEX — the last two
    # exist so e2e can drive all three official FAK report layouts (#43).
    assert stt["broker_id"] == BROKER and stt["instruments"] == 789, stt
    started = admin.start_scenario(scenario, paused=True)
    assert started["ticks"] == 8 and started["paused"] is True, started
    print("[ok] scenario loaded paused: %d ticks, day %s" % (started["ticks"], started["trading_day"]))

    # -- six investor sessions ----------------------------------------------
    wait_port(td_port)
    clients = {}
    try:
        for name in INVESTORS:
            cli = Client("127.0.0.1:%d" % td_port)
            cli.auth(BROKER, name)
            cli.login(BROKER, name, password="")
            cli.settle_confirm()
            clients[name] = cli
        for cli in clients.values():
            cli.subscribe([RB, M])
        print("[ok] %d investor sessions logged in, md subscribed" % len(clients))

        # replay the first six ticks one step at a time (paused playback)
        for i in range(6):
            admin.step()
            wait_idx(admin, i + 1)
        for cli in clients.values():
            drain(cli)  # market data only, no orders yet
        pb = admin.status()["playback"]
        assert pb["idx"] == 6 and pb["paused"] is True, pb
        print("[ok] 6/8 ticks replayed while paused (rb2601 + jd2602 depth live)")

        A, B, C, D, E, F = (clients[n] for n in INVESTORS)
        feed = Feed(clients)

        # -- 1) two resting sells, then a crossing buy ----------------------
        # price priority: A's 3500 fills before the tick's 3502 ask1;
        # time priority: A (arrived first) fills before B at the same price.
        A.order_insert(RB, direction="1", offset="0", volume=2, limit_price=3500.0,
                       exchange="SHFE", order_ref="A1")
        feed.pump("smoke001")
        assert feed.st("smoke001", "A1") == ["a", "3"], feed.log["smoke001"]
        assert feed.tr("smoke001", "A1") == [], feed.log["smoke001"]

        B.order_insert(RB, direction="1", offset="0", volume=2, limit_price=3500.0,
                       exchange="SHFE", order_ref="B1")
        feed.pump("smoke002")
        assert feed.st("smoke002", "B1") == ["a", "3"], feed.log["smoke002"]
        assert feed.tr("smoke002", "B1") == [], feed.log["smoke002"]

        C.order_insert(RB, direction="0", offset="0", volume=3, limit_price=3502.0,
                       exchange="SHFE", order_ref="C1")
        feed.pump("smoke001", "smoke002", "smoke003")
        # immediate fill: 'a', then 前态+新态 per fill, trade after the new state
        assert feed.st("smoke003", "C1") == ["a", "a", "1", "1", "0"], feed.log["smoke003"]
        assert feed.tr("smoke003", "C1") == [(2, 3500.0), (1, 3500.0)], feed.log["smoke003"]
        assert feed.st("smoke001", "A1") == ["a", "3", "3", "0"], feed.log["smoke001"]
        assert feed.tr("smoke001", "A1") == [(2, 3500.0)], feed.log["smoke001"]
        assert feed.st("smoke002", "B1") == ["a", "3", "3", "1"], feed.log["smoke002"]
        assert feed.tr("smoke002", "B1") == [(1, 3500.0)], feed.log["smoke002"]
        # maker and taker share one TradeID per book match
        assert feed.tids("smoke001", "A1")[0] == feed.tids("smoke003", "C1")[0]
        assert feed.tids("smoke002", "B1")[0] == feed.tids("smoke003", "C1")[1]
        print("[ok] book match: price+time priority, maker/taker dual trade reports")

        # -- 2) FOK: lookahead sees 1 book + 12 depth < 100 -> whole cancel --
        acct = C.qry_trading_account()
        assert close(acct["FrozenMargin"], 0.0, 1e-6), acct  # C1 fully filled
        C.order_insert(RB, direction="0", offset="0", volume=100, limit_price=3502.0,
                       exchange="SHFE", order_ref="C2", time_condition="1",
                       volume_condition="3")
        feed.pump("smoke003")
        assert feed.st("smoke003", "C2") == ["a", "a", "5"], feed.log["smoke003"]
        assert feed.tr("smoke003", "C2") == [], feed.log["smoke003"]
        acct = C.qry_trading_account()
        assert close(acct["FrozenMargin"], 0.0, 1e-6), acct   # 终态 '5' 解冻
        assert close(acct["FrozenCommission"], 0.0, 1e-6), acct
        print("[ok] FOK 100 lots: 13 tradable < 100 -> full cancel, freeze released")

        # -- 3) FAK (IOC + any volume): fill what crosses, cancel the rest ---
        # B still rests 1@3500, tick ask1 has 12@3502 -> 13 of 15 fill.
        # SHFE shape is 官方《报单回调规则》场景 8: the exchange pushes the
        # **cancel** report first (its VolumeTraded already carries 13), then
        # one '5' row per trade. No '3' (an IOC never rests) and no '1'.
        C.order_insert(RB, direction="0", offset="0", volume=15, limit_price=3502.0,
                       exchange="SHFE", order_ref="C3", time_condition="1",
                       volume_condition="1")
        feed.pump("smoke002", "smoke003")
        assert feed.st("smoke003", "C3") == ["a", "5", "5", "5"], feed.log["smoke003"]
        assert feed.tr("smoke003", "C3") == [(1, 3500.0), (12, 3502.0)], feed.log["smoke003"]
        c3 = feed.rows("smoke003", "C3")
        assert c3[1]["VolumeTraded"] == 13 and c3[1]["VolumeTotal"] == 2, c3[1]
        assert c3[2]["VolumeTraded"] == 1 and c3[2]["VolumeTotal"] == 14, c3[2]
        assert feed.st("smoke002", "B1") == ["a", "3", "3", "1", "1", "0"], feed.log["smoke002"]
        assert feed.tr("smoke002", "B1") == [(1, 3500.0), (1, 3500.0)], feed.log["smoke002"]
        assert feed.tids("smoke002", "B1")[1] == feed.tids("smoke003", "C3")[0]
        print("[ok] FAK 15 lots (SHFE 场景 8): cancel row first, then 1 '5' per trade")

        # -- 4) FAK with MinVolume: 12 tradable < 20 -> whole cancel ---------
        C.order_insert(RB, direction="0", offset="0", volume=10, limit_price=3502.0,
                       exchange="SHFE", order_ref="C4", time_condition="1",
                       volume_condition="2", min_volume=20)
        feed.pump("smoke003")
        assert feed.st("smoke003", "C4") == ["a", "a", "5"], feed.log["smoke003"]
        assert feed.tr("smoke003", "C4") == [], feed.log["smoke003"]
        acct = C.qry_trading_account()
        assert close(acct["FrozenMargin"], 0.0, 1e-6), acct
        print("[ok] FAK-MinVolume 10@3502 (min 20): 12 tradable < 20 -> full cancel")

        # -- 5) close-today on SHFE: the trade keeps 平今 --------------------
        C.order_insert(RB, direction="1", offset="3", volume=1, limit_price=3498.0,
                       exchange="SHFE", order_ref="C5")
        feed.pump("smoke003")
        assert feed.st("smoke003", "C5") == ["a", "a", "0"], feed.log["smoke003"]
        trades = feed.trades("smoke003", "C5")
        assert len(trades) == 1 and trades[0]["OffsetFlag"] == ord("3"), trades
        assert feed.tr("smoke003", "C5") == [(1, 3498.0)], feed.log["smoke003"]
        print("[ok] SHFE close-today: trade OffsetFlag stays '3' (平今)")

        # -- 6) DCE offset normalization on the trade report -----------------
        D.order_insert(M, direction="0", offset="0", volume=1, limit_price=3002.0,
                       exchange="DCE", order_ref="D1")
        feed.pump("smoke004")
        # 大商所特例 (notes/01 B3): DCE 对每一个进簿报单先返未成交确认 '3'——
        # 即使立即成交（IOC 类从不入簿、无此 '3'）；全部成交时 CTP 自补
        # 全部成交回报且不重复前态，故即时全成为 'a' → '3' → '0'
        assert feed.st("smoke004", "D1") == ["a", "3", "0"], feed.log["smoke004"]
        assert feed.tr("smoke004", "D1") == [(1, 3002.0)], feed.log["smoke004"]

        D.order_insert(M, direction="1", offset="3", volume=1, limit_price=3000.0,
                       exchange="DCE", order_ref="D2")
        feed.pump("smoke004")
        assert feed.st("smoke004", "D2") == ["a", "3", "0"], feed.log["smoke004"]
        trades = feed.trades("smoke004", "D2")
        assert len(trades) == 1, trades
        assert trades[0]["OffsetFlag"] == ord("1"), trades   # DCE: 平今 -> 平仓
        assert feed.tr("smoke004", "D2") == [(1, 3000.0)], feed.log["smoke004"]
        orders = [f for k, f in feed.log["smoke004"]
                  if k == RTN_ORDER and f["OrderRef"] == "D2"]
        assert orders and all(o["CombOffsetFlag"] == "3" for o in orders), orders
        print("[ok] DCE close-today: trade normalized to '1', order keeps 平今")

        # -- 7) self-trade prevention ----------------------------------------
        E.order_insert(RB, direction="1", offset="0", volume=1, limit_price=3500.0,
                       exchange="SHFE", order_ref="E1")
        feed.pump("smoke005")
        assert feed.st("smoke005", "E1") == ["a", "3"], feed.log["smoke005"]
        assert feed.tr("smoke005", "E1") == [], feed.log["smoke005"]
        E.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3501.0,
                       exchange="SHFE", order_ref="E2")
        feed.pump("smoke005")
        # 3501 crosses the resting 3500 ask but it is the same account; the
        # tick's ask1 (3502) does not cross -> the buy rests
        assert feed.st("smoke005", "E2") == ["a", "3"], feed.log["smoke005"]
        assert feed.tr("smoke005", "E2") == [], feed.log["smoke005"]
        acct = E.qry_trading_account()
        # Both estimates still frozen, one lot each. Deliberately NOT
        # 3500 + 3501: the two limit prices differ but the margin basis does
        # not move with them (昨结算), so the total is 2 x 3500 x 10 x 0.16.
        assert close(acct["FrozenMargin"], 2 * RB_PRE_SETTLE * RB_MULT * RB_MARGIN, 1e-4), acct
        print("[ok] self-trade prevention: same-account crossing skipped, both rest")

        for ref in ("E1", "E2"):
            E.order_action(RB, order_ref=ref)
            feed.pump("smoke005")
            assert feed.st("smoke005", ref) == ["a", "3", "3", "5"], feed.log["smoke005"]
            assert feed.tr("smoke005", ref) == [], feed.log["smoke005"]
        acct = E.qry_trading_account()
        assert close(acct["FrozenMargin"], 0.0, 1e-6), acct
        print("[ok] explicit cancels: 前态+'5' pairs, both freezes released")

        # -- 8) F rests 3499; the next tick's bid1 3500 crosses it -----------
        F.order_insert(RB, direction="1", offset="0", volume=1, limit_price=3499.0,
                       exchange="SHFE", order_ref="F1")
        feed.pump("smoke006")
        assert feed.st("smoke006", "F1") == ["a", "3"], feed.log["smoke006"]  # rests
        assert feed.tr("smoke006", "F1") == [], feed.log["smoke006"]

        admin.resume()
        wait_idx(admin, 8)
        feed.pump("smoke001", "smoke002", "smoke003", "smoke004", "smoke005", "smoke006")
        assert feed.st("smoke006", "F1") == ["a", "3", "3", "0"], feed.log["smoke006"]
        assert feed.tr("smoke006", "F1") == [(1, 3500.0)], feed.log["smoke006"]  # tick's bid
        print("[ok] tick-driven fill: resting 3499 sell matched by tick bid1 3500")

        # -- final state: accounts / positions / order + trade streams -------
        expect_account(A, "smoke001", margin=2 * 3500 * RB_MULT * RB_MARGIN, close_profit=0.0)
        expect_account(B, "smoke002", margin=2 * 3500 * RB_MULT * RB_MARGIN, close_profit=0.0)
        # C bought 16 lots (2+1+1+12) and closed 1, so 15 remain.
        # Occupancy is 15 x 昨结算 x mult x rate — one number regardless of the
        # four different fill prices, because `MarginPriceType == '1'` always
        # margined 今仓 against 昨结算 (notes/04 C2).
        #
        # The closed lot released the margin **its own detail was charged**
        # (a pro-rata slice of the 5600/lot booked at open), so the residue is
        # exactly 15 lots. The previous average-cost formula left a 2.4
        # residue; with per-lot details there is nothing left to round away.
        c_margin = 15 * RB_PRE_SETTLE * RB_MULT * RB_MARGIN
        # 先开先平 takes the oldest detail (C1's lot @3500), and it is 今仓, so
        # the basis is its own entry — not the 3501.5 average of all four fills.
        c_close_profit = (3498.0 - 3500.0) * RB_MULT
        expect_account(C, "smoke003", margin=c_margin, close_profit=c_close_profit)
        expect_account(D, "smoke004", margin=0.0, close_profit=-20.0)
        expect_account(E, "smoke005", margin=0.0, close_profit=0.0)
        # F opened 1 lot at 3499 and still holds it, so its margin is locked at
        # the basis in force when it was booked — 3499, not the 3500 every
        # other account shows. Margin does not re-price a position when a
        # later tick prints a different 昨结算.
        expect_account(F, "smoke006", margin=3499.0 * RB_MULT * RB_MARGIN, close_profit=0.0)
        print("[ok] accounts: margins + close profits + zero residual freezes")

        expect_position(A, RB, ord("3"), 2)   # short 2 today
        expect_position(B, RB, ord("3"), 2)
        expect_position(C, RB, ord("2"), 15)  # long 15 today (16 opened - 1 closed)
        expect_position(D, M, ord("2"), 0)    # flat after the round trip
        expect_position(E, RB, ord("2"), 0)   # cancelled before any fill
        expect_position(F, RB, ord("3"), 1)
        print("[ok] positions: today/yd split per investor")

        rows = {r["OrderRef"]: r for r in C.qry_order(RB)}
        assert set(rows) == {"C1", "C2", "C3", "C4", "C5"}, rows
        assert {ref: chr(r["OrderStatus"]) for ref, r in rows.items()} == {
            "C1": "0", "C2": "5", "C3": "5", "C4": "5", "C5": "0"}, rows
        assert rows["C3"]["VolumeTraded"] == 13 and rows["C3"]["VolumeTotal"] == 2, rows["C3"]
        assert len(A.qry_order(RB)) == 1 and len(B.qry_order(RB)) == 1, "A/B one order each"
        assert len(D.qry_order(M)) == 2 and len(E.qry_order(RB)) == 2, "D/E two orders each"
        assert len(F.qry_order(RB)) == 1, "F one order"
        print("[ok] qry_order: latest state per order (C3 partial 13/2, terminal '5's)")

        assert len(A.qry_trade(RB)) == 1 and len(B.qry_trade(RB)) == 2
        assert len(C.qry_trade(RB)) == 5 and len(D.qry_trade(M)) == 2
        assert len(E.qry_trade(RB)) == 0 and len(F.qry_trade(RB)) == 1
        prices = [round(t["Price"], 6) for t in C.qry_trade(RB)]
        assert prices == [3500.0, 3500.0, 3500.0, 3502.0, 3498.0], prices
        print("[ok] qry_trade streams: per-investor fill counts and prices")
    finally:
        for cli in clients.values():
            cli.close()

    # -- journal: the authoritative event stream ------------------------------
    admin.shutdown()
    time.sleep(0.2)
    journal_dir = os.path.join(data_dir, "journal")
    files = sorted(f for f in os.listdir(journal_dir) if f.endswith(".jsonl"))
    assert files, "no journal file written"
    events = []
    for name in files:
        with open(os.path.join(journal_dir, name), "r", encoding="utf-8") as f:
            for line in f:
                events.append(json.loads(line))
    types = [e["type"] for e in events]
    for want in ("session_auth", "order_insert", "order_update", "fill", "order_cancel"):
        assert want in types, (want, types)
    fills = [e for e in events if e["type"] == "fill"]
    # 4 (C1: 2 matches x maker+taker) + 3 (C3: 1 match + 1 market) + 1 (C5)
    # + 2 (D) + 1 (F tick) = 11 fill records
    assert len(fills) == 11, len(fills)
    per_investor = {}
    for e in fills:
        per_investor[e["investor"]] = per_investor.get(e["investor"], 0) + 1
    assert per_investor == {
        "smoke001": 1, "smoke002": 2, "smoke003": 5, "smoke004": 2, "smoke006": 1,
    }, per_investor
    cancels = [e for e in events if e["type"] == "order_cancel"]
    assert len(cancels) == 2, cancels
    assert {e["data"]["order_ref"] for e in cancels} == {"E1", "E2"}, cancels
    assert all(e["data"]["cancel_time"] for e in cancels), cancels
    # terminal '5' notifications: C2/C4 auto-cancels + C3's three 场景 8 rows
    # (撤单行 + 每笔成交一行) + E1/E2 explicit cancels
    fives = [e for e in events if e["type"] == "order_update" and e["data"]["status"] == "5"]
    assert len(fives) == 7, fives
    c3_fives = [e for e in fives if e["data"]["order_ref"] == "C3"]
    # 撤单行先到且已带成交量，之后每笔成交各一行（官方场景 8）
    assert [(e["data"]["volume_traded"], e["data"]["volume_total"]) for e in c3_fives] == [
        (13, 2), (1, 14), (13, 2)], c3_fives
    assert all(e.get("seq", 0) > 0 for e in events), "seq missing"
    print("[ok] journal: %d events, %d fills, %d cancels, %d terminal '5's"
          % (len(events), len(fills), len(cancels), len(fives)))


def expect_account(cli: Client, name: str, margin: float, close_profit: float) -> None:
    a = cli.qry_trading_account()
    assert close(a["CurrMargin"], margin, 1e-4), (name, a)
    assert close(a["CloseProfit"], close_profit, 1e-6), (name, a)
    assert close(a["FrozenMargin"], 0.0, 1e-6), (name, a)
    assert close(a["FrozenCommission"], 0.0, 1e-6), (name, a)


def expect_position(cli: Client, instrument: str, direction: int, volume: int) -> None:
    rows = cli.qry_investor_position(instrument)
    if volume == 0:
        assert rows == [] or all(r["Position"] == 0 for r in rows), rows
        return
    assert len(rows) == 1, rows
    p = rows[0]
    assert p["PosiDirection"] == direction, p
    assert p["Position"] == volume and p["TodayPosition"] == volume, p
    assert p["YdPosition"] == 0, p
    assert p["LongFrozen"] == 0 and p["ShortFrozen"] == 0, p


if __name__ == "__main__":
    sys.exit(main())
