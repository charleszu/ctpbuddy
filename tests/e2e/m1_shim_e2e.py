#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""M1 shim end-to-end: a REAL CTP application (demo_td.exe) talks through the
CTPBuddy shim DLLs to the Rust `ctpbuddy-server`.

This closes the M1 loop: the shim is only proven when an ordinary CTP 6.7.13
app -- one that includes the stock vendor headers and links the shim import
libs -- completes a full session (login -> settle -> subscribe -> tick ->
crossing fill -> queries -> park/cancel -> close -> logout) against the real
core, and the journal records the flow. The demo also rides out both halves
of CTP's query flow control: the in-flight gate (-2) and the per-second
QryFreq budget (ErrorID 90 + client retry).

Harness choreography: the scenario is loaded PAUSED; when the demo prints
"subscribed" the harness resumes playback so the first RTN_DEPTH_MD lands
after the subscription is in place.

Usage:
    python tests/e2e/m1_shim_e2e.py
Env:
    CTPBUDDY_CORE   explicit path to the ctpbuddy-server binary
    CTPBUDDY_SHIM   explicit path to shim/bin (defaults to the repo layout)
"""
from __future__ import annotations

import json
import os
import queue
import subprocess
import sys
import tempfile
import threading
import time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)  # reuse the smoke harness helpers
sys.path.insert(0, os.path.join(REPO, "py"))

from m1_smoke import BROKER, INITIAL_FUNDS, INSTRUMENT, INVESTOR, find_core, free_port, make_scenario, wait_port  # noqa: E402

from ctpbuddy.sdk import Admin  # noqa: E402

SHIM_BIN = os.environ.get("CTPBUDDY_SHIM", os.path.join(REPO, "shim", "bin"))
DEMO_EXE = os.path.join(SHIM_BIN, "demo_td.exe")
DEMO_TIMEOUT_SEC = 90
DEMO_MODE = os.environ.get("CTPBUDDY_SHIM_MODE", "")


def find_demo() -> str:
    if not os.path.exists(DEMO_EXE):
        raise SystemExit(
            "%s not found; build it first: python shim/build_msvc.py --demo" % DEMO_EXE
        )
    for dll in ("thosttraderapi_se.dll", "thostmduserapi_se.dll"):
        if not os.path.exists(os.path.join(SHIM_BIN, dll)):
            raise SystemExit("%s missing next to the demo; run shim/build_msvc.py" %
                             os.path.join(SHIM_BIN, dll))
    return DEMO_EXE


def run_demo(front: str, admin: Admin, log: list) -> int:
    """Run demo_td.exe; resume the scenario once it is subscribed.

    Returns the demo exit code; all output lines land in `log`.
    """
    proc = subprocess.Popen(
        [DEMO_EXE, front, BROKER, INVESTOR] + ([DEMO_MODE] if DEMO_MODE else []),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        cwd=SHIM_BIN,
    )
    lines: "queue.Queue[str]" = queue.Queue()

    def pump() -> None:
        assert proc.stdout is not None
        for line in proc.stdout:
            line = line.rstrip()
            lines.put(line)
            print("  demo| " + line)
        lines.put("")  # EOF sentinel

    t = threading.Thread(target=pump, daemon=True)
    t.start()

    deadline = time.time() + DEMO_TIMEOUT_SEC
    subscribed = False
    finished = False
    exit_code = None
    while time.time() < deadline:
        try:
            line = lines.get(timeout=0.2)
        except queue.Empty:
            continue
        if line == "":
            finished = True
            break
        log.append(line)
        if "subscribed" in line and not subscribed:
            subscribed = True
            print("  [ok] demo subscribed -> resuming scenario playback")
            admin.resume()
        if line.startswith("DEMO:"):
            finished = True
            break
    if not finished:
        proc.kill()
        raise AssertionError("demo did not finish within %ds" % DEMO_TIMEOUT_SEC)
    exit_code = proc.wait(timeout=10)
    t.join(timeout=5)
    return exit_code


def main() -> int:
    demo = find_demo()
    core = find_core()
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-shim-e2e-")
    scenario = os.path.join(tmp, "scenario")
    os.makedirs(scenario)
    make_scenario(scenario)
    data_dir = os.path.join(tmp, "data")
    td_port, admin_port = free_port(), free_port()

    print("[e2e] core=%s" % core)
    print("[e2e] demo=%s" % demo)
    proc = subprocess.Popen(
        [
            core,
            "--td", "127.0.0.1:%d" % td_port,
            "--admin", "127.0.0.1:%d" % admin_port,
            "--broker-id", BROKER,
            "--speed", "0",
            "--initial-funds", str(INITIAL_FUNDS),
            "--data-dir", data_dir,
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        wait_port(admin_port)
        admin = Admin("127.0.0.1:%d" % admin_port)
        assert admin.ping()["cmd"] == "ping"
        started = admin.start_scenario(scenario, paused=True)
        assert started["ticks"] == 3 and started["paused"] is True, started
        print("[ok] scenario loaded paused: %d ticks, day %s" % (started["ticks"], started["trading_day"]))

        wait_port(td_port)
        log: list = []
        t0 = time.time()
        code = run_demo("tcp://127.0.0.1:%d" % td_port, admin, log)
        dt = time.time() - t0
        if code != 0:
            raise AssertionError("demo_td.exe exited %d (see demo output above)" % code)
        if DEMO_MODE == "auth-check":
            if not any(line.strip() == "AUTH CHECKS: PASS" for line in log):
                raise AssertionError("auth checks did not report PASS")
            print("[ok] explicit ReqAuthenticate callback/error/login checks")
            admin.shutdown()
            print("\nM1 SHIM AUTH E2E: PASS")
            return 0
        if not any(line.strip() == "DEMO: PASS" for line in log):
            raise AssertionError("demo did not report PASS")
        print("[ok] full CTP round trip through the shim in %.1fs" % dt)

        # -- CTP query flow control (docs: 查询流控): the client-side in-flight
        #    gate (-2, shim) and the front-side per-second budget (QryFreq,
        #    core answering ErrorID 90, demo retrying) must both have engaged.
        assert any("rc=-2" in line for line in log), "in-flight query gate (-2) not exercised"
        assert any("NEED_RETRY loop" in line for line in log), "per-second QryFreq throttle not exercised"
        print("[ok] query flow control: -2 in-flight gate + QryFreq NEED_RETRY retry")

        # -- journal: the authoritative event stream must record the flow -------
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
        for want in ("session_auth", "session_login", "order_insert", "order_update", "fill", "order_cancel"):
            assert want in types, (want, types)
        fills = [e for e in events if e["type"] == "fill"]
        assert len(fills) == 2, fills
        assert all(e.get("seq", 0) > 0 for e in events), "seq missing"
        print("[ok] journal: %d events, types=%s" % (len(events), ",".join(sorted(set(types)))))

        print("\nM1 SHIM E2E: PASS")
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


if __name__ == "__main__":
    sys.exit(main())
