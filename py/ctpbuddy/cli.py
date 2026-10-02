"""`ctpbuddy` command line entry.

M1 surface:
  ctpbuddy serve [--scenario DIR] ...   start the Rust core (front + admin)
  ctpbuddy status                        core admin status
  ctpbuddy scenario validate DIR         check a scenario is loadable
  ctpbuddy scenario ticks DIR            summarize a scenario's tick stream
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
    print("instruments   %s" % v.get("instruments", 0))
    print("open orders   %s" % v.get("open_orders", 0))
    print("playback      loaded=%s idx=%s/%s paused=%s speed=%s vt=%s %s" % (
        pb.get("loaded"), pb.get("idx"), pb.get("total"),
        pb.get("paused"), pb.get("speed"), pb.get("virtual_time"), pb.get("trading_day"),
    ))
    for a in v.get("accounts", []):
        print("  account %-14s balance=%.2f available=%.2f profit=%.2f" % (
            a.get("investor"), a.get("balance", 0.0), a.get("available", 0.0), a.get("position_profit", 0.0),
        ))
    return 0


def cmd_scenario_validate(args: argparse.Namespace) -> int:
    from .sources import validate_scenario

    problems = validate_scenario(args.dir)
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
    return p


def main(argv: Optional[List[str]] = None) -> int:
    args = build_parser().parse_args(argv)
    return args.func(args)


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
