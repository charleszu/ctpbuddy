"""Deterministic journal replay (DESIGN.md §11.2, M2-3).

Replay merges the recorded request stream with the scenario tick stream
by virtual time (同时刻按 journal 记录序): the driver spawns a fresh
core, loads the recording's scenario **paused**, then walks the journal
in order —

- `session_auth` opens a client connection (connections are opened in
  the journal's first-appearance order and, when the journal carries
  the recorded `front_id`, the server's connection numbering is aligned
  with dummy connections so order keys `front/session/ref` match);
- `session_login` / `session_logout` drive the matching client;
- `order_insert` / `order_cancel` re-issue the recorded request (a
  recorded rejection is re-checked against the replay's `error_id`);
- `md_watermark` advances playback: `idx == current+1` is a plain step,
  any other index is a seek pattern (forward skip / backward seek /
  loop restart) and is repositioned by the watermark's `vt_ms` — the
  vt of the tick the recording had just released — before stepping to
  the recorded index.

After the walk the fresh journal's **core hash** (§11.4) must equal the
recording's: the replay reproduces the recording's semantic core
byte-for-byte. `reset_account` / `deposit` / `withdraw` / `settle` /
`admin` inputs are re-issued as admin commands when supported.

Limitations (documented in DESIGN §11.2): distinct tick `vt_ms` values
are assumed for seek patterns (ties resolve to the first tick); the
spawn-level `--speed` of a startup-path recording is not journaled, so
such replays use the scenario clock.
"""
from __future__ import annotations

import os
import queue
import shutil
import socket
import subprocess
import sys
import tempfile
import time
from collections import deque
from typing import Any, Deque, Dict, List, Optional, Tuple

from .journal import (
    canonical,
    core_normalize,
    hash_stream,
    load_events,
    verify_events,
)
from .sdk import Admin, Client, CTPError
from .scenario import load_scenario_spec

#: Journal events that are inputs to a replay (requests + the tick trace).
REPLAYABLE_TYPES = frozenset({
    "scenario_loaded",
    "session_auth", "session_login", "session_logout",
    "order_insert", "order_cancel",
    "reset_account", "deposit", "withdraw", "settle", "admin",
    "md_watermark",
})
#: Admin-command inputs the driver knows how to re-issue.
_ADMIN_INPUTS = frozenset({"reset_account", "deposit", "withdraw", "settle", "admin"})

_STEP_TIMEOUT = 20.0


class ReplayError(Exception):
    """The replay could not be driven to completion."""


def find_core() -> Optional[str]:
    env = os.environ.get("CTPBUDDY_CORE")
    if env and os.path.exists(env):
        return env
    found = shutil.which("ctpbuddy-server")
    if found:
        return found
    here = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    for rel in (
        os.path.join("core", "target", "debug", "ctpbuddy-server"),
        os.path.join("core", "target", "release", "ctpbuddy-server"),
        os.path.join("core", "target", "debug", "ctpbuddy-server.exe"),
        os.path.join("core", "target", "release", "ctpbuddy-server.exe"),
    ):
        cand = os.path.join(here, rel)
        if os.path.exists(cand):
            return cand
    return None


def free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port


def _connect_admin(addr: str, timeout: float = 10.0) -> Admin:
    """Connect the real admin session, retrying until the port is listening.

    Deliberately NOT a transient probe connection: whether the core accepts
    a connect-then-close probe before the real session is a race, and every
    consumed connection id shifts the replay's front_ids (DESIGN §11.2
    alignment). The first real connection gets id 1, deterministically.
    """
    deadline = time.time() + timeout
    while True:
        try:
            return Admin(addr)
        except OSError:
            if time.time() >= deadline:
                raise ReplayError("server did not open admin port %s" % addr)
            time.sleep(0.05)


def first_core_diff(
    recorded: List[Dict[str, Any]], replay: List[Dict[str, Any]]
) -> Optional[Tuple[int, str, str]]:
    """First differing event of the two normalized cores: `(index, a, b)`."""
    a = core_normalize(recorded)
    b = core_normalize(replay)
    for i in range(min(len(a), len(b))):
        ca, cb = canonical(a[i]), canonical(b[i])
        if ca != cb:
            return i, ca, cb
    if len(a) != len(b):
        i = min(len(a), len(b))
        return i, canonical(a[i]) if i < len(a) else "<end>", canonical(b[i]) if i < len(b) else "<end>"
    return None


def _num(v: Any) -> Optional[float]:
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        return None
    return float(v)


def _char_code(v: Any, default: str) -> str:
    """Journal direction/offset are CTP char codes (48='0'); be lenient."""
    if isinstance(v, str) and v:
        return v[0]
    n = _num(v)
    if n is None:
        return default
    try:
        return chr(int(n))
    except (ValueError, OverflowError):
        return default


def _drain(cli: Client) -> None:
    """Non-blocking drain of pending push frames (keeps queues flat)."""
    try:
        while True:
            next(cli.events(timeout=0.001))
    except (StopIteration, queue.Empty):
        pass


class _Walker:
    """Drives one replay server through the recorded input stream."""

    def __init__(self, admin: Admin, td_addr: str, broker_id: str,
                 verbose: bool = False) -> None:
        self.admin = admin
        self.td_addr = td_addr
        self.broker_id = broker_id
        self.verbose = verbose
        self.problems: List[str] = []
        self.notes: List[str] = []
        self.authed: Dict[Tuple[str, str], Deque[Client]] = {}
        self.active: Dict[Tuple[str, str], Client] = {}
        self.by_front: Dict[int, Client] = {}
        self.dummies: List[Client] = []
        self.clients: List[Client] = []
        self.scenario_dir = ""
        self._load_checked = False
        self.counts = {"load": 0, "session": 0, "order": 0, "cancel": 0,
                       "reset": 0, "admin": 0, "tick": 0}

    # -- helpers ------------------------------------------------------------

    def _say(self, msg: str) -> None:
        if self.verbose:
            print("[replay] %s" % msg)

    def _playback(self) -> Dict[str, Any]:
        return self.admin.status()["playback"]

    def _step_until(self, target: int) -> None:
        # Poll strictly slower than the core's 10ms pulse: one step command
        # per pulse is enough (step_once is idempotent), and a faster poll
        # only floods the world loop.
        deadline = time.time() + _STEP_TIMEOUT
        while True:
            cur = int(self._playback()["idx"])
            if cur >= target:
                return
            if time.time() >= deadline:
                raise ReplayError("playback stuck at idx %d (target %d)" % (cur, target))
            self.admin.step()
            time.sleep(0.015)

    def _align_front(self, front: Optional[float]) -> None:
        """Open throwaway connections until the next conn id is `front`, so
        the replay's front_ids match the recording's (order keys match).

        Uses the server's `next_conn_id` (a global counter that never reuses
        ids — closed connections leave gaps, so `connections + 1` is NOT the
        next id in general).
        """
        if front is None:
            return
        target = int(front)
        while True:
            nxt = self._next_conn_id()
            if nxt >= target:
                break
            self.dummies.append(Client(self.td_addr))
            # the world loop registers connections asynchronously: wait for
            # the counter to move before sizing the next gap
            deadline = time.time() + 5.0
            while time.time() < deadline:
                if self._next_conn_id() > nxt:
                    break
                time.sleep(0.004)
        nxt = self._next_conn_id()
        if nxt != target:
            self.notes.append(
                "front_id %d not reproducible (next connection id is %d); "
                "order keys may differ" % (target, nxt)
            )

    def _next_conn_id(self) -> int:
        st = self.admin.status()
        nxt = st.get("next_conn_id")
        if nxt is None:  # older core without the field
            return int(st["connections"]) + 1
        return int(nxt)

    # -- event handlers -----------------------------------------------------

    def on_watermark(self, ev: Dict[str, Any]) -> None:
        data = ev.get("data") or {}
        idx = _num(data.get("idx"))
        if idx is None:
            raise ReplayError("md_watermark without numeric idx: %r" % (data,))
        target = int(idx)
        cur = int(self._playback()["idx"])
        if target == cur + 1:
            self._step_until(target)
        else:
            # seek pattern (forward skip / backward seek / loop restart):
            # reposition at the vt of the tick the recording had just
            # released (the watermark envelope vt), then step to the index.
            vt = _num(ev.get("vt_ms"))
            if vt is None:
                raise ReplayError(
                    "md_watermark idx %d is not current+1 and carries no vt_ms" % target
                )
            self.admin.seek(vt)
            self._step_until(target)
        self.counts["tick"] += 1

    def on_scenario_loaded(self, ev: Dict[str, Any]) -> None:
        data = ev.get("data") or {}
        path = data.get("path") or ""
        if not path or not os.path.isdir(path):
            path = self.scenario_dir
        # the walk drives every tick release deterministically: always load
        # paused, whatever the recording did (speed is irrelevant while
        # paused; the release *sequence* is what the journal sees)
        speed = _num(data.get("speed"))
        spec = load_scenario_spec(path)
        started = self.admin.start_scenario(path, paused=True, speed=speed, spec=spec)
        if not self._load_checked:
            self._load_checked = True
            rec_ticks = _num(data.get("ticks"))
            if rec_ticks is not None and int(started.get("ticks", -1)) != int(rec_ticks):
                self.notes.append(
                    "replay loaded %s ticks, the recording %s — wrong scenario dir?"
                    % (started.get("ticks"), int(rec_ticks)))
        self._say("scenario %s: %s ticks, paused" % (
            started.get("name") or "-", started.get("ticks")))
        self.counts["load"] += 1

    def on_session_auth(self, ev: Dict[str, Any]) -> None:
        broker = ev.get("broker") or self.broker_id
        user = ev.get("investor") or ""
        if not user:
            raise ReplayError("session_auth without investor: %r" % (ev,))
        data = ev.get("data") or {}
        front = _num(data.get("front_id"))
        cli = self.by_front.get(int(front)) if front is not None else None
        if cli is None:
            self._align_front(front)
            cli = Client(self.td_addr)
            self.clients.append(cli)
            if front is not None:
                self.by_front[int(front)] = cli
        cli.auth(broker, user, app_id=data.get("app_id") or "ctpbuddy-python")
        self.authed.setdefault((broker, user), deque()).append(cli)
        self.counts["session"] += 1
        self._say("auth %s/%s (front_id %s)" % (broker, user, front))

    def on_session_login(self, ev: Dict[str, Any]) -> None:
        broker = ev.get("broker") or self.broker_id
        user = ev.get("investor") or ""
        q = self.authed.get((broker, user))
        if not q:
            raise ReplayError("session_login for %s/%s without a preceding session_auth" % (broker, user))
        cli = q.popleft()
        cli.login(broker, user, password="")
        try:
            cli.settle_confirm()  # ritual only: never journaled
        except CTPError:
            pass
        self.active[(broker, user)] = cli
        self.counts["session"] += 1
        self._say("login %s/%s" % (broker, user))

    def on_session_logout(self, ev: Dict[str, Any]) -> None:
        broker = ev.get("broker") or self.broker_id
        user = ev.get("investor") or ""
        cli = self.active.pop((broker, user), None)
        if cli is None:
            raise ReplayError("session_logout for %s/%s without a session" % (broker, user))
        cli.logout()
        self.counts["session"] += 1

    def _route(self, ev: Dict[str, Any]) -> Client:
        broker = ev.get("broker") or self.broker_id
        user = ev.get("investor") or ""
        cli = self.active.get((broker, user))
        if cli is None:
            raise ReplayError("no active session for %s/%s (event %s)" % (
                broker, user, ev.get("type")))
        return cli

    def on_order_insert(self, ev: Dict[str, Any]) -> None:
        cli = self._route(ev)
        d = ev.get("data") or {}
        outcome = d.get("outcome") or {}
        expected_accepted = bool(outcome.get("accepted", True))
        expected_err = _num(outcome.get("error_id"))
        kw = dict(
            instrument=d.get("instrument", ""),
            direction=_char_code(d.get("direction"), "0"),
            offset=_char_code(d.get("offset"), "0"),
            volume=int(d.get("volume", 1)),
            limit_price=float(d.get("limit_price", 0.0)),
            exchange=d.get("exchange", "") or "",
            order_ref=d.get("order_ref", "") or "",
            price_type=d.get("price_type", "2") or "2",
            time_condition=d.get("time_condition", "3") or "3",
            volume_condition=d.get("volume_condition", "1") or "1",
            min_volume=int(d.get("min_volume", 1)),
        )
        ref = kw["order_ref"] or "?"
        try:
            cli.order_insert(**kw)
            _drain(cli)
        except CTPError as e:
            if expected_accepted:
                self.problems.append(
                    "order %s (%s %s @%s x%d) was accepted in the recording "
                    "but rejected on replay: %s" % (
                        ref, kw["direction"], kw["offset"], kw["limit_price"],
                        kw["volume"], e))
            elif expected_err is not None and float(e.error_id) != expected_err:
                self.problems.append(
                    "order %s rejected with error %s on replay, %s in the recording"
                    % (ref, e.error_id, int(expected_err)))
        else:
            if not expected_accepted:
                self.problems.append(
                    "order %s (%s %s @%s x%d) was REJECTED in the recording "
                    "(error %s) but accepted on replay" % (
                        ref, kw["direction"], kw["offset"], kw["limit_price"],
                        kw["volume"], int(expected_err) if expected_err is not None else "?"))
        self.counts["order"] += 1
        self._say("order %s: %s %s @%s x%d (recorded accepted=%s)" % (
            ref, kw["direction"], kw["offset"], kw["limit_price"], kw["volume"],
            expected_accepted))

    def on_order_cancel(self, ev: Dict[str, Any]) -> None:
        cli = self._route(ev)
        d = ev.get("data") or {}
        outcome = d.get("outcome") or {}
        expected_accepted = bool(outcome.get("accepted", True))
        expected_err = _num(outcome.get("error_id"))
        ref = d.get("order_ref", "") or "?"
        try:
            cli.order_action(
                instrument=d.get("instrument", ""),
                order_ref=d.get("order_ref", "") or "",
                order_sys_id=d.get("order_sys_id", "") or "",
            )
            _drain(cli)
        except CTPError as e:
            # a rejected instruction (CTP 报盘拒绝 or the order-frequency
            # gate) is re-checked by error_id, exactly like a rejected insert
            if expected_accepted:
                self.problems.append(
                    "cancel %s was accepted in the recording but rejected on replay: %s"
                    % (ref, e))
            elif expected_err is not None and float(e.error_id) != expected_err:
                self.problems.append(
                    "cancel %s rejected with error %s on replay, %s in the recording"
                    % (ref, e.error_id, int(expected_err)))
        else:
            if not expected_accepted:
                self.problems.append(
                    "cancel %s was REJECTED in the recording (error %s) but accepted on replay"
                    % (ref, int(expected_err) if expected_err is not None else "?"))
        self.counts["cancel"] += 1
        self._say("cancel %s (recorded accepted=%s)" % (ref, expected_accepted))

    def on_admin_input(self, ev: Dict[str, Any]) -> None:
        t = ev["type"]
        if t == "reset_account":
            # standalone admin reset (a start_scenario byproduct never
            # reaches the journal — only scenario_loaded does)
            self.admin.reset_account(ev.get("investor") or "")
            self.counts["reset"] += 1
            self._say("reset_account %s" % (ev.get("investor") or "(all)"))
            return
        self.problems.append(
            "%s input events are not replayable by this version (event %s)"
            % (t, ev.get("type")))

    # -- main loop ----------------------------------------------------------

    def walk(self, events: List[Dict[str, Any]], scenario_dir: str) -> None:
        self.scenario_dir = scenario_dir
        for ev in events:
            t = ev.get("type")
            if t not in REPLAYABLE_TYPES:
                continue
            if t == "md_watermark":
                self.on_watermark(ev)
            elif t == "scenario_loaded":
                # every recorded load is reproduced at its journal position
                # (the first one included — there is no separate bootstrap
                # load when the recording itself starts with a scenario)
                self.on_scenario_loaded(ev)
            elif t == "session_auth":
                self.on_session_auth(ev)
            elif t == "session_login":
                self.on_session_login(ev)
            elif t == "session_logout":
                self.on_session_logout(ev)
            elif t == "order_insert":
                self.on_order_insert(ev)
            elif t == "order_cancel":
                self.on_order_cancel(ev)
            elif t in _ADMIN_INPUTS:
                self.on_admin_input(ev)

    def close(self) -> None:
        for cli in self.clients + self.dummies:
            try:
                cli.close()
            except OSError:
                pass


def replay_journal(
    journal: str,
    scenario: str,
    *,
    core: Optional[str] = None,
    broker_id: str = "8888",
    initial_funds: Optional[float] = None,
    keep_data: bool = False,
    verbose: bool = False,
) -> Dict[str, Any]:
    """Replay `journal` against `scenario` and compare core hashes.

    Returns a result dict: `ok`, `problems`, `notes`, `recorded_core`,
    `replay_core`, `diff`, `counts`, `data_dir` (kept when `keep_data`).
    """
    core = core or find_core()
    if core is None:
        raise ReplayError(
            "ctpbuddy-server binary not found; set CTPBUDDY_CORE or add it to PATH")
    events = load_events(journal)
    problems = verify_events(events)
    if problems:
        return {
            "ok": False, "problems": ["recording is structurally invalid:"] + problems,
            "notes": [], "recorded_core": None, "replay_core": None, "diff": None,
            "counts": {}, "data_dir": None, "events": len(events),
        }

    first_load = next((e for e in events if e["type"] == "scenario_loaded"), None)
    spec = load_scenario_spec(scenario)

    data_dir = tempfile.mkdtemp(prefix="ctpbuddy-replay-")
    td_port, admin_port = free_port(), free_port()
    cmd = [
        core,
        "--td", "127.0.0.1:%d" % td_port,
        "--admin", "127.0.0.1:%d" % admin_port,
        "--broker-id", broker_id,
        "--speed", "0",
        "--data-dir", data_dir,
    ]
    if initial_funds is not None:
        cmd += ["--initial-funds", str(initial_funds)]
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    admin: Optional[Admin] = None
    walker: Optional[_Walker] = None
    server_log = ""
    try:
        admin = _connect_admin("127.0.0.1:%d" % admin_port)
        admin.ping()
        walker = _Walker(admin, "127.0.0.1:%d" % td_port, broker_id, verbose=verbose)

        if first_load is None:
            # startup-path recording: the scenario was present from boot, so
            # the replay bootstraps it before walking (paused; the walk drives
            # every release). A journaled load is reproduced at its own
            # position by the walk itself.
            started = admin.start_scenario(scenario, paused=True, spec=spec)
            if verbose:
                print("[replay] bootstrap scenario %s: %s ticks, paused" % (
                    started.get("name") or "-", started.get("ticks")))
        elif spec and spec.get("name") != (first_load["data"] or {}).get("name"):
            walker.notes.append(
                "scenario name %r != recorded %r" % (
                    spec.get("name"), (first_load["data"] or {}).get("name")))
        walker.walk(events, scenario)

        admin.shutdown()
        try:
            server_log, _ = proc.communicate(timeout=8)
        except subprocess.TimeoutExpired:
            proc.kill()
            server_log, _ = proc.communicate()
        walker.close()
        admin.close()

        replay_events = load_events(os.path.join(data_dir, "journal"))
        recorded_core = hash_stream(events, core=True)
        replay_core = hash_stream(replay_events, core=True)
        diff = first_core_diff(events, replay_events)
        ok = not walker.problems and recorded_core == replay_core
        return {
            "ok": ok,
            "problems": walker.problems,
            "notes": walker.notes,
            "recorded_core": recorded_core,
            "replay_core": replay_core,
            "diff": diff,
            "counts": walker.counts,
            "data_dir": data_dir if keep_data else None,
            "events": len(events),
            "replay_events": len(replay_events),
            "server_log": server_log,
        }
    finally:
        if walker is not None:
            walker.close()
        if admin is not None:
            try:
                admin.close()
            except OSError:
                pass
        if proc.poll() is None:
            proc.kill()
            try:
                proc.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                pass
        if not keep_data:
            shutil.rmtree(data_dir, ignore_errors=True)


def format_result(result: Dict[str, Any]) -> str:
    """Human-readable replay report (CLI + e2e output)."""
    lines: List[str] = []
    for p in result.get("problems", []):
        lines.append("  ! %s" % p)
    for n in result.get("notes", []):
        lines.append("  - note: %s" % n)
    if result.get("recorded_core"):
        lines.append("recorded core hash  %s" % result["recorded_core"])
    if result.get("replay_core"):
        lines.append("replay   core hash  %s" % result["replay_core"])
    d = result.get("diff")
    if d is not None:
        i, a, b = d
        lines.append("first core diff at event %d:" % i)
        lines.append("  recorded: %s" % a)
        lines.append("  replay:   %s" % b)
    lines.append("RESULT: %s" % ("PASS — replay reproduces the recording's semantic core"
                                 if result.get("ok") else "FAIL"))
    return "\n".join(lines)


if __name__ == "__main__":  # pragma: no cover
    import argparse

    ap = argparse.ArgumentParser(description="replay a journal against a scenario")
    ap.add_argument("journal")
    ap.add_argument("--scenario", required=True)
    ap.add_argument("--broker-id", default="8888")
    ap.add_argument("--initial-funds", type=float, default=None)
    ap.add_argument("--keep-data", action="store_true")
    ap.add_argument("-v", "--verbose", action="store_true")
    args = ap.parse_args()
    try:
        res = replay_journal(
            args.journal, args.scenario, broker_id=args.broker_id,
            initial_funds=args.initial_funds, keep_data=args.keep_data,
            verbose=args.verbose)
    except (ReplayError, OSError) as e:
        print("error: %s" % e, file=sys.stderr)
        sys.exit(2)
    if res.get("data_dir"):
        print("[replay] data dir kept at %s" % res["data_dir"])
    print(format_result(res))
    sys.exit(0 if res.get("ok") else 1)
