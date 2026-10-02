"""`ctpbuddy` command line entry.

M1 surface:
  ctpbuddy serve [--scenario DIR] ...   start the Rust core (front + admin)
  ctpbuddy status                        core admin status

M2 scenario DSL surface:
  ctpbuddy scenario validate DIR         check scenario.yaml + ticks.csv load cleanly
  ctpbuddy scenario ticks DIR            summarize a scenario's tick stream
  ctpbuddy scenario compile DIR          scenario.yaml -> scenario.json (core format)
  ctpbuddy replay status|pause|resume|step|seek|loop|speed   playback control
"""
from __future__ import annotations

import argparse
import os
import shutil
import signal
import subprocess
import sys
from typing import List, Optional

from . import __version__
from .sdk import Admin


def _find_core() -> Optional[str]:
    env = os.environ.get("CTPBUDDY_CORE")
    if env and os.path.exists(env):
        return env
    found = shutil.which("ctpbuddy-server")
    if found:
        return found
    # dev checkout: py/ is a sibling of core/
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


def cmd_serve(args: argparse.Namespace) -> int:
    core = _find_core()
    if core is None:
        print("error: ctpbuddy-server binary not found; set CTPBUDDY_CORE or add it to PATH", file=sys.stderr)
        return 1
    cmd: List[str] = [core, "--td", args.td, "--admin", args.admin, "--broker-id", args.broker_id]
    if args.scenario:
        # YAML is authored, JSON is consumed: compile scenario.yaml so the
        # core's startup path sees the spec (the core never parses YAML).
        from .scenario import ScenarioError, compile_scenario

        try:
            jpath = compile_scenario(args.scenario)
        except ScenarioError as e:
            print("error: scenario 编译失败: %s" % e, file=sys.stderr)
            return 1
        if jpath:
            print("[ctpbuddy] compiled scenario spec: %s" % jpath)
        cmd += ["--scenario", args.scenario]
    if args.speed is not None:
        cmd += ["--speed", str(args.speed)]
    if args.initial_funds is not None:
        cmd += ["--initial-funds", str(args.initial_funds)]
    if args.data_dir:
        cmd += ["--data-dir", args.data_dir]
    print("[ctpbuddy] starting:", " ".join(cmd))
    proc = subprocess.Popen(cmd)

    def _stop(signum: int, frame: object) -> None:  # pragma: no cover - signal path
        proc.terminate()

    signal.signal(signal.SIGINT, _stop)
    signal.signal(signal.SIGTERM, _stop)
    return proc.wait()


def cmd_status(args: argparse.Namespace) -> int:
    with Admin(args.admin) as admin:
        v = admin.status()
    pb = v.get("playback", {})
    print("server        %s (broker %s)" % (v.get("server", "?"), v.get("broker_id", "?")))
    print("journal seq   %s" % v.get("journal_seq", 0))
    print("connections   %s" % v.get("connections", 0))
    print("scenario      %s" % (v.get("scenario") or "-"))
    print("instruments   %s" % v.get("instruments", 0))
    print("open orders   %s" % v.get("open_orders", 0))
    print("playback      loaded=%s idx=%s/%s paused=%s speed=%s loop=%s vt=%s %s" % (
        pb.get("loaded"), pb.get("idx"), pb.get("total"),
        pb.get("paused"), pb.get("speed"), pb.get("looping"),
        pb.get("virtual_time"), pb.get("trading_day"),
    ))
    _print_assertions(v.get("assertions"))
    for a in v.get("accounts", []):
        print("  account %-14s balance=%.2f available=%.2f profit=%.2f" % (
            a.get("investor"), a.get("balance", 0.0), a.get("available", 0.0), a.get("position_profit", 0.0),
        ))
    return 0


def _print_assertions(a: object) -> None:
    if not isinstance(a, dict):
        return
    total = a.get("total", 0)
    if not total:
        return
    print("assertions    total=%s evaluated=%s passed=%s failed=%s" % (
        total, a.get("evaluated", 0), a.get("passed", 0), a.get("failed", 0),
    ))
    for it in a.get("items", []):
        mark = "PASS" if it.get("pass") else ("FAIL" if it.get("evaluated") else "wait")
        actual = it.get("actual")
        actual_s = "-" if actual is None else "%.4f" % actual
        print("  [%s] +%-6s %s.%s %s %s (actual %s)" % (
            mark, _fmt_ms(it.get("after_ms")), it.get("investor"), it.get("metric"),
            it.get("op"), it.get("value"), actual_s,
        ))


def _fmt_ms(ms: object) -> str:
    try:
        return "%.0fs" % (float(ms) / 1000.0)
    except (TypeError, ValueError):
        return str(ms)


def cmd_replay(args: argparse.Namespace) -> int:
    if args.action == "status":
        return cmd_status(args)
    with Admin(args.admin) as admin:
        if args.action == "pause":
            v = admin.pause()
            print("paused=%s" % v.get("paused"))
        elif args.action == "resume":
            v = admin.resume()
            print("paused=%s" % v.get("paused"))
        elif args.action == "step":
            v = admin.step()
            print("paused=%s" % v.get("paused"))
        elif args.action == "seek":
            v = admin.seek(args.at)
            print("seek at=%s idx=%s/%s vt=%s" % (
                v.get("at_ms"), v.get("idx"), v.get("total"), v.get("virtual_time"),
            ))
        elif args.action == "loop":
            v = admin.loop(not args.off)
            print("looping=%s" % v.get("looping"))
        elif args.action == "speed":
            v = admin.set_speed(args.value)
            print("speed=%s" % v.get("speed"))
        else:
            raise AssertionError("unreachable replay action %r" % args.action)
    return 0


def cmd_scenario_validate(args: argparse.Namespace) -> int:
    from .scenario import ScenarioError, load_scenario_spec
    from .sources import validate_scenario

    problems = validate_scenario(args.dir)
    # the DSL spec (if any) must also parse and validate cleanly
    try:
        spec = load_scenario_spec(args.dir)
    except ScenarioError as e:
        problems = list(problems) + ["scenario.yaml: %s" % e]
    else:
        if spec:
            n = len(spec.get("assertions", []))
            print("dsl           name=%s transforms=%d accounts=%d assertions=%d" % (
                spec.get("name") or "-",
                len(spec.get("transforms", [])),
                len(spec.get("accounts", [])),
                n,
            ))
    if problems:
        print("INVALID: %s" % args.dir)
        for p in problems:
            print("  -", p)
        return 1
    print("OK: %s" % args.dir)
    return 0


def cmd_scenario_ticks(args: argparse.Namespace) -> int:
    from .sources import iter_ticks

    n = 0
    instruments = set()
    first = last = None
    for t in iter_ticks(args.dir):
        if n == 0:
            first = t
        last = t
        n += 1
        instruments.add(t["instrument"])
    print("ticks       %d" % n)
    print("instruments %s" % ",".join(sorted(instruments)))
    if first:
        print("first       %s %s %s" % (first["trading_day"], first["update_time"], first["last_price"]))
        print("last        %s %s %s" % (last["trading_day"], last["update_time"], last["last_price"]))
    return 0


def cmd_scenario_compile(args: argparse.Namespace) -> int:
    from .scenario import ScenarioError, compile_scenario

    try:
        jpath = compile_scenario(args.dir)
    except ScenarioError as e:
        print("INVALID: %s" % args.dir)
        print("  -", e)
        return 1
    if jpath is None:
        print("error: %s 中没有 scenario.yaml" % args.dir, file=sys.stderr)
        return 1
    print("compiled: %s" % jpath)
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="ctpbuddy", description=__doc__)
    p.add_argument("--version", action="version", version="ctpbuddy " + __version__)
    sub = p.add_subparsers(dest="cmd", required=True)

    sp = sub.add_parser("serve", help="start the Rust core (front + admin endpoints)")
    sp.add_argument("--scenario", help="scenario dir (instruments.csv optional, ticks.csv required)")
    sp.add_argument("--td", default="127.0.0.1:5560", help="CTP td front endpoint")
    sp.add_argument("--admin", default="127.0.0.1:5561", help="admin control endpoint")
    sp.add_argument("--broker-id", default="8888", help="the only BrokerID this core serves")
    sp.add_argument("--speed", type=float, default=None, help="playback speed multiplier (0 = as fast as possible)")
    sp.add_argument("--initial-funds", type=float, default=None, help="new-account initial funds")
    sp.add_argument("--data-dir", default=None, help="journal / runtime data dir")
    sp.set_defaults(func=cmd_serve)

    sp = sub.add_parser("status", help="core admin status")
    sp.add_argument("--admin", default="127.0.0.1:5561")
    sp.set_defaults(func=cmd_status)

    sp = sub.add_parser("scenario", help="scenario utilities")
    ssub = sp.add_subparsers(dest="scenario_cmd", required=True)
    vp = ssub.add_parser("validate", help="check a scenario is loadable by the core")
    vp.add_argument("dir")
    vp.set_defaults(func=cmd_scenario_validate)
    tp = ssub.add_parser("ticks", help="summarize a scenario's tick stream")
    tp.add_argument("dir")
    tp.set_defaults(func=cmd_scenario_ticks)
    cp = ssub.add_parser("compile", help="compile scenario.yaml to scenario.json (core format)")
    cp.add_argument("dir")
    cp.set_defaults(func=cmd_scenario_compile)

    sp = sub.add_parser("replay", help="playback control (pause/resume/step/seek/loop/speed)")
    rsub = sp.add_subparsers(dest="replay_action", required=True)
    for name, help_text in (
        ("status", "core status (same as `ctpbuddy status`)"),
        ("pause", "pause playback"),
        ("resume", "resume playback"),
        ("step", "advance one tick while paused"),
    ):
        rp = rsub.add_parser(name, help=help_text)
        rp.add_argument("--admin", default="127.0.0.1:5561")
        rp.set_defaults(func=cmd_replay, action=name)
    sk = rsub.add_parser("seek", help="seek to a virtual time (HH:MM:SS or ms since midnight)")
    sk.add_argument("at")
    sk.add_argument("--admin", default="127.0.0.1:5561")
    sk.set_defaults(func=cmd_replay, action="seek")
    lp = rsub.add_parser("loop", help="toggle looping playback")
    lp.add_argument("--off", action="store_true", help="turn looping off")
    lp.add_argument("--admin", default="127.0.0.1:5561")
    lp.set_defaults(func=cmd_replay, action="loop")
    sd = rsub.add_parser("speed", help="set playback speed multiplier (0 = as fast as possible)")
    sd.add_argument("value", type=float)
    sd.add_argument("--admin", default="127.0.0.1:5561")
    sd.set_defaults(func=cmd_replay, action="speed")
    return p


def main(argv: Optional[List[str]] = None) -> int:
    args = build_parser().parse_args(argv)
    return args.func(args)


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
