#!/usr/bin/env python
"""M2-3 end-to-end: journal recording, deterministic hashing and replay.

Runs the real Rust `ctpbuddy-server` against the real wire protocol:

- **record**: a scripted, fully deterministic flow (paused scenario stepped
  tick by tick, requests injected between steps): three investor sessions,
  an immediate fill, a resting order filled by a later gap tick, a resting
  order cancelled, a rejected order, and the scenario's one-shot
  assertions (incl. the designed failure);
- **double run**: the SAME driver run twice must produce identical
  full-stream hashes — the M2-3 exit criterion "同一场景跑两次输出 hash 一致";
- **replay**: the recording is re-driven into a fresh core (`journal replay`
  semantics: connections in first-appearance order, requests at their
  recorded vt, ticks released per the recorded watermark trace) and the
  replay journal's core hash must equal the recording's;
- **mutation (negative control)**: a recorded limit price is tampered with;
  the replay must then diverge (core hash differs, replay reports FAIL);
- the CLI journal hash/show/verify surface over the same recording.

Usage:
    python tests/e2e/m2_journal.py
Env:
    CTPBUDDY_CORE   explicit path to the ctpbuddy-server binary
"""
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

from ctpbuddy.cli import main as cli_main  # noqa: E402
from ctpbuddy.journal import count_types, fmt_vt, hash_stream, load_events  # noqa: E402
from ctpbuddy.replay import ReplayError, format_result, replay_journal  # noqa: E402
from ctpbuddy.scenario import compile_scenario, load_scenario_spec  # noqa: E402
from ctpbuddy.sdk import Admin, Client  # noqa: E402
from ctpbuddy.sdk.client import rsp_info_of  # noqa: E402
from ctpbuddy.wire import ERR_RTN_ORDER_INSERT, RTN_DEPTH_MD, RTN_ORDER, RTN_TRADE  # noqa: E402

BROKER = "8888"
RB = "rb2601"  # SHFE: mult 10, tick 1, margin 0.16, limits 3150..3850
SCENARIO_NAME = "dsl-pipeline-demo"
TOTAL_TICKS = 11  # 12 written ticks, one dropped by the freeze
DEFAULT_FUNDS = 1_000_000.0
D1, D2, LEGACY = "dsl001", "dsl002", "legacy001"


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


def connect_admin(addr: str, timeout: float = 10.0) -> Admin:
    """Retry-connect the real admin session until the port is listening.

    Deliberately NOT a transient probe: whether the core accepts a
    connect-then-close probe before the real session is a race, and every
    consumed connection id shifts the recorded front_ids (the journal keeps
    them in session_auth — the double-run hash exit criterion requires them
    to be a pure function of the drive).
    """
    deadline = time.time() + timeout
    while True:
        try:
            return Admin(addr)
        except OSError:
            if time.time() >= deadline:
                raise
            time.sleep(0.05)


def connect_client(addr: str, timeout: float = 10.0) -> Client:
    """Retry-connect a real client session (same rationale as connect_admin)."""
    deadline = time.time() + timeout
    while True:
        try:
            return Client(addr)
        except OSError:
            if time.time() >= deadline:
                raise
            time.sleep(0.05)


def drain(cli: Client, quiet: float = 0.2):
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
    return [chr(f["OrderStatus"]) for k, f in evts if k == RTN_ORDER and f["OrderRef"] == ref]


def tr(evts, ref: str):
    return [(f["Volume"], round(f["Price"], 6)) for k, f in evts if k == RTN_TRADE and f["OrderRef"] == ref]


def wait_idx(admin: Admin, n: int, timeout: float = 5.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if admin.status()["playback"]["idx"] >= n:
            return
        time.sleep(0.02)
    raise AssertionError("playback did not reach tick %d" % n)


def close(a: float, b: float, tol: float = 1e-6) -> bool:
    return abs(a - b) <= tol


def prepare_scenario(workdir: str) -> str:
    """Copy the committed dsl_demo, force a fresh compile, sanity-check it."""
    scenario = os.path.join(workdir, "scenario")
    shutil.copytree(os.path.join(REPO, "scenarios", "dsl_demo"), scenario)
    os.remove(os.path.join(scenario, "scenario.json"))  # the repo json is a cache
    jpath = compile_scenario(scenario)
    assert jpath == os.path.join(scenario, "scenario.json"), jpath
    spec = load_scenario_spec(scenario)

    # the committed artifact must equal a fresh compile (staleness guard)
    with open(os.path.join(REPO, "scenarios", "dsl_demo", "scenario.json"), "r", encoding="utf-8") as f:
        assert json.load(f) == spec, "scenarios/dsl_demo/scenario.json is stale — recompile it"

    assert spec["name"] == SCENARIO_NAME, spec
    assert [t["kind"] for t in spec["transforms"]] == ["freeze", "liquidity", "gap"], spec
    assert [(a["investor"], a["balance"]) for a in spec["accounts"]] == [
        (D1, 800000.0), (D2, 400000.0)], spec
    assert len(spec["assertions"]) == 4, spec
    assert cli_main(["scenario", "validate", scenario]) == 0
    return scenario


def journal_file(journal_dir: str) -> str:
    files = sorted(f for f in os.listdir(journal_dir) if f.endswith(".jsonl"))
    assert files, "没有 journal 文件"
    # 启动墙钟日和回放交易日可能不同；不能假设只有一个文件。
    return journal_dir


def record_run(core: str, scenario: str, rundir: str) -> tuple[str, int]:
    """Drive the deterministic scripted flow once.

    Every request is injected between paused steps, so the world-loop
    order — and therefore the journal — is a pure function of the script.
    Returns ``(journal_dir, d1r_error)``: the rejected order's ErrorID as
    observed by the client, cross-checked against the journaled outcome.
    """
    data_dir = os.path.join(rundir, "data")
    d1r_error = 0
    td_port, admin_port = free_port(), free_port()
    proc = subprocess.Popen(
        [
            core,
            "--td", "127.0.0.1:%d" % td_port,
            "--admin", "127.0.0.1:%d" % admin_port,
            "--broker-id", BROKER,
            "--speed", "0",
            "--initial-funds", str(DEFAULT_FUNDS),
            "--data-dir", data_dir,
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        admin = connect_admin("127.0.0.1:%d" % admin_port)
        assert admin.ping()["cmd"] == "ping"
        # load the scenario paused from the very start: the stepped drive
        # below owns every tick release
        started = admin.start_scenario(
            scenario, paused=True, spec=load_scenario_spec(scenario))
        assert started["name"] == SCENARIO_NAME and started["ticks"] == TOTAL_TICKS, started
        assert started["paused"] is True and started["assertions_passed"] == 0, started

        clients = {}
        try:
            for name in (D1, D2, LEGACY):
                cli = connect_client("127.0.0.1:%d" % td_port)
                cli.auth(BROKER, name)
                cli.login(BROKER, name, password="")
                cli.settle_confirm()
                cli.subscribe([RB])
                clients[name] = cli
            for cli in clients.values():
                drain(cli)

            def step(n: int):
                admin.step()
                wait_idx(admin, n)

            # 09:30:00 / 09:30:30 raw
            step(1); step(2)
            # 09:31:00: liquidity x0.5 (ask1 3504 vol 6) -> crossing fill
            step(3)
            clients[D1].order_insert(RB, direction="0", offset="0", volume=2,
                                     limit_price=3504.0, exchange="SHFE", order_ref="D1A")
            ev1 = drain(clients[D1])
            assert st(ev1, "D1A") == ["a", "a", "0"], ev1
            assert tr(ev1, "D1A") == [(2, 3504.0)], ev1

            # 09:31:30: t0+90s -> the three 90s assertions evaluate
            step(4)
            a = admin.status()["assertions"]
            assert a["evaluated"] == 3 and a["passed"] == 2 and a["failed"] == 1, a
            items = {(i["investor"], i["metric"]): i for i in a["items"]}
            assert close(items[(D1, "orders_filled")]["actual"], 1.0), items
            assert 790000.0 < items[(D1, "balance")]["actual"] < 800000.0, items
            assert items[(D2, "orders_filled")]["pass"] is False, items

            # a resting sell (tick bid 3503) the 09:33:00 gap tick will cross
            clients[D2].order_insert(RB, direction="1", offset="0", volume=1,
                                     limit_price=3506.0, exchange="SHFE", order_ref="D2A")
            ev2 = drain(clients[D2])
            assert st(ev2, "D2A") == ["a", "3"], ev2
            assert tr(ev2, "D2A") == [], ev2
            # a rejected order: 9999 is beyond the 3850 upper limit. At the
            # 涨跌停板 check the front office has already accepted (DESIGN
            # §8.12): ReqOrderInsert answers OnRspOrderInsert{0} and the
            # exchange half lands later on OnErrRtnOrderInsert -- a client that
            # only挂 the request surface never sees it.
            clients[D1].order_insert(RB, direction="0", offset="0", volume=1,
                                     limit_price=9999.0, exchange="SHFE", order_ref="D1R")
            late = clients[D1].wait_late(ERR_RTN_ORDER_INSERT)
            assert late is not None, "涨跌停拒绝必须经 OnErrRtnOrderInsert 回报"
            d1r_error = rsp_info_of(late.payload)["ErrorID"]
            assert d1r_error == 163, late  # PRICE_OVER_LIMIT
            assert "涨跌停" in rsp_info_of(late.payload)["ErrorMsg"], late
            drain(clients[D1])

            # 09:32:00 was frozen away -> the 5th delivered tick is 09:32:30
            step(5)
            # 09:33:00: gap +2 (bid 3506) fills the resting sell at maker price
            step(6)
            ev3 = drain(clients[D2])
            assert st(ev3, "D2A")[-1] == "0", ev3
            assert tr(ev3, "D2A") == [(1, 3506.0)], ev3
            clients[D1].order_insert(RB, direction="0", offset="0", volume=2,
                                     limit_price=3510.0, exchange="SHFE", order_ref="D1B")
            ev4 = drain(clients[D1])
            assert st(ev4, "D1B") == ["a", "a", "0"], ev4
            assert tr(ev4, "D1B") == [(2, 3510.0)], ev4

            # 09:33:30: rest a buy below the bid, then cancel it
            step(7)
            clients[D2].order_insert(RB, direction="0", offset="0", volume=1,
                                     limit_price=3506.0, exchange="SHFE", order_ref="D2B")
            ev5 = drain(clients[D2])
            assert st(ev5, "D2B") == ["a", "3"], ev5
            clients[D2].order_action(RB, order_ref="D2B")
            ev6 = drain(clients[D2])
            assert st(ev6, "D2B")[-1] == "5", ev6

            # 09:34:00: t0+240s -> the fills assertion evaluates (dsl001 2 trades)
            step(8)
            a = admin.status()["assertions"]
            assert a["evaluated"] == 4 and a["passed"] == 3 and a["failed"] == 1, a
            assert close(a["items"][2]["actual"], 2.0), a

            # drain the rest of the stream
            for n in range(9, TOTAL_TICKS + 1):
                step(n)
            for cli in clients.values():
                drain(cli)

            # final ledger: dsl001 long 4 today, dsl002 short 1 today
            acct = clients[D1].qry_trading_account()
            # 4 lots at 昨结算 3500, not at the two fill prices (notes/04 C2)
            assert close(acct["CurrMargin"], 4 * 3500 * 10 * 0.16, 1e-4), acct
            assert close(acct["FrozenMargin"], 0.0, 1e-6), acct
            pos = clients[D1].qry_investor_position(RB)
            assert len(pos) == 1 and pos[0]["Position"] == 4 and pos[0]["TodayPosition"] == 4, pos
            pos2 = clients[D2].qry_investor_position(RB)
            assert len(pos2) == 1 and pos2[0]["PosiDirection"] == ord("3"), pos2
            assert pos2[0]["Position"] == 1, pos2
        finally:
            for cli in clients.values():
                cli.close()
        time.sleep(0.2)
        admin.shutdown()
        try:
            out, _ = proc.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            out, _ = proc.communicate()
        if "assertion FAILED" not in out:
            print(out.strip())
            raise AssertionError("the designed dsl002 assertion failure is missing from the log")
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.communicate()
    jdir = os.path.join(data_dir, "journal")
    assert os.path.isdir(jdir) and journal_file(jdir), "no journal written"
    return jdir, d1r_error


def check_recording(jfile: str, d1r_error: int) -> None:
    """The recording itself must carry the expected semantic payload."""
    events = load_events(jfile)
    types = [e["type"] for e in events]
    for t in ("server_start", "scenario_loaded", "session_auth", "session_login",
              "order_insert", "order_cancel", "order_update", "fill", "md_watermark",
              "assertion", "server_stop"):
        assert t in types, (t, count_types(events))
    inserts = [e for e in events if e["type"] == "order_insert"]
    by_ref = {e["data"]["order_ref"]: e for e in inserts}
    assert set(by_ref) == {"D1A", "D1B", "D1R", "D2A", "D2B"}, sorted(by_ref)
    # the request payload is complete enough to replay FAK/FOK exactly
    d = by_ref["D1A"]["data"]
    assert d["time_condition"] == "3" and d["volume_condition"] == "1", d
    assert d["min_volume"] == 1 and d["price_type"] == "2", d
    assert d["exchange"] == "SHFE" and d["limit_price"] == 3504.0, d
    assert d["outcome"]["accepted"] is True and len(d["outcome"]["fills"]) == 1, d
    # the rejected order keeps its full request too
    r = by_ref["D1R"]["data"]
    assert r["outcome"]["accepted"] is False, r
    assert r["outcome"]["error_id"] == d1r_error, r
    assert r["limit_price"] == 9999.0 and r["direction"] == ord("0"), r
    # a resting order, later filled by the gap tick at the maker price
    assert by_ref["D2A"]["data"]["outcome"]["accepted"] is True, by_ref["D2A"]
    # the cancel event
    cancels = [e for e in events if e["type"] == "order_cancel"]
    assert len(cancels) == 1 and cancels[0]["data"]["order_ref"] == "D2B", cancels
    # assertions: 3 pass + 1 designed failure, each one-shot
    assertions = [e for e in events if e["type"] == "assertion"]
    assert len(assertions) == 4, assertions
    assert sum(1 for e in assertions if e["data"]["pass"]) == 3, assertions
    failed = [e for e in assertions if not e["data"]["pass"]]
    assert failed[0]["investor"] == D2 and failed[0]["data"]["metric"] == "orders_filled", failed
    # watermark trace: 11 delivered ticks (the frozen 09:32:00 never appears)
    watermarks = [e["data"]["idx"] for e in events if e["type"] == "md_watermark"]
    assert watermarks == list(range(1, TOTAL_TICKS + 1)), watermarks
    # session events in first-appearance order
    sessions = [(e["type"], e["investor"]) for e in events
                if e["type"] in ("session_auth", "session_login")]
    assert sessions == [
        ("session_auth", D1), ("session_login", D1),
        ("session_auth", D2), ("session_login", D2),
        ("session_auth", LEGACY), ("session_login", LEGACY),
    ], sessions
    print("[ok] recording: %d events (%s)" % (
        len(events), " ".join("%s=%d" % kv for kv in sorted(count_types(events).items()))))


def main() -> int:
    core = find_core()
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-m2jnl-")
    try:
        scenario = prepare_scenario(tmp)

        # -- phase A: the same driver twice -> identical full-stream hash ----
        run1, d1r_error = record_run(core, scenario, os.path.join(tmp, "run1"))
        jf1 = journal_file(run1)
        check_recording(jf1, d1r_error)
        events1 = load_events(jf1)
        full1 = hash_stream(events1)
        core1 = hash_stream(events1, core=True)
        print("[ok] recorded run1: %d events, full hash %s" % (len(events1), full1[:16]))
        print("       core hash %s" % core1[:16])

        run2, _ = record_run(core, scenario, os.path.join(tmp, "run2"))
        events2 = load_events(journal_file(run2))
        full2 = hash_stream(events2)
        assert full1 == full2, (
            "M2-3 exit criterion failed: two identical drives produced different "
            "journals (full hashes %s vs %s)" % (full1, full2))
        assert hash_stream(events2, core=True) == core1
        print("[ok] determinism: run2 full hash == run1 (%s...)" % full2[:16])

        # -- phase B: CLI surface over the recording -------------------------
        assert cli_main(["journal", "verify", jf1]) == 0
        assert cli_main(["journal", "hash", jf1]) == 0
        assert cli_main(["journal", "hash", jf1, "--core"]) == 0
        assert cli_main(["journal", "show", jf1, "--type", "order_insert"]) == 0
        assert cli_main(["journal", "show", jf1, "--last", "3"]) == 0
        print("[ok] cli: journal verify / hash / hash --core / show")

        # -- phase C: replay the recording into a fresh core -----------------
        res = replay_journal(jf1, scenario, broker_id=BROKER,
                             initial_funds=DEFAULT_FUNDS)
        print(format_result(res))
        assert res["ok"], format_result(res)
        assert res["recorded_core"] == core1, (res["recorded_core"], core1)
        assert res["replay_core"] == core1, (res["replay_core"], core1)
        assert res["replay_events"] == len(events1), (res["replay_events"], len(events1))
        c = res["counts"]
        assert c["session"] == 6 and c["order"] == 5, c
        assert c["tick"] == TOTAL_TICKS and c["load"] == 1, c
        assert not res["problems"] and not res["notes"], res
        print("[ok] replay: re-driven recording reproduces the exact core hash")

        # -- phase D: mutate a recorded request -> replay must diverge -------
        mutated = os.path.join(tmp, "mutated.jsonl")
        n_mut = 0
        with open(mutated, "w", encoding="utf-8") as fout:
            for original in events1:
                ev = json.loads(json.dumps(original))
                if ev.get("type") == "order_insert" and \
                        ev["data"].get("order_ref") == "D1A":
                    # 3504 crossed the (halved) ask and filled; a buy limit
                    # BELOW the ask must rest instead — the engine's REACTION
                    # diverges, not just the recorded field
                    ev["data"]["limit_price"] = 3500.0
                    n_mut += 1
                fout.write(json.dumps(ev, ensure_ascii=False) + "\n")
        assert n_mut == 1, n_mut
        assert hash_stream(load_events(mutated)) != full1
        res2 = replay_journal(mutated, scenario, broker_id=BROKER,
                              initial_funds=DEFAULT_FUNDS)
        assert not res2["ok"], "the mutated replay must not pass"
        assert res2["recorded_core"] != res2["replay_core"], res2
        assert res2["replay_core"] != core1, "the replay itself must be unchanged"
        i, a, b = res2["diff"]
        assert 0 < i < 10, (
            "the first divergence must be the engine's reaction (an order_update /"
            " fill), before the tampered order_insert event itself (got %d)" % i)
        print("[ok] mutation: D1A limit 3504->3500 (now below the ask) diverges "
              "the replay core (first diff at core event %d)" % i)

        print("\nM2 JOURNAL: PASS")
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
