"""`ctpbuddy` command line entry.

M1 surface:
  ctpbuddy serve [--scenario DIR] ...   start the Rust core (front + admin)
  ctpbuddy status                        core admin status

M2 scenario DSL surface:
  ctpbuddy scenario validate DIR         check scenario.yaml + ticks.csv load cleanly
  ctpbuddy scenario ticks DIR            summarize a scenario's tick stream
  ctpbuddy scenario compile DIR          scenario.yaml -> scenario.json (core format)
  ctpbuddy replay status|pause|resume|step|seek|loop|speed   playback control
  ctpbuddy assertions check [--admin ADDR] [--total N]      检查场景断言

M2-3 journal surface:
  ctpbuddy journal hash FILE [--core] [--only ...] [--skip ...]
                                        deterministic sha256 over the event stream
  ctpbuddy journal show FILE [--type T] [--last N]
                                        human-readable event listing
  ctpbuddy journal verify FILE          structural health (seq chain, types)
  ctpbuddy journal replay FILE --scenario DIR
                                        re-drive the recording, compare core hashes
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import signal
import subprocess
import sys
from typing import Any, Dict, List, Optional

from . import __version__
from .replay import find_core
from .sdk import Admin


def cmd_calendar_validate(args: argparse.Namespace) -> int:
    from .calendar import CalendarError, TradingCalendar

    try:
        calendar = TradingCalendar.from_file(args.file, expected_sha256=args.sha256)
    except (CalendarError, OSError) as exc:
        print("INVALID: %s" % exc, file=sys.stderr)
        return 1
    print("OK: %s" % args.file)
    print("version      %s" % calendar.metadata["version"])
    print("source       %s@%s (%s)" % (
        calendar.metadata["source"]["name"],
        calendar.metadata["source"]["revision"],
        calendar.metadata["source"]["license"],
    ))
    print("sha256       %s" % calendar.sha256)
    coverage = calendar.coverage
    print("coverage     %s..%s (%d rows, %d missing natural days, %d night records)" % (
        coverage["start"], coverage["end"], coverage["days"], coverage["missing_days"], coverage["night_records"]))
    return 0


def cmd_refdata_export(args: argparse.Namespace) -> int:
    """Materialize a ref-data provider to the canonical JSONL directory.

    This is the bridge in the plugin protocol: a provider may read anything
    (a spreadsheet, a Parquet dump, an exchange settlement file), but the core
    only ever sees the normalized JSONL this writes — no interpreter in the
    hot path, the same split as the market-data plugin (DESIGN §7.3).
    """
    from . import refdata

    spec: Dict[str, Any] = {"kind": args.kind, "provider": args.provider}
    if args.path:
        spec["path"] = args.path
    if args.options:
        for item in args.options:
            if "=" not in item:
                print("error: --options expects key=value, got %r" % item, file=sys.stderr)
                return 1
            k, v = item.split("=", 1)
            spec.setdefault("options", {})[k] = v
    try:
        provider = refdata.load_provider(spec)
        written = refdata.write_jsonl(provider, args.out)
    except (ValueError, ImportError) as e:
        print("error: %s" % e, file=sys.stderr)
        return 1
    for path in written:
        n = sum(1 for _ in open(path, encoding="utf-8"))
        print("wrote %-22s %5d rows  %s" % (os.path.basename(path), n, path))
    print("\n[ctpbuddy] pass it to the core with:  ctpbuddy serve --refdata %s" % args.out)
    return 0


def cmd_refdata_show(args: argparse.Namespace) -> int:
    """Validate a ref-data directory and summarize each table."""
    from . import refdata

    provider = refdata.jsonl_dir(args.dir)
    problems = refdata.validate(provider)
    for name, rows in refdata.iter_tables(provider):
        print("%-22s %5d rows" % (name, len(rows)))
        if args.verbose:
            for row in rows[: args.limit]:
                print("    " + ", ".join("%s=%s" % kv for kv in sorted(row.items())))
    if problems:
        print("\nproblems:", file=sys.stderr)
        for p in problems:
            print("  - %s" % p, file=sys.stderr)
        return 1
    print("\nok: %s" % args.dir)
    return 0


def cmd_serve(args: argparse.Namespace) -> int:
    core = find_core()
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
    if args.refdata:
        # A ref-data directory is a plain directory of JSONL the core reads
        # directly; nothing is compiled here (unlike a scenario's YAML).
        cmd += ["--refdata", args.refdata]
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


def cmd_web(args: argparse.Namespace) -> int:
    from .web import serve
    return serve(args.host, args.port, args.admin, args.db)


def cmd_install_shim(args: argparse.Namespace) -> int:
    from .shim_install import ShimInstallError, install_shim

    try:
        result = install_shim(args.target_dir, args.shim_dir, apply=args.apply)
    except (ShimInstallError, OSError) as exc:
        print("error: %s" % exc, file=sys.stderr)
        return 1
    mode = "applied" if result["applied"] else "dry-run"
    print("%s: %s" % (mode, result["action"]))
    print("target-dir   %s" % result["target_dir"])
    print("shim-dir     %s" % result["shim_dir"])
    print("version      %s" % result["metadata"]["version"])
    print("architecture %s" % result["metadata"]["architecture"])
    for item in result["files"]:
        print("  %s sha256=%s existed=%s" % (item["name"], item["source_sha256"], item["existed"]))
    if not args.apply:
        print("dry-run only; use --apply to modify target-dir")
    else:
        print("backup       %s" % result["backup_dir"])
        print("restore      %s" % result["restore_script"])
    return 0


def cmd_restore_shim(args: argparse.Namespace) -> int:
    from .shim_install import ShimInstallError, restore_shim

    try:
        result = restore_shim(args.target_dir)
    except (ShimInstallError, OSError) as exc:
        print("error: %s" % exc, file=sys.stderr)
        return 1
    print("restored: %s" % result["target_dir"])
    print("backup:   %s" % result["backup_dir"])
    return 0


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


def cmd_assertions_check(args: argparse.Namespace) -> int:
    try:
        with Admin(args.admin) as admin:
            status = admin.status()
    except (OSError, RuntimeError, ValueError) as e:
        print("error: %s" % e, file=sys.stderr)
        return 1
    assertions = status.get("assertions")
    if not isinstance(assertions, dict) or not assertions.get("total", 0):
        print("INVALID: no scenario assertions installed")
        return 1
    _print_assertions(assertions)
    total = assertions.get("total", 0)
    evaluated = assertions.get("evaluated", 0)
    failed = assertions.get("failed", 0)
    if args.total is not None and total != args.total:
        print("INVALID: expected total=%d, got %s" % (args.total, total))
        return 1
    if evaluated != total or failed:
        print("INVALID: assertions are not all passing")
        return 1
    print("OK: %d assertions passed" % total)
    return 0


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


# ---- journal ---------------------------------------------------------------


def _load_journal_or_fail(path: str):
    from .journal import JournalError, load_events

    try:
        return load_events(path), None
    except (JournalError, OSError) as e:
        return None, str(e)


def cmd_journal_hash(args: argparse.Namespace) -> int:
    from .journal import hash_stream

    events, err = _load_journal_or_fail(args.file)
    if err:
        print("error: %s" % err, file=sys.stderr)
        return 1
    only = [t for t in (args.only or "").split(",") if t] or None
    skip = [t for t in (args.skip or "").split(",") if t] or None
    digest = hash_stream(events, core=args.core, only=only, skip=skip)
    scope = "core (noise types dropped, seq re-encoded, session ids normalized)" if args.core \
        else "full stream (ts_wall dropped)"
    print("file         %s" % args.file)
    print("events       %d" % len(events))
    print("scope        %s" % scope)
    if only:
        print("only         %s" % ",".join(only))
    if skip:
        print("skip         %s" % ",".join(skip))
    print("sha256       %s" % digest)
    return 0


def cmd_journal_show(args: argparse.Namespace) -> int:
    from .journal import fmt_vt

    events, err = _load_journal_or_fail(args.file)
    if err:
        print("error: %s" % err, file=sys.stderr)
        return 1
    if args.type:
        events = [e for e in events if e.get("type") in args.type]
    if args.last:
        events = events[-args.last:]
    for e in events:
        who = e.get("investor") or ""
        data = json.dumps(e.get("data"), ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        print("%6s %s %-15s %-12s %s" % (
            e.get("seq"), fmt_vt(e.get("vt_ms")), e.get("type"), who, data))
    print("(%d events)" % len(events))
    return 0


def cmd_journal_verify(args: argparse.Namespace) -> int:
    from .journal import count_types, verify_events

    events, err = _load_journal_or_fail(args.file)
    if err:
        print("error: %s" % err, file=sys.stderr)
        return 1
    problems = verify_events(events)
    if problems:
        print("INVALID: %s" % args.file)
        for p in problems:
            print("  -", p)
        return 1
    counts = count_types(events)
    print("OK: %s" % args.file)
    print("events       %d (seq 1..%d unbroken)" % (len(events), len(events)))
    print("types        %s" % " ".join("%s=%d" % kv for kv in sorted(counts.items())))
    return 0


def cmd_journal_rebuild(args: argparse.Namespace) -> int:
    import sqlite3
    from .journal import JournalError
    from .store import rebuild

    database = args.db or os.path.join(
        os.path.dirname(os.path.abspath(args.file.rstrip("/\\"))), "ctpbuddy.db")
    try:
        result = rebuild(args.file, database)
    except (JournalError, OSError, sqlite3.Error, ValueError, KeyError, TypeError) as e:
        print("error: %s" % e, file=sys.stderr)
        return 1
    print(json.dumps(dict(result, database=database), ensure_ascii=False, sort_keys=True))
    return 0


def cmd_journal_query(args: argparse.Namespace) -> int:
    import sqlite3
    from .journal import JournalError
    from .store import Projection

    try:
        with Projection(args.db) as projection:
            rows = projection.query(args.table, broker=args.broker, investor=args.investor,
                                    trading_day=args.trading_day, limit=args.limit, offset=args.offset)
    except (JournalError, OSError, sqlite3.Error, ValueError) as e:
        print("error: %s" % e, file=sys.stderr)
        return 1
    print(json.dumps(rows, ensure_ascii=False, sort_keys=True))
    return 0


def cmd_journal_replay(args: argparse.Namespace) -> int:
    from .replay import ReplayError, format_result, replay_journal

    try:
        res = replay_journal(
            args.file,
            args.scenario,
            broker_id=args.broker_id,
            initial_funds=args.initial_funds,
            keep_data=args.keep_data,
            verbose=args.verbose,
        )
    except (ReplayError, OSError) as e:
        print("error: %s" % e, file=sys.stderr)
        return 1
    print("events       %d recorded / %d replayed" % (
        res.get("events", 0), res.get("replay_events", 0)))
    counts = res.get("counts") or {}
    if counts:
        print("replayed     %s" % ", ".join("%s=%d" % kv for kv in sorted(counts.items())))
    print(format_result(res))
    if args.verbose and res.get("server_log"):
        print("---- replay server log ----")
        print(res["server_log"].strip())
    return 0 if res.get("ok") else 1


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="ctpbuddy", description=__doc__)
    p.add_argument("--version", action="version", version="ctpbuddy " + __version__)
    sub = p.add_subparsers(dest="cmd", required=True)

    sp = sub.add_parser("serve", help="start the Rust core (front + admin endpoints)")
    sp.add_argument("--scenario", help="scenario dir (refdata/ optional, ticks.csv required)")
    sp.add_argument("--refdata", help="contracts + margin/commission rates (JSONL dir from `refdata export`)")
    sp.add_argument("--td", default="127.0.0.1:5560", help="CTP td front endpoint")
    sp.add_argument("--admin", default="127.0.0.1:5561", help="admin control endpoint")
    sp.add_argument("--broker-id", default="8888", help="the only BrokerID this core serves")
    sp.add_argument("--speed", type=float, default=None, help="playback speed multiplier (0 = as fast as possible)")
    sp.add_argument("--initial-funds", type=float, default=None, help="new-account initial funds")
    sp.add_argument("--data-dir", default=None, help="journal / runtime data dir")
    sp.set_defaults(func=cmd_serve)

    sp = sub.add_parser("web", help="start the local settings Web backend")
    sp.add_argument("--host", default="loopback", help="loopback or an explicit loopback IP")
    sp.add_argument("--port", type=int, default=8080)
    sp.add_argument("--admin", default="127.0.0.1:5561")
    sp.add_argument("--db", default=None, help="SQLite journal 投影路径，启用只读查询 API")
    sp.set_defaults(func=cmd_web)

    sp = sub.add_parser("install-shim", help="安全安装受支持的 shim DLL（默认 dry-run）")
    sp.add_argument("--target-dir", required=True, help="已存在的目标目录")
    sp.add_argument("--shim-dir", required=True, help="已存在的 shim 产物目录")
    sp.add_argument("--apply", action="store_true", help="实际修改目标目录；默认仅预览")
    sp.set_defaults(func=cmd_install_shim)

    sp = sub.add_parser("restore-shim", aliases=["uninstall-shim", "restore"], help="恢复 install-shim 创建的备份")
    sp.add_argument("--target-dir", required=True, help="已安装 shim 的目标目录")
    sp.set_defaults(func=cmd_restore_shim)

    sp = sub.add_parser("status", help="core admin status")
    sp.add_argument("--admin", default="127.0.0.1:5561")
    sp.set_defaults(func=cmd_status)

    sp = sub.add_parser("calendar", help="离线期货交易日历")
    csub = sp.add_subparsers(dest="calendar_cmd", required=True)
    cv = csub.add_parser("validate", help="校验 JSON 快照并输出固定版本和 SHA256")
    cv.add_argument("file")
    cv.add_argument("--sha256", help="期望的快照 SHA256")
    cv.set_defaults(func=cmd_calendar_validate)

    sp = sub.add_parser(
        "refdata",
        help="reference data (contracts + rates): export a provider to the core's format",
    )
    rsub = sp.add_subparsers(dest="refdata_cmd", required=True)
    ep = rsub.add_parser("export", help="run a provider and write canonical JSONL")
    ep.add_argument("--kind", choices=("csv", "jsonl", "plugin"), default="csv")
    ep.add_argument("--provider", help="package.module:Attribute (kind=plugin)")
    ep.add_argument("--path", help="source directory (kind=csv / jsonl)")
    ep.add_argument("--options", action="append", metavar="K=V",
                    help="extra provider options, repeatable")
    ep.add_argument("--out", required=True, help="output directory for the JSONL tables")
    ep.set_defaults(func=cmd_refdata_export)
    vp = rsub.add_parser("show", help="validate and summarize a ref-data directory")
    vp.add_argument("dir")
    vp.add_argument("-v", "--verbose", action="store_true", help="print sample rows")
    vp.add_argument("--limit", type=int, default=3)
    vp.set_defaults(func=cmd_refdata_show)

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

    sp = sub.add_parser("journal", help="journal utilities (hash / show / verify / replay)")
    jsub = sp.add_subparsers(dest="journal_cmd", required=True)
    from .store import TABLES
    rb = jsub.add_parser("rebuild", help="从 JSONL journal 原子重建 SQLite 投影")
    rb.add_argument("file", help="完整 journal JSONL 文件或目录")
    rb.add_argument("--db", help="目标投影路径，默认 journal 所在目录旁的 ctpbuddy.db")
    rb.set_defaults(func=cmd_journal_rebuild)
    qp = jsub.add_parser("query", help="只读查询 SQLite journal 投影，输出 JSON")
    qp.add_argument("table", choices=TABLES)
    qp.add_argument("--db", default="data/ctpbuddy.db")
    qp.add_argument("--broker")
    qp.add_argument("--investor")
    qp.add_argument("--trading-day")
    qp.add_argument("--limit", type=int, default=100)
    qp.add_argument("--offset", type=int, default=0)
    qp.set_defaults(func=cmd_journal_query)
    hp = jsub.add_parser("hash", help="deterministic sha256 over the event stream")
    hp.add_argument("file", help="journal .jsonl file or a journal directory")
    hp.add_argument("--core", action="store_true",
                    help="hash the semantic core (drop noise types, re-encode seq, "
                         "normalize session ids) — the replay comparison hash")
    hp.add_argument("--only", help="comma-separated event types to include")
    hp.add_argument("--skip", help="comma-separated event types to exclude")
    hp.set_defaults(func=cmd_journal_hash)
    shp = jsub.add_parser("show", help="list journal events")
    shp.add_argument("file", help="journal .jsonl file or a journal directory")
    shp.add_argument("--type", action="append", help="only these event types (repeatable)")
    shp.add_argument("--last", type=int, help="only the last N events")
    shp.set_defaults(func=cmd_journal_show)
    vp = jsub.add_parser("verify", help="structural health of a recording")
    vp.add_argument("file", help="journal .jsonl file or a journal directory")
    vp.set_defaults(func=cmd_journal_verify)
    rp = jsub.add_parser("replay", help="re-drive a recording and compare core hashes")
    rp.add_argument("file", help="journal .jsonl file or a journal directory")
    rp.add_argument("--scenario", required=True, help="scenario dir the recording was made with")
    rp.add_argument("--broker-id", default="8888")
    rp.add_argument("--initial-funds", type=float, default=None,
                    help="server default funds for new accounts (must match the recording)")
    rp.add_argument("--keep-data", action="store_true", help="keep the replay data dir")
    rp.add_argument("-v", "--verbose", action="store_true")
    rp.set_defaults(func=cmd_journal_replay)

    sp = sub.add_parser("assertions", help="scenario assertion checks")
    asub = sp.add_subparsers(dest="assertions_cmd", required=True)
    ac = asub.add_parser("check", help="fail unless all installed assertions passed")
    ac.add_argument("--admin", default="127.0.0.1:5561")
    ac.add_argument("--total", type=int, help="expected assertion count")
    ac.set_defaults(func=cmd_assertions_check)

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
