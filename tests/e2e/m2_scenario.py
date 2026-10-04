#!/usr/bin/env python
"""M2-2 end-to-end: the scenario DSL pipeline and playback control.

Runs the real Rust `ctpbuddy-server` against the real wire protocol and drives
the whole M2-2 surface:

- scenario.yaml authoring -> compile (scenario.json) -> core startup path
  (`--scenario`), plus the ADMIN `start_scenario` path with the normalized spec
  inlined (the core never sees YAML);
- the transform pipeline through the real wire: freeze drops the 09:32:00 tick,
  liquidity halves the five-level volumes from 09:31:00, gap shifts prices +2
  from 09:33:00 (limits untouched) — verified both on the pushed depth MD and
  on real fill prices (a resting sell that rests before the gap tick is filled
  by it at the shifted price);
- scenario accounts (per-investor initial funds, default elsewhere);
- one-shot assertions at deterministic virtual times, incl. a failing one,
  observable through admin status and the journal `assertion` events;
- playback control: stepped replay while paused, seek (HH:MM:SS), loop restart.

Usage:
    python tests/e2e/m2_scenario.py
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
from ctpbuddy.scenario import compile_scenario, load_scenario_spec  # noqa: E402
from ctpbuddy.sdk import Admin, Client  # noqa: E402
from ctpbuddy.wire import RTN_DEPTH_MD, RTN_ORDER, RTN_TRADE  # noqa: E402

BROKER = "8888"
RB = "rb2601"  # SHFE: mult 10, tick 1, margin 0.16
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


def wait_port(port: int, timeout: float = 10.0) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            socket.create_connection(("127.0.0.1", port), timeout=0.5).close()
            return
        except OSError:
            time.sleep(0.05)
    raise SystemExit("server did not open port %d" % port)


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


class Feed:
    """Per-client accumulation of push events across the whole flow."""

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

    def md(self, name):
        return [f for k, f in self.log[name] if k == RTN_DEPTH_MD]


def check_md(md_fields, *, t, last, bid1, bid1v, ask1, ask1v):
    """Exactly one pushed depth tick with the expected transformed values."""
    assert len(md_fields) == 1, md_fields
    f = md_fields[0]
    assert f["UpdateTime"] == t, f
    assert close(f["LastPrice"], last), f
    assert close(f["BidPrice1"], bid1) and close(f["AskPrice1"], ask1), f
    assert f["BidVolume1"] == bid1v and f["AskVolume1"] == ask1v, f
    assert close(f["UpperLimitPrice"], 3850.0) and close(f["LowerLimitPrice"], 3150.0), f


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

    # the CLI surface over the same inputs
    assert cli_main(["scenario", "validate", scenario]) == 0
    assert cli_main(["scenario", "compile", scenario]) == 0
    return scenario


def run_flow(core: str, scenario: str) -> None:
    data_dir = os.path.join(os.path.dirname(scenario), "data")
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
            "--scenario", scenario,
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        run_scenario(td_port, admin_port, data_dir, scenario)
        print("\nM2 SCENARIO: PASS")
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


def run_scenario(td_port: int, admin_port: int, data_dir: str, scenario: str) -> None:
    # -- phase 0: startup path consumed scenario.json -----------------------
    wait_port(admin_port)
    admin = Admin("127.0.0.1:%d" % admin_port)
    assert admin.ping()["cmd"] == "ping"
    stt = admin.status()
    assert stt["broker_id"] == BROKER
    assert stt["scenario"] == SCENARIO_NAME, stt           # scenario.json was read
    assert stt["playback"]["total"] == TOTAL_TICKS, stt    # freeze applied at startup
    assert stt["assertions"]["total"] == 4, stt            # assertions registered
    accts = {a["investor"]: a for a in stt["accounts"]}
    assert close(accts[D1]["balance"], 800000.0), accts     # scenario accounts
    assert close(accts[D2]["balance"], 400000.0), accts
    assert LEGACY not in accts, accts
    print("[ok] startup: scenario.json consumed (%s, %d ticks, %d assertions, scenario accounts)" % (
        SCENARIO_NAME, TOTAL_TICKS, stt["assertions"]["total"]))

    # -- phase 1: investor sessions carry the scenario funds -----------------
    wait_port(td_port)
    clients = {}
    try:
        for name in (D1, D2, LEGACY):
            cli = Client("127.0.0.1:%d" % td_port)
            cli.auth(BROKER, name)
            cli.login(BROKER, name, password="")
            cli.settle_confirm()
            clients[name] = cli
        for cli in clients.values():
            cli.subscribe([RB])
        acct = {n: clients[n].qry_trading_account() for n in clients}
        assert close(acct[D1]["Balance"], 800000.0), acct[D1]
        assert close(acct[D2]["Balance"], 400000.0), acct[D2]
        assert close(acct[LEGACY]["Balance"], DEFAULT_FUNDS), acct[LEGACY]
        print("[ok] accounts: dsl001 800k / dsl002 400k (spec) vs legacy 1M (server default)")

        # -- phase 2: ADMIN reload with the inline spec, paused ---------------
        spec = load_scenario_spec(scenario)
        started = admin.start_scenario(scenario, paused=True, spec=spec)
        assert started["ticks"] == TOTAL_TICKS and started["paused"] is True, started
        assert started["name"] == SCENARIO_NAME, started
        assert started["assertions_passed"] == 0 and started["assertions_failed"] == 0, started
        pb = admin.status()["playback"]
        assert pb["idx"] == 0 and pb["paused"] is True and pb["looping"] is False, pb
        assert close(pb["speed"], 0.0), pb
        assert cli_main(["replay", "status", "--admin", "127.0.0.1:%d" % admin_port]) == 0
        print("[ok] admin start_scenario: inline spec, %d ticks, paused at idx 0" % started["ticks"])
        assert cli_main(["assertions", "check", "--admin", "127.0.0.1:%d" % admin_port]) == 1

        feed = Feed(clients)

        # -- phase 3: transforms, verified tick by tick ----------------------
        # j=0 09:30:00 raw (liquidity/gap not yet in force)
        admin.step(); wait_idx(admin, 1); feed.pump(D1)
        check_md(feed.md(D1)[-1:], t="09:30:00", last=3500, bid1=3498, bid1v=10, ask1=3502, ask1v=12)
        # j=1 09:30:30 raw
        admin.step(); wait_idx(admin, 2); feed.pump(D1)
        check_md(feed.md(D1)[-1:], t="09:30:30", last=3501, bid1=3499, bid1v=10, ask1=3503, ask1v=12)
        # j=2 09:31:00 liquidity x0.5: 10->5 / 12->6, prices untouched
        admin.step(); wait_idx(admin, 3); feed.pump(D1)
        check_md(feed.md(D1)[-1:], t="09:31:00", last=3502, bid1=3500, bid1v=5, ask1=3504, ask1v=6)
        print("[ok] liquidity: five-level volumes halved from 09:31:00 (10->5, 12->6)")

        # a crossing buy fills against the (halved) tick depth at the ask price
        clients[D1].order_insert(RB, direction="0", offset="0", volume=2, limit_price=3504.0,
                                 exchange="SHFE", order_ref="D1A")
        feed.pump(D1)
        assert feed.st(D1, "D1A") == ["a", "a", "0"], feed.log[D1]
        assert feed.tr(D1, "D1A") == [(2, 3504.0)], feed.log[D1]

        # j=3 09:31:30: virtual clock passes t0+90s -> the 90s assertions fire
        admin.step(); wait_idx(admin, 4); feed.pump(D1)
        check_md(feed.md(D1)[-1:], t="09:31:30", last=3503, bid1=3501, bid1v=5, ask1=3505, ask1v=6)
        a = admin.status()["assertions"]
        assert a["total"] == 4 and a["evaluated"] == 3, a
        assert a["passed"] == 2 and a["failed"] == 1, a
        items = {(i["investor"], i["metric"]): i for i in a["items"]}
        assert close(items[(D1, "orders_filled")]["actual"], 1.0), items
        # dynamic equity = 800000 minus the commission of the 2-lot fill
        bal = items[(D1, "balance")]["actual"]
        assert 790000.0 < bal < 800000.0 and items[(D1, "balance")]["pass"] is True, items
        assert close(items[(D2, "orders_filled")]["actual"], 0.0)
        assert items[(D2, "orders_filled")]["pass"] is False, items
        assert items[(D1, "fills")]["evaluated"] is False, items
        print("[ok] assertions @09:31:30: 2 pass + 1 fail (dsl002 never ordered), fills still pending")

        # j=4 09:32:30: the freeze dropped the 09:32:00 tick entirely
        admin.step(); wait_idx(admin, 5); feed.pump(D1)
        pushed = [f["UpdateTime"] for f in feed.md(D1)]
        assert pushed == ["09:30:00", "09:30:30", "09:31:00", "09:31:30", "09:32:30"], pushed
        check_md(feed.md(D1)[-1:], t="09:32:30", last=3505, bid1=3503, bid1v=5, ask1=3507, ask1v=6)
        print("[ok] freeze: 09:32:00 tick never delivered (%d pushes so far)" % len(pushed))

        # a sell at 3506 rests (tick bid1 is 3503); the gap tick will cross it
        clients[D2].order_insert(RB, direction="1", offset="0", volume=1, limit_price=3506.0,
                                 exchange="SHFE", order_ref="D2A")
        feed.pump(D2)
        assert feed.st(D2, "D2A") == ["a", "3"], feed.log[D2]
        assert feed.tr(D2, "D2A") == [], feed.log[D2]

        # j=5 09:33:00: gap +2 (3504/3508 -> 3506/3510); the resting sell is
        # filled by the shifted tick bid at the maker price 3506
        admin.step(); wait_idx(admin, 6); feed.pump(D1, D2)
        check_md(feed.md(D1)[-1:], t="09:33:00", last=3508, bid1=3506, bid1v=5, ask1=3510, ask1v=6)
        assert feed.st(D2, "D2A") == ["a", "3", "3", "0"], feed.log[D2]
        assert feed.tr(D2, "D2A") == [(1, 3506.0)], feed.log[D2]
        # second dsl001 fill at the shifted ask
        clients[D1].order_insert(RB, direction="0", offset="0", volume=2, limit_price=3510.0,
                                 exchange="SHFE", order_ref="D1B")
        feed.pump(D1)
        assert feed.st(D1, "D1B") == ["a", "a", "0"], feed.log[D1]
        assert feed.tr(D1, "D1B") == [(2, 3510.0)], feed.log[D1]
        print("[ok] gap: prices +2 from 09:33:00 (limits untouched); resting sell filled @3506 by the tick")

        # j=6..j=7 09:33:30 / 09:34:00: the 240s assertion fires at 09:34:00
        admin.step(); wait_idx(admin, 7); feed.pump(D1)
        check_md(feed.md(D1)[-1:], t="09:33:30", last=3509, bid1=3507, bid1v=5, ask1=3511, ask1v=6)
        admin.step(); wait_idx(admin, 8); feed.pump(D1)
        check_md(feed.md(D1)[-1:], t="09:34:00", last=3510, bid1=3508, bid1v=5, ask1=3512, ask1v=6)
        a = admin.status()["assertions"]
        assert a["evaluated"] == 4 and a["passed"] == 3 and a["failed"] == 1, a
        assert cli_main(["assertions", "check", "--admin", "127.0.0.1:%d" % admin_port, "--total", "4"]) == 1
        assert close(a["items"][2]["actual"], 2.0), a  # dsl001 fills >= 2
        print("[ok] assertions @09:34:00: fills=2 evaluated, 3 pass / 1 fail overall")

        # drain the rest of the stream
        for n in range(9, TOTAL_TICKS + 1):
            admin.step(); wait_idx(admin, n)
        feed.pump(D1)
        assert len(feed.md(D1)) == TOTAL_TICKS, len(feed.md(D1))
        pb = admin.status()["playback"]
        assert pb["idx"] == TOTAL_TICKS and pb["paused"] is True, pb
        print("[ok] stepped replay: %d/%d ticks, stream finished while paused" % (pb["idx"], pb["total"]))

        # -- phase 4: seek ----------------------------------------------------
        r = admin.seek("09:34:00")
        assert r["idx"] == 7 and r["total"] == TOTAL_TICKS, r
        assert r["virtual_time"] == "09:34:00", r
        stt = admin.status()
        assert stt["playback"]["idx"] == 7 and stt["playback"]["virtual_time"] == "09:34:00", stt
        n_before = len(feed.md(D1))
        admin.step(); wait_idx(admin, 8); feed.pump(D1)
        assert len(feed.md(D1)) == n_before + 1, (n_before, len(feed.md(D1)))
        assert feed.md(D1)[-1]["UpdateTime"] == "09:34:00"
        print("[ok] seek 09:34:00: idx 7, next delivered tick is exactly 09:34:00")

        # -- phase 5: loop restart -------------------------------------------
        assert admin.loop(True)["looping"] is True
        assert admin.status()["playback"]["looping"] is True
        # seek past the last day-session tick (17:59:59 is the end of the
        # trading-day timeline; 18:00+ is the night session that *opens* it)
        r = admin.seek("17:59:59")
        assert r["idx"] == TOTAL_TICKS and r["virtual_time"] == "17:59:59", r
        deadline = time.time() + 5.0
        idx = None
        while time.time() < deadline:
            idx = admin.status()["playback"]["idx"]
            if idx == 0:
                break
            time.sleep(0.02)
        assert idx == 0, "loop did not restart the stream (idx stays %s)" % idx
        n_before = len(feed.md(D1))
        admin.step(); wait_idx(admin, 1); feed.pump(D1)
        assert len(feed.md(D1)) == n_before + 1, (n_before, len(feed.md(D1)))
        assert feed.md(D1)[-1]["UpdateTime"] == "09:30:00"
        assert admin.loop(False)["looping"] is False
        print("[ok] loop: finished stream restarted at 09:30:00 while paused (step delivers tick 0)")

        # -- final ledger state ----------------------------------------------
        a = clients[D1].qry_trading_account()
        # 4 lots, margined at 昨结算 3500 x mult 10 x company rate 0.16 — the two
        # fill prices (3504 / 3510) do not enter the margin basis.
        assert close(a["CurrMargin"], 4 * 3500 * 10 * 0.16, 1e-4), a
        assert close(a["FrozenMargin"], 0.0, 1e-6) and close(a["FrozenCommission"], 0.0, 1e-6), a
        pos = clients[D1].qry_investor_position(RB)
        assert len(pos) == 1 and pos[0]["PosiDirection"] == ord("2"), pos
        assert pos[0]["Position"] == 4 and pos[0]["TodayPosition"] == 4, pos
        pos2 = clients[D2].qry_investor_position(RB)
        assert len(pos2) == 1 and pos2[0]["PosiDirection"] == ord("3"), pos2  # short today
        assert pos2[0]["Position"] == 1, pos2
        print("[ok] ledger: dsl001 long 4 today, dsl002 short 1 today, no residual freezes")
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
    assert "server_start" in types and "scenario_loaded" in types, types
    loaded = [e for e in events if e["type"] == "scenario_loaded"]
    assert len(loaded) == 1, loaded
    d = loaded[0]["data"]
    assert d["name"] == SCENARIO_NAME and d["ticks"] == TOTAL_TICKS, d
    assert d["transforms"] == 3 and d["accounts"] == 2 and d["assertions"] == 4, d
    assert d["paused"] is True and close(d["speed"], 0.0), d

    assertions = [e for e in events if e["type"] == "assertion"]
    # 4 evaluated on the first speed-0 pulse after startup + 4 after the
    # admin reload (the reload installs a fresh one-shot set)
    assert len(assertions) == 8, assertions
    loaded_seq = loaded[0]["seq"]
    startup = [e for e in assertions if e["seq"] < loaded_seq]
    reloaded = [e for e in assertions if e["seq"] > loaded_seq]
    assert len(startup) == 4 and len(reloaded) == 4, (startup, reloaded)
    # startup: nothing ordered yet -> only the balance assertion passes
    assert [e["data"]["pass"] for e in startup] == [False, True, False, False], startup
    assert [(e["investor"], e["data"]["metric"]) for e in startup] == [
        (D1, "orders_filled"), (D1, "balance"), (D1, "fills"), (D2, "orders_filled")], startup
    # reload: the driven flow — 3 pass, dsl002 never orders -> 1 fail
    by_key = {(e["investor"], e["data"]["metric"]): e["data"] for e in reloaded}
    assert set(by_key) == {
        (D1, "orders_filled"), (D1, "balance"), (D1, "fills"), (D2, "orders_filled")}, by_key
    assert close(by_key[(D1, "orders_filled")]["actual"], 1.0), by_key
    assert 790000.0 < by_key[(D1, "balance")]["actual"] < 800000.0, by_key
    assert close(by_key[(D1, "fills")]["actual"], 2.0), by_key
    assert close(by_key[(D2, "orders_filled")]["actual"], 0.0), by_key
    assert {k for k, v in by_key.items() if v["pass"]} == {
        (D1, "orders_filled"), (D1, "balance"), (D1, "fills")}, by_key
    assert by_key[(D1, "fills")]["op"] == ">=" and close(by_key[(D1, "fills")]["value"], 2.0), by_key
    assert by_key[(D1, "fills")]["after_ms"] == 240000, by_key
    assert all(e.get("seq", 0) > 0 for e in events), "seq missing"
    # each assertion is one-shot: exactly one journal event per installed set
    assert len({e["seq"] for e in reloaded}) == 4, reloaded
    print("[ok] journal: %d events, scenario_loaded + 8 one-shot assertion records (startup + reload)"
          % len(events))


def main() -> int:
    core = find_core()
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-m2scn-")
    scenario = prepare_scenario(tmp)
    run_flow(core, scenario)
    return 0


if __name__ == "__main__":
    sys.exit(main())
