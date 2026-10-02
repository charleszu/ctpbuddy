#!/usr/bin/env python
"""M2-4 end-to-end: 报单流控规则表 + 订单状态机与回报时序.

Runs the real Rust `ctpbuddy-server` against the real wire protocol in two
phases, each with its own server + data dir + journal:

Phase 1 (``--order-freq 2``) — the front-office per-(broker, investor)
insert+cancel budget (DESIGN §8.3, LK 流控文档):
- the third of three back-to-back inserts is rejected with the official
  ``116 ORDER_FREQ_LIMIT`` "CTP:下单频率限制";
- insert and cancel share the same per-second budget;
- the budget is keyed per (broker, investor): one investor's exhausted
  window never blocks another's;
- the wall-clock window recovers after ~1s;
- the journal records both rejections with the instruction-level
  OrderSubmitStatus ('4' 报单已被拒绝 / '5' 撤单已被拒绝) and outcome.

Phase 2 (high budget) — the OSS state machine + §8.9 return sequences:
- SHFE GFD park 'a','3' -> explicit cancel 'a','3','3','5', with the
  QryOrder projection showing OrderSubmitStatus '3' (已经接受);
- SHFE immediate full fill 'a','a','0';
- DCE immediate full fill 'a','3','0' (报单确认 '3' + CTP 自补 '0', no
  前态 repeat);
- DCE partial -> full 'a','3','3','1','0' (the maker side);
- exchange="" backfill from the catalog keeps the DCE special case;
- DCE tick-driven fill 'a','3','0'.

The assertions check ErrorID + ErrorMsg but deliberately not the push
surface (kind): the insert-rejection / cancel-rejection push faces are
still being reconciled against the official 报单回调规则 (task #42) and
may move between OnRspOrderInsert and OnErrRtnOrderInsert.

Usage:
    python tests/e2e/m2_flow.py
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

from ctpbuddy.sdk import Admin, CTPError, Client  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402
from ctpbuddy.wire import RTN_ORDER, RTN_TRADE  # noqa: E402

BROKER = "8888"
RB = "rb2601"   # SHFE: mult 10, tick 1, margin 0.16
M = "jd2602"     # DCE:  mult 10, tick 1, margin 0.15
INITIAL_FUNDS = 2_000_000.0

ERR_FREQ = 116            # ORDER_FREQ_LIMIT
FREQ_MSG = "CTP:下单频率限制"

# Phase-1 investors (freq gate); Phase-2 investors walk the state machine.
P1 = ["freq001", "freq002"]
P2 = ["flow001", "flow002", "flow003", "flow004", "flow005", "flow006", "flow007", "flow008"]


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
    """Four time-ordered ticks: three pre-flow (rb2601 + jd2602 depth live),
    one post-flow where the jd2602 bid1 3002 crosses a resting 3001 sell.
    """
    rows = []

    def tick(instrument, exchange, t, ms, last, bid1, bid1v, ask1, ask1v,
             upper, lower, bid2v="8", ask2v="10"):
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
            pre_settlement="3500",
            settlement="3500",
            pre_close="3498",
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

    # pre-flow: rb bid1 3498x10 / ask1 3502x12, m bid1 3000x10 / ask1 3002x12
    for t, ms in (("09:30:00", 0), ("09:30:00", 500), ("09:31:00", 0)):
        tick(RB, "SHFE", t, ms, 3500, 3498, 10, 3502, 12, 3850, 3150)
        tick(M, "DCE", t, ms, 3000, 3000, 10, 3002, 12, 3300, 2700)
    # post-flow: the jd2602 bid jumps to 3002 (crosses a resting 3001 sell)
    tick(M, "DCE", "09:32:00", 0, 3002, 3002, 5, 3003, 8, 3300, 2700)
    write_canonical(dirpath, rows)


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


def wait_idx(admin: Admin, n: int, timeout: float = 5.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if admin.status()["playback"]["idx"] >= n:
            return
        time.sleep(0.02)
    raise AssertionError("playback did not reach tick %d" % n)


def read_journal(data_dir: str):
    journal_dir = os.path.join(data_dir, "journal")
    files = sorted(f for f in os.listdir(journal_dir) if f.endswith(".jsonl"))
    assert files, "no journal file written"
    events = []
    for name in files:
        with open(os.path.join(journal_dir, name), "r", encoding="utf-8") as f:
            for line in f:
                events.append(json.loads(line))
    return events


def st(evts, ref: str):
    """OrderStatus chars of every RTN_ORDER for `ref`, in arrival order."""
    return [chr(f["OrderStatus"]) for k, f in evts if k == RTN_ORDER and f["OrderRef"] == ref]


def tr(evts, ref: str):
    """(Volume, Price) of every RTN_TRADE for `ref`, in arrival order."""
    return [(f["Volume"], round(f["Price"], 6)) for k, f in evts if k == RTN_TRADE and f["OrderRef"] == ref]


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


class Server:
    """One ctpbuddy-server process + its admin plane + data dir."""

    def __init__(self, core: str, root: str, name: str, order_freq: int):
        self.core = core
        self.name = name
        self.data_dir = os.path.join(root, "data-" + name)
        self.td_port, self.admin_port = free_port(), free_port()
        self.proc = subprocess.Popen(
            [
                core,
                "--td", "127.0.0.1:%d" % self.td_port,
                "--admin", "127.0.0.1:%d" % self.admin_port,
                "--broker-id", BROKER,
                "--speed", "0",
                "--initial-funds", str(INITIAL_FUNDS),
                "--qry-freq", "16",
                "--order-freq", str(order_freq),
                "--data-dir", self.data_dir,
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )

    def admin(self) -> Admin:
        wait_port(self.admin_port)
        return Admin("127.0.0.1:%d" % self.admin_port)

    def shutdown(self) -> None:
        try:
            Admin("127.0.0.1:%d" % self.admin_port).shutdown()
        except Exception:
            self.proc.terminate()
        try:
            out, _ = self.proc.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            out, _ = self.proc.communicate()
        print("---- %s server log ----" % self.name)
        print(out.strip())


def login_all(td_port: int, investors) -> dict:
    wait_port(td_port)
    clients = {}
    try:
        for name in investors:
            cli = Client("127.0.0.1:%d" % td_port)
            cli.auth(BROKER, name)
            cli.login(BROKER, name, password="")
            cli.settle_confirm()
            clients[name] = cli
    except Exception:
        for cli in clients.values():
            cli.close()
        raise
    return clients


def phase1(core: str, root: str, scenario: str) -> None:
    """报单流控：--order-freq 2 下的插入/撤单共享预算与恢复."""
    srv = Server(core, root, "freq", order_freq=2)
    try:
        admin = srv.admin()
        assert admin.ping()["cmd"] == "ping"
        started = admin.start_scenario(scenario, paused=True)
        assert started["paused"] is True, started
        clients = login_all(srv.td_port, P1)
        A, B = clients["freq001"], clients["freq002"]
        try:
            # (a) insert side: three in a row — the third is over budget.
            # A resting buy parks (3497 is below every ask), no market data
            # needed while playback is paused.
            A.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3497.0,
                           exchange="SHFE", order_ref="F1")
            A.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3497.0,
                           exchange="SHFE", order_ref="F2")
            try:
                A.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3497.0,
                               exchange="SHFE", order_ref="F3")
                raise AssertionError("third insert within budget accepted")
            except CTPError as e:
                assert e.error_id == ERR_FREQ, e
                assert FREQ_MSG in e.msg, e
            print("[ok] 3 back-to-back inserts: the 3rd rejected (ErrorID %d, %s)"
                  % (ERR_FREQ, FREQ_MSG))

            # (b) wall-clock recovery: past the one-second window the budget
            # is back.
            time.sleep(1.05)
            A.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3497.0,
                           exchange="SHFE", order_ref="F4")
            print("[ok] after ~1s the per-(broker,investor) window recovers")

            # (c) insert + cancel share the budget: on a fresh window 1 insert
            # + 1 cancel spend it, the next insert is rejected.
            time.sleep(1.05)
            A.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3497.0,
                           exchange="SHFE", order_ref="F5")
            A.order_action(RB, order_ref="F5")
            try:
                A.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3497.0,
                               exchange="SHFE", order_ref="F6")
                raise AssertionError("insert after insert+cancel accepted")
            except CTPError as e:
                assert e.error_id == ERR_FREQ, e
                assert FREQ_MSG in e.msg, e
            print("[ok] cancel spends the same per-second budget as insert")

            # (d) cancel side: two inserts + one cancel — the cancel is the
            # over-budget request and is rejected (文档口径 OnRspOrderAction).
            time.sleep(1.05)
            A.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3497.0,
                           exchange="SHFE", order_ref="F7")
            A.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3497.0,
                           exchange="SHFE", order_ref="F8")
            try:
                A.order_action(RB, order_ref="F7")
                raise AssertionError("over-budget cancel accepted")
            except CTPError as e:
                assert e.error_id == ERR_FREQ, e
                assert FREQ_MSG in e.msg, e
            print("[ok] over-budget cancel rejected the same way (ErrorID %d)" % ERR_FREQ)

            # (e) per-investor isolation: B's window is intact while A's is
            # exhausted.
            B.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3497.0,
                           exchange="SHFE", order_ref="G1")
            print("[ok] the budget is keyed per (broker, investor): B unaffected")

            # (f) recovery once more, so the flow ends clean.
            time.sleep(1.05)
            A.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3497.0,
                           exchange="SHFE", order_ref="F9")
        finally:
            for cli in clients.values():
                cli.close()
        admin.shutdown()
        time.sleep(0.2)

        # journal: both rejections carry the instruction-level submit status
        events = read_journal(srv.data_dir)
        rej_inserts = [
            e for e in events
            if e["type"] == "order_insert" and e["data"]["submit_status"] == "4"
        ]
        rej_cancels = [
            e for e in events
            if e["type"] == "order_cancel" and e["data"]["submit_status"] == "5"
        ]
        # F3 (a) and F6 (c) rejected inserts; F7's cancel (d) rejected
        assert len(rej_inserts) == 2, rej_inserts
        assert len(rej_cancels) == 1, rej_cancels
        assert {e["data"]["outcome"]["error_id"] for e in rej_inserts} == {ERR_FREQ}
        assert {e["data"]["outcome"]["error_id"] for e in rej_cancels} == {ERR_FREQ}
        assert all(e["data"]["outcome"]["accepted"] is False for e in rej_inserts + rej_cancels)
        assert {e["data"]["order_ref"] for e in rej_inserts} == {"F3", "F6"}, rej_inserts
        assert rej_cancels[0]["data"]["order_ref"] == "F7", rej_cancels
        # every accepted insert keeps the '0' 报单已提交 status
        ok_inserts = [
            e for e in events
            if e["type"] == "order_insert" and e["data"]["submit_status"] != "4"
        ]
        assert len(ok_inserts) == 8, ok_inserts  # F1 F2 F4 F5 F7 F8 F9 + G1 (investor 2)
        assert all(e["data"]["outcome"]["accepted"] is True for e in ok_inserts)
        print("[ok] journal: %d accepted + %d rejected inserts, %d rejected cancels, "
              "submit_status '4'/'5' + outcome recorded"
              % (len(ok_inserts), len(rej_inserts), len(rej_cancels)))
    finally:
        srv.shutdown()


def phase2(core: str, root: str, scenario: str) -> None:
    """订单状态机：OSS 七态 + §8.9 回报时序（含大商所特例）."""
    srv = Server(core, root, "flow", order_freq=1000)
    try:
        admin = srv.admin()
        assert admin.ping()["cmd"] == "ping"
        started = admin.start_scenario(scenario, paused=True)
        assert started["ticks"] == 7 and started["paused"] is True, started
        clients = login_all(srv.td_port, P2)
        try:
            # replay the six pre-flow rows (3 times x rb/m) so both depth
            # books are live
            for i in range(6):
                admin.step()
                wait_idx(admin, i + 1)
            A, B, C, D, E, F, G, H = (clients[n] for n in P2)
            feed = Feed(clients)
            feed.pump(*P2)  # market data only, no orders yet

            def st(name, ref):
                return feed.st(name, ref)

            def trades(name, ref):
                return feed.tr(name, ref)

            # -- 1) SHFE GFD park -> explicit cancel -------------------------
            A.order_insert(RB, direction="1", offset="0", volume=1, limit_price=3503.0,
                           exchange="SHFE", order_ref="A1")
            feed.pump("flow001")
            assert st("flow001", "A1") == ["a", "3"], feed.log["flow001"]
            # OSS: the initial push is '0' 报单已提交, every later row '3' 已经接受
            oss = [chr(f["OrderSubmitStatus"]) for k, f in feed.log["flow001"]
                   if k == RTN_ORDER and f["OrderRef"] == "A1"]
            assert oss == ["0", "3"], feed.log["flow001"]
            row = [r for r in A.qry_order(RB) if r["OrderRef"] == "A1"]
            assert len(row) == 1, row
            assert chr(row[0]["OrderStatus"]) == "3", row
            assert chr(row[0]["OrderSubmitStatus"]) == "3", row   # 已经接受
            A.order_action(RB, order_ref="A1")
            feed.pump("flow001")
            assert st("flow001", "A1") == ["a", "3", "3", "5"], feed.log["flow001"]
            row = [r for r in A.qry_order(RB) if r["OrderRef"] == "A1"][0]
            assert chr(row["OrderStatus"]) == "5" and chr(row["OrderSubmitStatus"]) == "3", row
            print("[ok] SHFE GFD: park 'a','3' -> cancel 'a','3','3','5'; QryOrder OSS '3'")

            # -- 2) SHFE immediate full fill ---------------------------------
            B.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3502.0,
                           exchange="SHFE", order_ref="B1")
            feed.pump("flow002")
            assert st("flow002", "B1") == ["a", "a", "0"], feed.log["flow002"]
            assert trades("flow002", "B1") == [(1, 3502.0)], feed.log["flow002"]
            print("[ok] SHFE immediate full fill: 'a','a','0' (crosses the tick ask1)")

            # -- 3) DCE immediate full fill ----------------------------------
            C.order_insert(M, direction="0", offset="0", volume=1, limit_price=3002.0,
                           exchange="DCE", order_ref="C1")
            feed.pump("flow003")
            # '3' 报单确认 + CTP 自补 '0'（不重复前态）
            assert st("flow003", "C1") == ["a", "3", "0"], feed.log["flow003"]
            assert trades("flow003", "C1") == [(1, 3002.0)], feed.log["flow003"]
            print("[ok] DCE immediate full fill: 'a','3','0' (no 前态 on the self-completed '0')")

            # -- 4) DCE partial -> full (maker side) --------------------------
            # D rests 3@3001 — strictly better than the tick ask1 3002, so
            # the takers match the book, not the snapshot depth
            D.order_insert(M, direction="1", offset="0", volume=3, limit_price=3001.0,
                           exchange="DCE", order_ref="D1")
            feed.pump("flow004")
            assert st("flow004", "D1") == ["a", "3"], feed.log["flow004"]  # rests
            E.order_insert(M, direction="0", offset="0", volume=2, limit_price=3002.0,
                           exchange="DCE", order_ref="E1")
            F.order_insert(M, direction="0", offset="0", volume=1, limit_price=3002.0,
                           exchange="DCE", order_ref="F1")
            feed.pump("flow004", "flow005", "flow006")
            # partial '3'->'1' keeps the 前态 pair; the completing fill is
            # CTP-self-completed: no repeated 前态 '1'
            assert st("flow004", "D1") == ["a", "3", "3", "1", "0"], feed.log["flow004"]
            assert trades("flow004", "D1") == [(2, 3001.0), (1, 3001.0)], feed.log["flow004"]
            for name, ref, vol in (("flow005", "E1", 2), ("flow006", "F1", 1)):
                assert st(name, ref) == ["a", "3", "0"], feed.log[name]
                assert trades(name, ref) == [(vol, 3001.0)], feed.log[name]
            print("[ok] DCE partial->full (maker): 'a','3','3','1','0'; takers 'a','3','0'")

            # -- 5) exchange="" backfill keeps the DCE special case ----------
            G.order_insert(M, direction="0", offset="0", volume=1, limit_price=3002.0,
                           exchange="", order_ref="G1")
            feed.pump("flow007")
            assert st("flow007", "G1") == ["a", "3", "0"], feed.log["flow007"]
            assert trades("flow007", "G1") == [(1, 3002.0)], feed.log["flow007"]
            print("[ok] exchange=\"\" backfilled from the catalog: DCE rules still apply")

            # -- 6) DCE tick-driven fill -------------------------------------
            # 3001 sits between the pre-flow bid1 3000 and ask1 3002: it
            # rests until the post-flow tick lifts the bid to 3002
            H.order_insert(M, direction="1", offset="0", volume=1, limit_price=3001.0,
                           exchange="DCE", order_ref="H1")
            feed.pump("flow008")
            assert st("flow008", "H1") == ["a", "3"], feed.log["flow008"]
            admin.resume()
            wait_idx(admin, 7)
            feed.pump("flow008")
            assert st("flow008", "H1") == ["a", "3", "0"], feed.log["flow008"]
            # the tick bid1 3002 crosses the resting 3001 sell
            assert trades("flow008", "H1") == [(1, 3002.0)], feed.log["flow008"]
            print("[ok] DCE tick-driven fill: 'a','3','0' at the tick bid1 3002")
        finally:
            for cli in clients.values():
                cli.close()
        admin.shutdown()
        time.sleep(0.2)

        events = read_journal(srv.data_dir)
        types = [e["type"] for e in events]
        for want in ("session_auth", "order_insert", "order_update", "fill", "order_cancel"):
            assert want in types, (want, types)
        cancels = [e for e in events if e["type"] == "order_cancel"]
        assert len(cancels) == 1 and cancels[0]["data"]["order_ref"] == "A1", cancels
        assert cancels[0]["data"]["submit_status"] == "3" and \
            cancels[0]["data"]["outcome"]["accepted"] is True, cancels
        # every accepted insert journals '0' 报单已提交 — no rejections here
        inserts = [e for e in events if e["type"] == "order_insert"]
        assert len(inserts) == 8, inserts  # A1 B1 C1 D1 E1 F1 G1 H1
        assert all(e["data"]["submit_status"] == "0" for e in inserts), inserts
        assert all(e["data"]["outcome"]["accepted"] is True for e in inserts), inserts
        fives = [e for e in events if e["type"] == "order_update" and e["data"]["status"] == "5"]
        assert len(fives) == 1, fives   # only A1 reached '5'
        fills = [e for e in events if e["type"] == "fill"]
        # B1 C1 (market) + D1/E1, D1/F1 (book matches: maker+taker each)
        # + G1 (market) + H1 (tick) = 8 records
        assert len(fills) == 8, fills
        print("[ok] journal: %d inserts (all submit_status '0'), %d fills, 1 cancel"
              % (len(inserts), len(fills)))
    finally:
        srv.shutdown()


def main() -> int:
    core = find_core()
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-m2flow-")
    scenario = os.path.join(tmp, "scenario")
    os.makedirs(scenario)
    make_scenario(scenario)
    phase1(core, tmp, scenario)
    phase2(core, tmp, scenario)
    print("\nM2 FLOW: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
