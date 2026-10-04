#!/usr/bin/env python
"""Python-side unit checks (stdlib only, no pytest needed).

Covers the pieces the e2e smoke depends on:
- generated struct layout (must match ctypes-verified sizes at codegen time)
- frame codec roundtrip incl. split writes and clean EOF
- canonical CSV scenario roundtrip + validation
- scenario DSL: mini-YAML parse, spec normalization, compile cache semantics

Run: python tests/test_py.py
"""
from __future__ import annotations

import json
import os
import socket
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "py"))

from ctpbuddy.calendar import CalendarError, TradingCalendar  # noqa: E402
from ctpbuddy.generated import structs  # noqa: E402
from ctpbuddy.scenario import (  # noqa: E402
    ScenarioError,
    compile_scenario,
    load_scenario_spec,
    normalize_spec,
    parse_duration_ms,
    parse_time_ms,
    parse_yaml,
)
from ctpbuddy.sources import (  # noqa: E402
    CANONICAL_COLUMNS,
    iter_ticks,
    validate_scenario,
    write_canonical,
)
from ctpbuddy.wire import HEADER_LEN, Frame  # noqa: E402


def test_struct_layout() -> None:
    n = structs.verify_layout()
    assert n == 519, n
    # a packed struct must roundtrip through pack/unpack
    buf = structs.pack(
        "CThostFtdcInputOrderField",
        BrokerID="8888",
        InvestorID="inv001",
        InstrumentID="rb2601",
        LimitPrice=3502.0,
        VolumeTotalOriginal=3,
        Direction="0",
        CombOffsetFlag="0",
    )
    f = structs.unpack("CThostFtdcInputOrderField", buf)
    assert f["BrokerID"] == "8888" and f["InvestorID"] == "inv001", f
    assert f["InstrumentID"] == "rb2601" and f["LimitPrice"] == 3502.0, f
    assert f["VolumeTotalOriginal"] == 3, f
    assert f["Direction"] == ord("0") and f["CombOffsetFlag"] == "0", f
    print("[ok] struct layout (%d structs) + pack/unpack roundtrip" % n)


def test_frames() -> None:
    s1, s2 = socket.socketpair()
    try:
        f = Frame(0x1001, 42, b"payload-bytes")
        assert len(f.encode()) == HEADER_LEN + len(b"payload-bytes")
        s1.sendall(f.encode())
        g = Frame.decode_from(s2)
        assert g.msg_type == 0x1001 and g.req_id == 42 and g.payload == b"payload-bytes", g
        # split writes must reassemble
        s1.sendall(f.encode()[:7])
        s1.sendall(f.encode()[7:])
        g = Frame.decode_from(s2)
        assert g.payload == b"payload-bytes", g
        # empty payload frame
        s1.sendall(Frame(0x1050, 1).encode())
        g = Frame.decode_from(s2)
        assert g.msg_type == 0x1050 and g.payload == b"", g
        # clean EOF
        s1.close()
        assert Frame.decode_from(s2) is None
    finally:
        s2.close()
    print("[ok] frame codec: roundtrip, split writes, empty payload, EOF")


def test_scenario() -> None:
    with tempfile.TemporaryDirectory() as d:
        row = {c: "" for c in CANONICAL_COLUMNS}
        row.update(
            instrument="rb2601", exchange="SHFE", trading_day="20261002",
            update_time="09:30:00", update_millisec="0", last_price="3500",
            volume="10", upper="3850", lower="3150",
            bid1="3498", ask1="3502", bidvol1="5", askvol1="5",
        )
        write_canonical(d, [row])
        assert validate_scenario(d) == [], validate_scenario(d)
        ticks = list(iter_ticks(d))
        assert len(ticks) == 1 and ticks[0]["instrument"] == "rb2601", ticks
        assert ticks[0]["bid1"] == "3498" and ticks[0]["ask1"] == "3502", ticks[0]
        # broken scenarios are rejected
        with open(os.path.join(d, "ticks.csv"), "a", encoding="utf-8") as fh:
            fh.write("rb2601,SHFE,20261002,09:30:01,0\n")
        problems = validate_scenario(d)
        assert problems and "columns" in problems[0], problems
    print("[ok] scenario: write/validate/iter + broken-case rejection")


def test_dsl_spec() -> None:
    """DESIGN §7.4 example verbatim + authoring styles + time/duration primitives."""
    design_example = (
        "# scenario.yaml 示例\n"
        "name: flash-crash-cu\n"
        "source:\n"
        "  kind: csv            # csv | parquet | plugin\n"
        "  path: ./ticks/cu2501.csv\n"
        "transforms:\n"
        "  - kind: freeze       # 停牌：指定时段无新行情\n"
        '    at: "2025-03-14 10:29:00"\n'
        "    duration: 90s\n"
        "  - kind: gap          # 跳空：指定时刻价格平移\n"
        '    at: "2025-03-14 11:00:00"\n'
        "    shift: -3.0\n"
        "  - kind: liquidity    # 流动性缩放：五档挂量 × 0.1\n"
        '    from: "2025-03-14 13:30:00"\n'
        "    scale: 0.1\n"
        "clock:\n"
        "  time_scale: 50        # 50 倍速\n"
        '  start: "2025-03-14 09:00:00"\n'
        "accounts:\n"
        "  - investor: 001      # 自动开户，初始资金可配\n"
        "    balance: 2000000\n"
        "assertions:             # 可选：场景内断言（CI 用）\n"
        "  - after: 30s\n"
        "    investor: 001\n"
        '    expect: { orders_filled: ">=1" }\n'
    )
    spec = normalize_spec(parse_yaml(design_example))
    assert spec["name"] == "flash-crash-cu", spec
    assert spec["source"] == {"kind": "csv", "path": "./ticks/cu2501.csv"}, spec
    assert spec["transforms"] == [
        {"kind": "freeze", "at_ms": 37740000.0, "duration_ms": 90000.0},
        {"kind": "gap", "at_ms": 39600000.0, "shift": -3.0},
        {"kind": "liquidity", "from_ms": 48600000.0, "scale": 0.1},
    ], spec["transforms"]
    assert spec["clock"] == {"time_scale": 50.0, "start_ms": 32400000.0}, spec
    # unquoted 001 parses as int then stringifies — same as real YAML semantics
    assert spec["accounts"] == [{"investor": "1", "balance": 2000000.0}], spec
    assert spec["assertions"] == [
        {"after_ms": 30000.0, "investor": "1", "metric": "orders_filled", "op": ">=", "value": 1.0},
    ], spec["assertions"]

    # sequence at the key's own indent is valid authoring style too
    flat = (
        "name: flat\n"
        "source:\n"
        "  kind: csv\n"
        "  path: ticks.csv\n"
        "transforms:\n"
        "- kind: freeze\n"
        '  at: "09:32:00"\n'
        "  duration: 30s\n"
        "clock:\n"
        "  time_scale: 0\n"
        "assertions:\n"
        "- after: 1h30m\n"
        '  investor: "7"\n'
        "  expect: { fills: '>=2', balance: '>0' }\n"
    )
    spec2 = normalize_spec(parse_yaml(flat))
    assert spec2["transforms"] == [
        {"kind": "freeze", "at_ms": 34320000.0, "duration_ms": 30000.0},
    ], spec2["transforms"]
    assert spec2["clock"] == {"time_scale": 0.0}, spec2["clock"]
    # one expect entry fans out to one assertion per metric
    assert spec2["assertions"] == [
        {"after_ms": 5400000.0, "investor": "7", "metric": "fills", "op": ">=", "value": 2.0},
        {"after_ms": 5400000.0, "investor": "7", "metric": "balance", "op": ">", "value": 0.0},
    ], spec2["assertions"]

    # time / duration primitives
    assert parse_time_ms("09:30:00") == 34200000.0
    assert parse_time_ms("09:30:00.500") == 34200500.0
    assert parse_time_ms("2025-03-14 13:30:00") == 48600000.0
    assert parse_duration_ms("90s") == 90000.0
    assert parse_duration_ms("5m") == 300000.0
    assert parse_duration_ms("1h") == 3600000.0
    assert parse_duration_ms("1h30m") == 5400000.0
    assert parse_duration_ms("30") == 30000.0
    print("[ok] dsl: DESIGN §7.4 example, flat style, expect fan-out, time/duration")


def test_dsl_errors() -> None:
    """Every schema violation must fail with a path-carrying message."""

    def bad(text: str, needle: str) -> None:
        try:
            normalize_spec(parse_yaml(text))
        except ScenarioError as e:
            assert needle in str(e), (needle, str(e))
            return
        raise AssertionError("应当报错: %r（线索 %r）" % (text, needle))

    bad("bogus: 1\n", "未知顶层字段")
    bad("transforms:\n  - kind: zap\n", "未知 kind")
    bad("transforms:\n  - kind: gap\n    at: \"09:00:00\"\n", "shift")
    bad('assertions:\n  - after: 1s\n    investor: "1"\n    expect: { bogus: ">=1" }\n', "未知指标")
    bad('assertions:\n  - after: 1s\n    investor: "1"\n    expect: { fills: "many" }\n', "无法解析")
    bad("transforms:\n  - kind: freeze\n    at: \"25:00:00\"\n    duration: 1s\n", "时间越界")
    bad("clock:\n\ttime_scale: 1\n", "tab")
    bad("- item\n", "顶层必须是映射")
    # YAML-level errors carry the offending line number
    try:
        parse_yaml("name: x\n  stray: 1\n")
        raise AssertionError("缩进错位应当报错")
    except ScenarioError as e:
        assert "line 2" in str(e), str(e)
    print("[ok] dsl: error paths (unknown field/kind/metric, bad expr, out-of-range, tab, non-map, indent)")


def test_dsl_compile() -> None:
    """compile_scenario writes scenario.json; load prefers fresh json, else yaml."""
    with tempfile.TemporaryDirectory() as d:
        # legacy ticks.csv-only scenario: no spec at all
        assert load_scenario_spec(d) == {}, load_scenario_spec(d)
        assert compile_scenario(d) is None
        yaml_text = (
            "name: compile-me\n"
            "source: { kind: csv, path: ticks.csv }\n"
            "transforms:\n"
            "  - kind: liquidity\n"
            '    from: "09:31:00"\n'
            "    scale: 0.5\n"
        )
        with open(os.path.join(d, "scenario.yaml"), "w", encoding="utf-8") as fh:
            fh.write(yaml_text)
        jpath = compile_scenario(d)
        assert jpath == os.path.join(d, "scenario.json"), jpath
        spec = load_scenario_spec(d)
        assert spec["name"] == "compile-me", spec
        assert spec["transforms"] == [
            {"kind": "liquidity", "from_ms": 34260000.0, "scale": 0.5},
        ], spec
        with open(jpath, "r", encoding="utf-8") as fh:
            assert json.load(fh) == spec

        # yaml edited but kept older than the json -> compiled cache wins
        with open(os.path.join(d, "scenario.yaml"), "w", encoding="utf-8") as fh:
            fh.write("name: stale-yaml\n")
        past = time.time() - 10
        os.utime(os.path.join(d, "scenario.yaml"), (past, past))
        assert load_scenario_spec(d)["name"] == "compile-me"

        # yaml newer than the json -> yaml is re-parsed
        future = time.time() + 10
        os.utime(os.path.join(d, "scenario.yaml"), (future, future))
        assert load_scenario_spec(d)["name"] == "stale-yaml"
        print("[ok] dsl: compile/load cache semantics + legacy fallback")


def test_calendar() -> None:
    snapshot = {
        "schema": "ctpbuddy.trading-calendar/v1",
        "version": "fixture-2026-10-r1",
        "source": {"kind": "fixture", "name": "unit-test", "revision": "r1", "license": "test", "scope": "futures"},
        "days": [
            {"date": "2026-10-02", "is_trading_day": True, "trading_day": "20261002", "exchanges": {"SHFE": {"night_action_day": "2026-10-02", "night_trading_day": "20261005"}}},
            {"date": "2026-10-03", "is_trading_day": False},
            {"date": "2026-10-04", "is_trading_day": False},
            {"date": "2026-10-05", "is_trading_day": True, "trading_day": "20261005"},
        ],
    }
    cal = TradingCalendar.from_json(json.dumps(snapshot))
    assert cal.is_trading_day("2026-10-02")
    assert not cal.is_trading_day("20261003")
    assert cal.next_trading_day("20261002") == "20261005"
    assert cal.night_session("2026-10-02", "SHFE") == ("20261002", "20261005")
    assert cal.metadata["sha256"] == cal.sha256
    override = dict(snapshot, version="user-r2", days=[{"date": "2026-10-03", "is_trading_day": True, "trading_day": "20261003"}])
    merged = cal.with_overrides(override)
    assert merged.is_trading_day("2026-10-03")
    assert merged.next_trading_day("2026-10-02") == "20261003"
    for bad in (
        dict(snapshot, version="main"),
        dict(snapshot, source=dict(snapshot["source"], scope="equities")),
        dict(snapshot, days=snapshot["days"] + [snapshot["days"][0]]),
        dict(snapshot, days=[dict(snapshot["days"][0], trading_day="20261003")]),
    ):
        try:
            TradingCalendar.from_json(bad)
            raise AssertionError("非法日历应拒绝")
        except CalendarError:
            pass
    try:
        cal.night_session("2026-10-02", "DCE")
        raise AssertionError("未显式供给的夜盘映射不应推断")
    except CalendarError:
        pass
    from unittest.mock import patch
    from ctpbuddy.sdk import Admin
    from ctpbuddy.cli import main as cli_main
    for lookup in (
        lambda: cal.is_trading_day("2026-10-06"),
        lambda: cal.next_trading_day("2026-10-05"),
        lambda: TradingCalendar.from_json('{"schema":1,"schema":2}'),
        lambda: TradingCalendar.from_json([]),
        lambda: TradingCalendar.from_json(dict(snapshot, days=[dict(snapshot["days"][0], date="2026-02-30")])),
        lambda: TradingCalendar.from_json(dict(snapshot, days=[dict(snapshot["days"][0], is_trading_day=1)])),
        lambda: TradingCalendar.from_json(dict(snapshot, source=dict(snapshot["source"], kind="github", revision="main"))),
        lambda: TradingCalendar.from_json(dict(snapshot, days=[dict(snapshot["days"][0], exchanges={"SHFE": {"night_trading_day": "20261005"}})])),
    ):
        try:
            lookup()
            raise AssertionError("无效输入或缺失覆盖必须拒绝")
        except CalendarError:
            pass
    # 覆盖自然日时同时移除旧夜盘，不能保留过期映射。
    removed = cal.with_overrides(dict(snapshot, version="remove-night-r1", days=[dict(snapshot["days"][0], exchanges={})]))
    try:
        removed.night_session("2026-10-02", "SHFE")
        raise AssertionError("旧夜盘应随整日覆盖删除")
    except CalendarError:
        pass
    with tempfile.TemporaryDirectory() as root:
        path = os.path.join(root, "calendar.json")
        with open(path, "w", encoding="utf-8") as fh:
            json.dump(snapshot, fh)
        assert TradingCalendar.from_file(path, cal.sha256).sha256 == cal.sha256
        assert cli_main(["calendar", "validate", path, "--sha256", cal.sha256]) == 0
        assert cli_main(["calendar", "validate", path, "--sha256", "bad"]) == 1
    with patch("ctpbuddy.sdk.admin.socket.create_connection"):
        admin = Admin(calendar=cal)
        with patch.object(admin, "cmd", return_value={"ok": True}) as cmd, patch.object(admin, "status", return_value={"playback": {"trading_day": "20261002"}}) as status:
            admin.settle_day({"rb": 1})
            cmd.assert_called_once_with("settle_day", settlement_prices={"rb": 1}, next_trading_day="20261005")
            cmd.reset_mock()
            status.reset_mock()
            admin.settle_day({"rb": 1}, "20261003")
            status.assert_not_called()
            cmd.assert_called_once_with("settle_day", settlement_prices={"rb": 1}, next_trading_day="20261003")
        admin.calendar = None
        try:
            admin.settle_day({})
            raise AssertionError("未配置日历时不能自动结算")
        except ValueError:
            pass
    sample_path = os.path.join(REPO, "calendar", "production", "shfe-2026-new-year.sample.json")
    sample = TradingCalendar.from_file(sample_path)
    assert sample.coverage == {"start": "2025-12-31", "end": "2026-01-05", "days": 6, "missing_days": 0, "night_records": 1}
    try:
        sample.night_session("2025-12-31", "SHFE")
        raise AssertionError("官方明确关闭的夜盘不能返回映射")
    except CalendarError as exc:
        assert "明确休市" in str(exc)
    assert cli_main(["calendar", "validate", sample_path]) == 0
    print("[ok] calendar: offline validation/hash/CLI, coverage, override, explicit nights and Admin compatibility")


def test_journal_hash() -> None:
    """Deterministic journal hashing (DESIGN §11.4, M2-3): ts_wall excluded,
    core projection drops noise types / re-encodes seq / normalizes the
    connection-numbering fields of session events."""
    from ctpbuddy.journal import (
        JournalError,
        canonical,
        core_normalize,
        count_types,
        fmt_vt,
        hash_stream,
        load_events,
        verify_events,
    )

    def ev(seq: int, vt: float, typ: str, data=None, **kw) -> dict:
        e = {
            "seq": seq,
            "ts_wall": "2026-10-02T09:30:00.000+08:00",
            "trading_day": "20261002",
            "vt_ms": vt,
            "type": typ,
            "data": data if data is not None else {},
        }
        e.update(kw)
        return e

    stream = [
        ev(1, 34200000.0, "server_start", {"version": "0.1.0", "scenario": "/tmp/x"}),
        ev(2, 34200000.0, "session_auth", {"front_id": 2, "app_id": "ctpbuddy-python"},
           broker="8888", investor="dsl001"),
        ev(3, 34200000.0, "session_login", {"front_id": 2, "session_id": 1},
           broker="8888", investor="dsl001"),
        ev(4, 34260000.0, "md_watermark", {"idx": 1}),
        ev(5, 34260000.0, "order_insert",
           {"order_ref": "D1A", "instrument": "rb2601", "limit_price": 3504.0},
           broker="8888", investor="dsl001"),
        ev(6, 34290000.0, "assertion", {"metric": "fills", "pass": True},
           broker="8888", investor="dsl001"),
    ]

    # ts_wall is the wall clock: never part of the hash
    bumped = [dict(e, ts_wall="2099-01-01T00:00:00.000+08:00") for e in stream]
    assert hash_stream(stream) == hash_stream(bumped)
    # any semantic change moves the full hash
    tampered = [dict(e) for e in stream]
    tampered[4] = dict(tampered[4], data=dict(tampered[4]["data"], limit_price=3505.0))
    assert hash_stream(stream) != hash_stream(tampered)

    # core projection: noise types dropped, seq re-encoded 1..n
    core = core_normalize(stream)
    assert [e["type"] for e in core] == [
        "session_auth", "session_login", "order_insert", "assertion"]
    assert [e["seq"] for e in core] == [1, 2, 3, 4]
    # interleaved noise events do not move the core hash (but do the full one)
    noisy = stream[:2] + [ev(99, 34230000.0, "md_watermark", {"idx": 0})] + stream[2:]
    assert hash_stream(noisy, core=True) == hash_stream(stream, core=True)
    assert hash_stream(noisy) != hash_stream(stream)

    # connection numbering is normalized: a replay whose connections were
    # opened in a different order still hashes equal on the core
    shifted = [dict(e) for e in stream]
    shifted[1] = dict(shifted[1], data=dict(shifted[1]["data"], front_id=9))
    shifted[2] = dict(shifted[2], data=dict(shifted[2]["data"], front_id=9))
    assert hash_stream(shifted, core=True) == hash_stream(stream, core=True)
    # a re-login on the same connection (session_id 2) IS semantic
    relogin = [dict(e) for e in stream]
    relogin[2] = dict(relogin[2], data=dict(relogin[2]["data"], session_id=2))
    assert hash_stream(relogin, core=True) != hash_stream(stream, core=True)

    # only/skip selectors apply on top
    assert hash_stream(stream, only=["order_insert"]) == hash_stream([stream[4]])
    assert hash_stream(stream, skip=["md_watermark"]) != hash_stream(stream)
    assert hash_stream(stream, only=["order_insert"], skip=["order_insert"]) == \
        hash_stream([])

    # structural verification
    assert verify_events(stream) == []
    broken = stream[:1] + [dict(stream[1], seq=7)] + stream[2:]
    assert verify_events(broken), "seq gap must be reported"
    unknown = stream[:1] + [dict(stream[1], type="bogus_type")] + stream[3:]
    assert any("bogus_type" in p for p in verify_events(unknown))
    missing = stream[:1] + [{k: v for k, v in stream[1].items() if k != "vt_ms"}] + stream[2:]
    assert any("vt_ms" in p for p in verify_events(missing))

    # canonical form: ts_wall-free, sorted keys, compact separators
    c = canonical(stream[1])
    assert "ts_wall" not in c, c
    assert c.index('"broker"') < c.index('"data"') < c.index('"seq"'), c
    assert ", " not in c and ": " not in c, c
    assert fmt_vt(34260000.0) == "09:31:00.000"
    assert fmt_vt("nope") == "nope"
    assert count_types(stream)["order_insert"] == 1

    # file/dir loading roundtrip + error paths
    with tempfile.TemporaryDirectory() as d:
        p = os.path.join(d, "20261002.jsonl")
        with open(p, "w", encoding="utf-8") as fh:
            for e in stream:
                fh.write(json.dumps(e, ensure_ascii=False) + "\n")
        assert load_events(p) == stream
        assert load_events(d) == stream  # directory form
        assert hash_stream(load_events(p)) == hash_stream(stream)
        with open(p, "a", encoding="utf-8") as fh:
            fh.write("{not json}\n")
        try:
            load_events(p)
            raise AssertionError("malformed line must raise")
        except JournalError as e:
            assert "20261002.jsonl:7" in str(e), str(e)
    print("[ok] journal: ts_wall exclusion, core projection + id normalization, "
          "selectors, verify, canonical form, load errors")


def test_journal_projection() -> None:
    from ctpbuddy.store import Projection, rebuild

    with tempfile.TemporaryDirectory() as d:
        journal = os.path.join(d, "journal")
        os.mkdir(journal)
        events = [
            {"seq": 1, "ts_wall": "2026-10-03T09:30:00+08:00", "trading_day": "20261003", "vt_ms": 1, "type": "session_login", "broker": "8888", "investor": "u1", "data": {}},
            {"seq": 2, "ts_wall": "2026-10-03T09:30:01+08:00", "trading_day": "20261003", "vt_ms": 2, "type": "order_insert", "broker": "8888", "investor": "u1", "data": {"order_ref": "r1", "order_sys_id": "0001", "instrument": "rb2601", "exchange": "SHFE", "direction": 0, "offset": 0, "limit_price": 3500, "volume": 2, "outcome": {"accepted": True}}},
            {"seq": 3, "ts_wall": "2026-10-03T09:30:02+08:00", "trading_day": "20261003", "vt_ms": 3, "type": "order_update", "broker": "8888", "investor": "u1", "data": {"order_ref": "r1", "order_sys_id": "0001", "status": "0", "volume_traded": 2, "volume_total": 0}},
            {"seq": 4, "ts_wall": "2026-10-03T09:30:02+08:00", "trading_day": "20261003", "vt_ms": 3, "type": "fill", "broker": "8888", "investor": "u1", "data": {"trade_id": "t1", "order_sys_id": "0001", "order_ref": "r1", "instrument": "rb2601", "direction": 0, "offset": 0, "price": 3500, "volume": 2}},
            {"seq": 5, "ts_wall": "2026-10-03T09:30:03+08:00", "trading_day": "20261003", "vt_ms": 4, "type": "settings_updated", "data": {"before": {"qry_freq": 1}, "after": {"qry_freq": 2}}},
        ]
        path = os.path.join(journal, "20261003.jsonl")
        with open(path, "w", encoding="utf-8") as fh:
            for event in events:
                fh.write(json.dumps(event, ensure_ascii=False) + "\n")
        db = os.path.join(d, "ctpbuddy.db")
        first = rebuild(journal, db)
        second = rebuild(journal, db)
        assert first == second
        with Projection(db) as projection:
            assert len(projection.query("account", investor="u1")) == 1
            order = projection.query("order_record", investor="u1")[0]
            assert order["status"] == "0" and order["volume_traded"] == 2
            assert projection.query("trade_record")[0]["trade_id"] == "t1"
            assert projection.query("position_change")[0]["volume_delta"] == 2
            assert projection.query("audit_log")[0]["action"] == "settings_updated"
        with open(db, "wb") as fh:
            fh.write(b"not a sqlite database")
        rebuild(journal, db)
        with Projection(db) as projection:
            assert projection.query("trade_record")[0]["trade_id"] == "t1"
    print("[ok] journal projection: schema/version, deterministic rebuild, recovery and queries")


def test_web_projection() -> None:
    from ctpbuddy.store import rebuild
    from ctpbuddy.web import make_server

    with tempfile.TemporaryDirectory() as d:
        journal = os.path.join(d, "journal")
        os.mkdir(journal)
        events = [
            {"seq": 1, "ts_wall": "2026-10-03T09:30:00+08:00", "trading_day": "20261003", "vt_ms": 1, "type": "session_login", "broker": "8888", "investor": "web1", "data": {}},
            {"seq": 2, "ts_wall": "2026-10-03T09:30:01+08:00", "trading_day": "20261003", "vt_ms": 2, "type": "order_insert", "broker": "8888", "investor": "web1", "data": {"order_ref": "w1", "order_sys_id": "1", "instrument": "rb2601", "exchange": "SHFE", "direction": 0, "offset": 0, "limit_price": 3500, "volume": 1, "outcome": {"accepted": True}}},
            {"seq": 3, "ts_wall": "2026-10-03T09:30:02+08:00", "trading_day": "20261003", "vt_ms": 3, "type": "fill", "broker": "8888", "investor": "web1", "data": {"trade_id": "t1", "order_sys_id": "1", "order_ref": "w1", "instrument": "rb2601", "direction": 0, "offset": 0, "price": 3500, "volume": 1}},
            {"seq": 4, "ts_wall": "2026-10-03T09:30:03+08:00", "trading_day": "20261003", "vt_ms": 4, "type": "settlement_report", "broker": "8888", "investor": "web1", "data": {"settlement_id": 1, "account_id": "web1", "currency_id": "CNY", "source": "user_supplied", "content_bytes": [65, 66]}},
            {"seq": 5, "ts_wall": "2026-10-03T09:30:04+08:00", "trading_day": "20261003", "vt_ms": 5, "type": "settings_updated", "data": {"after": {"qry_freq": 2}}},
        ]
        with open(os.path.join(journal, "20261003.jsonl"), "w", encoding="utf-8") as fh:
            for event in events:
                fh.write(json.dumps(event, ensure_ascii=False) + "\n")
        db = os.path.join(d, "projection.db")
        rebuild(journal, db)
        server = make_server("127.0.0.1", 0, "127.0.0.1:1", db)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            base = "http://127.0.0.1:%d" % server.server_address[1]
            host = "127.0.0.1:%d" % server.server_address[1]
            page = urllib.request.urlopen(urllib.request.Request(base + "/", headers={"Host": host})).read().decode()
            assert "SQLite 投影浏览" in page and "projectionTable" in page
            assert "innerHTML" not in open(os.path.join(REPO, "py", "ctpbuddy", "assets", "settings.js"), encoding="utf-8").read()
            for table, key in (("account", "investor_id"), ("order_record", "order_ref"), ("trade_record", "trade_id"), ("audit_log", "action"), ("settlement_report", "settlement_id")):
                request = urllib.request.Request(base + "/api/projection?table=" + table + "&trading_day=20261003", headers={"Host": host})
                payload = json.loads(urllib.request.urlopen(request).read())
                assert payload["rows"] and key in payload["rows"][0], (table, payload)
                assert payload["snapshot"]["event_count"] == "5" and payload["realtime"] is False, payload
            empty = urllib.request.Request(base + "/api/projection?table=account&broker=missing", headers={"Host": host})
            empty_payload = json.loads(urllib.request.urlopen(empty).read())
            assert empty_payload["rows"] == [] and empty_payload["realtime"] is False
            injected = urllib.request.Request(base + "/api/projection?table=account%20WHERE%201%3D1", headers={"Host": host})
            try:
                urllib.request.urlopen(injected)
                raise AssertionError("投影表名注入被接受")
            except urllib.error.HTTPError as exc:
                assert exc.code == 400

            broken = make_server("127.0.0.1", 0, "127.0.0.1:1", os.path.join(d, "missing.db"))
            broken_thread = threading.Thread(target=broken.serve_forever, daemon=True)
            broken_thread.start()
            try:
                broken_base = "http://127.0.0.1:%d" % broken.server_address[1]
                broken_host = "127.0.0.1:%d" % broken.server_address[1]
                request = urllib.request.Request(broken_base + "/api/projection", headers={"Host": broken_host})
                try:
                    urllib.request.urlopen(request)
                    raise AssertionError("缺失投影库未报错")
                except urllib.error.HTTPError as exc:
                    assert exc.code == 503
                    body = exc.read().decode()
                    assert d not in body and "missing.db" not in body, body
            finally:
                broken.shutdown()
                broken.server_close()
        finally:
            server.shutdown()
            server.server_close()
    print("[ok] web projection: account/order/trade/audit, table allowlist and path-safe errors")


def test_web_replay() -> None:
    from unittest.mock import patch
    from ctpbuddy.web import make_server

    class FakeAdmin:
        state = {"loaded": True, "idx": 0, "total": 2, "paused": True, "speed": 1.0, "looping": False,
                 "virtual_time": "09:30:00.000", "trading_day": "20261002"}
        calls = []

        def __init__(self, *args, **kwargs):
            pass

        def __enter__(self):
            return self

        def __exit__(self, *args):
            pass

        def status(self):
            return {"scenario": "web-fixture", "playback": dict(self.state)}

        def cmd(self, name, **kwargs):
            self.calls.append((name, kwargs))
            if name == "pause":
                self.state["paused"] = True
            elif name == "resume":
                self.state["paused"] = False
            elif name == "step":
                assert self.state["paused"]
                self.state["idx"] += 1
                self.state["virtual_time"] = "09:30:01.000"
            elif name == "set_speed":
                self.state["speed"] = kwargs["speed"]
            elif name == "loop":
                self.state["looping"] = kwargs["on"]
            return {"ok": True, "cmd": name}

    with patch("ctpbuddy.web.Admin", FakeAdmin):
        server = make_server("127.0.0.1", 0, "127.0.0.1:1")
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            base = "http://127.0.0.1:%d" % server.server_address[1]
            host = "127.0.0.1:%d" % server.server_address[1]
            def request(path, data=None, method="GET", token=None, extra=None):
                headers = {"Host": host}
                if data is not None:
                    raw = data if isinstance(data, bytes) else json.dumps(data).encode()
                    headers.update({"Origin": base, "Content-Type": "application/json", "Content-Length": str(len(raw)), "X-CSRF-Token": token or csrf})
                else:
                    raw = None
                headers.update(extra or {})
                return urllib.request.urlopen(urllib.request.Request(base + path, data=raw, method=method, headers=headers), timeout=5)
            csrf = json.loads(request("/api/session").read())["token"]
            status = json.loads(request("/api/replay/status").read())
            assert status["playback"]["idx"] == 0 and status["realtime"] is True
            pause = json.loads(request("/api/replay", {"cmd": "pause"}, "POST").read())
            assert pause["playback"]["paused"]
            step = json.loads(request("/api/replay", {"cmd": "step"}, "POST").read())
            assert step["playback"]["idx"] == 1 and step["playback"]["virtual_time"] == "09:30:01.000"
            speed = json.loads(request("/api/replay", {"cmd": "set_speed", "speed": 2.5}, "POST").read())
            assert speed["playback"]["speed"] == 2.5
            loop = json.loads(request("/api/replay", {"cmd": "loop", "on": True}, "POST").read())
            assert loop["playback"]["looping"]
            bad_commands = ({"cmd": "shutdown"}, {"cmd": "reset_account"}, {"cmd": "start_scenario", "path": "x"}, {"cmd": "settle_day"}, {"cmd": "seek"}, {"cmd": "pause", "extra": 1}, {"cmd": "set_speed"}, {"cmd": "set_speed", "speed": float("nan")}, {"cmd": "set_speed", "speed": float("inf")}, {"cmd": "set_speed", "speed": -1}, {"cmd": "set_speed", "speed": 1001}, {"cmd": "set_speed", "speed": True}, {"cmd": "set_speed", "speed": "2"}, {"cmd": "loop", "on": 1}, {"cmd": []}, [], b'{"cmd":"pause","cmd":"resume"}', b'{"cmd":"set_speed","speed":1e309}', b'{"cmd":"set_speed","speed":' + b'9'*350 + b'}')
            for bad in bad_commands:
                try:
                    request("/api/replay", bad, "POST")
                    raise AssertionError("非法回放命令被接受: %r" % (bad,))
                except urllib.error.HTTPError as exc:
                    assert exc.code == 400, (bad, exc.code)
            no_csrf = urllib.request.Request(base + "/api/replay", data=json.dumps({"cmd": "pause"}).encode(), method="POST", headers={"Host": host, "Origin": base, "Content-Type": "application/json"})
            try:
                urllib.request.urlopen(no_csrf)
                raise AssertionError("缺失 CSRF 被接受")
            except urllib.error.HTTPError as exc:
                assert exc.code == 403
            assert [name for name, _ in FakeAdmin.calls] == ["pause", "step", "set_speed", "loop"]
            def rejected(path, data, expected, extra=None):
                count = len(FakeAdmin.calls)
                try:
                    request(path, data, "POST" if data is not None else "GET", extra=extra)
                    raise AssertionError("非法 HTTP 请求被接受")
                except urllib.error.HTTPError as exc:
                    assert exc.code == expected, (path, exc.code, expected)
                assert len(FakeAdmin.calls) == count
            for extra in ({"Host": "evil.invalid"}, {"Origin": "http://evil.invalid"}, {"Origin": "null"}, {"Sec-Fetch-Site": "cross-site"}, {"X-CSRF-Token": "wrong"}):
                rejected("/api/replay", {"cmd": "pause"}, 403, extra)
            rejected("/api/replay/status", None, 403, {"Host": "evil.invalid"})
            rejected("/api/replay/status?cmd=shutdown", None, 400)
            rejected("/api/replay?cmd=pause", {"cmd": "pause"}, 404)
            rejected("/api/replay", {"cmd": "pause"}, 415, {"Content-Type": "text/plain"})
            rejected("/api/replay", b"x" * 8193, 413)
            rejected("/api/replay", {"cmd": "pause"}, 400, {"Transfer-Encoding": "chunked"})
            import http.client
            for duplicate in ("Host", "Origin", "X-CSRF-Token", "Content-Length"):
                connection = http.client.HTTPConnection("127.0.0.1", server.server_address[1], timeout=5)
                raw = b'{"cmd":"pause"}'
                values = {"Host": host, "Origin": base, "X-CSRF-Token": csrf, "Content-Length": str(len(raw)), "Content-Type": "application/json"}
                connection.putrequest("POST", "/api/replay", skip_host=True)
                for key, value in values.items():
                    connection.putheader(key, value)
                    if key == duplicate:
                        connection.putheader(key, value)
                connection.endheaders(raw)
                response = connection.getresponse()
                assert response.status == (400 if duplicate == "Content-Length" else 403)
                response.read()
                connection.close()
            FakeAdmin.state["loaded"] = False
            rejected("/api/replay", {"cmd": "pause"}, 409)
            FakeAdmin.state.update(loaded=True, idx=2)
            assert json.loads(request("/api/replay/status").read())["playback"]["finished"]
            rejected("/api/replay", {"cmd": "resume"}, 409)
            FakeAdmin.state.update(idx=1, paused=False)
            rejected("/api/replay", {"cmd": "step"}, 409)
            assert not json.loads(request("/api/replay", {"cmd": "resume"}, "POST").read())["playback"]["paused"]
            for speed in (0, 1000):
                assert json.loads(request("/api/replay", {"cmd": "set_speed", "speed": speed}, "POST").read())["playback"]["speed"] == speed
            with patch("ctpbuddy.web.Admin", side_effect=ConnectionRefusedError("offline")):
                rejected("/api/replay/status", None, 503)
            with patch.object(FakeAdmin, "cmd", side_effect=RuntimeError("核心拒绝")):
                rejected("/api/replay", {"cmd": "pause"}, 409)
            from concurrent.futures import ThreadPoolExecutor
            active = threading.Condition()
            release = threading.Event()
            entered = 0
            original_status = FakeAdmin.status
            def blocked_status(client):
                nonlocal entered
                with active:
                    entered += 1
                    active.notify_all()
                assert release.wait(5)
                return original_status(client)
            with patch.object(FakeAdmin, "status", blocked_status), ThreadPoolExecutor(max_workers=8) as pool:
                futures = [pool.submit(lambda: request("/api/replay/status").read()) for _ in range(8)]
                try:
                    with active:
                        assert active.wait_for(lambda: entered == 8, timeout=5)
                    rejected("/api/replay/status", None, 429)
                finally:
                    release.set()
                assert all(json.loads(future.result())["realtime"] for future in futures)
            assert json.loads(request("/api/replay/status").read())["realtime"]
            js = open(os.path.join(REPO, "py", "ctpbuddy", "assets", "settings.js"), encoding="utf-8").read()
            assert "innerHTML" not in js and "textContent" in js
        finally:
            server.shutdown()
            server.server_close()
    print("[ok] web replay: real HTTP boundary, strict commands, CSRF, status/pause/step/speed/loop")


def test_assertions_cli() -> None:
    from unittest.mock import patch
    from ctpbuddy.cli import main as cli_main

    def check(summary, expected, *extra):
        with patch("ctpbuddy.cli.Admin") as admin:
            admin.return_value.__enter__.return_value.status.return_value = {
                "assertions": summary,
            }
            assert cli_main(["assertions", "check", *extra]) == expected

    check(None, 1)
    check({"total": 0}, 1)
    check({"total": 2, "evaluated": 1, "passed": 1, "failed": 0}, 1)
    check({"total": 2, "evaluated": 2, "passed": 1, "failed": 1}, 1)
    passed = {"total": 2, "evaluated": 2, "passed": 2, "failed": 0}
    check(passed, 0, "--total", "2")
    check(passed, 1, "--total", "3")
    with patch("ctpbuddy.cli.Admin", side_effect=ConnectionRefusedError("offline")):
        assert cli_main(["assertions", "check"]) == 1
    print("[ok] assertions CLI: pass/fail/pending/empty/count/offline exit codes")


def test_shim_install_security() -> None:
    from ctpbuddy.cli import main as cli_main
    from ctpbuddy.shim_install import ShimInstallError, install_shim, restore_shim

    with tempfile.TemporaryDirectory() as root:
        target = os.path.join(root, "target")
        shim = os.path.join(root, "shim")
        os.mkdir(target)
        os.mkdir(shim)
        for name, data in (("thosttraderapi_se.dll", b"shim-td-v1"), ("thostmduserapi_se.dll", b"shim-md-v1")):
            with open(os.path.join(shim, name), "wb") as fh:
                fh.write(data)
        with open(os.path.join(shim, "manifest.json"), "w", encoding="utf-8") as fh:
            json.dump({"version": "fixture-1", "architecture": "x64", "artifacts": [
                {"name": "thosttraderapi_se.dll"}, {"name": "thostmduserapi_se.dll"}]}, fh)
        with open(os.path.join(target, "thosttraderapi_se.dll"), "wb") as fh:
            fh.write(b"original-td")

        assert cli_main(["install-shim", "--target-dir", target, "--shim-dir", shim]) == 0
        assert open(os.path.join(target, "thosttraderapi_se.dll"), "rb").read() == b"original-td"
        result = install_shim(target, shim, apply=True)
        assert result["metadata"]["version"] == "fixture-1"
        assert open(os.path.join(target, "thostmduserapi_se.dll"), "rb").read() == b"shim-md-v1"
        assert os.path.isdir(result["backup_dir"])
        assert os.path.isfile(result["restore_script"])
        assert cli_main(["restore", "--target-dir", target]) == 0
        assert open(os.path.join(target, "thosttraderapi_se.dll"), "rb").read() == b"original-td"
        assert not os.path.exists(os.path.join(target, "thostmduserapi_se.dll"))

        # A changed installed file must not be overwritten during restore.
        install_shim(target, shim, apply=True)
        with open(os.path.join(target, "thostmduserapi_se.dll"), "ab") as fh:
            fh.write(b"changed")
        try:
            restore_shim(target)
            raise AssertionError("sha 变化后恢复应被拒绝")
        except ShimInstallError as exc:
            assert "SHA256" in str(exc)

        unknown = os.path.join(shim, "unknown.dll")
        open(unknown, "wb").close()
        with open(os.path.join(shim, "manifest.json"), "w", encoding="utf-8") as fh:
            json.dump({"artifacts": ["unknown.dll"]}, fh)
        os.mkdir(os.path.join(root, "other-target"))
        try:
            install_shim(os.path.join(root, "other-target"), shim)
            raise AssertionError("未知 DLL 应被拒绝")
        except ShimInstallError as exc:
            assert "不受支持" in str(exc)

        # Source/target overlap and traversal-like manifest entries are rejected.
        try:
            install_shim(target, target)
            raise AssertionError("重合路径应被拒绝")
        except ShimInstallError as exc:
            assert "重合" in str(exc)
        with open(os.path.join(shim, "manifest.json"), "w", encoding="utf-8") as fh:
            json.dump({"artifacts": ["../thosttraderapi_se.dll"]}, fh)
        try:
            install_shim(os.path.join(root, "other-target"), shim)
            raise AssertionError("路径穿越应被拒绝")
        except ShimInstallError as exc:
            assert "不受支持" in str(exc)
    print("[ok] shim install: dry-run/apply, allowlist, backup/restore, sha guard, overlap/traversal")


def test_shim_install_faults() -> None:
    from unittest.mock import patch
    from ctpbuddy.shim_install import ShimInstallError, install_shim, restore_shim

    def fixture(root):
        target = os.path.join(root, "target")
        shim = os.path.join(root, "shim")
        os.mkdir(target)
        os.mkdir(shim)
        for name, data in (("thosttraderapi_se.dll", b"td"), ("thostmduserapi_se.dll", b"md")):
            with open(os.path.join(shim, name), "wb") as fh:
                fh.write(data)
        with open(os.path.join(shim, "manifest.json"), "w", encoding="utf-8") as fh:
            json.dump({"version": "fault-fixture", "artifacts": list((
                {"name": "thosttraderapi_se.dll"}, {"name": "thostmduserapi_se.dll"}))}, fh)
        for name, data in (("thosttraderapi_se.dll", b"original-td"), ("thostmduserapi_se.dll", b"original-md")):
            with open(os.path.join(target, name), "wb") as fh:
                fh.write(data)
        return target, shim

    with tempfile.TemporaryDirectory() as root:
        target, shim = fixture(root)
        script = os.path.join(target, "restore_shim.py")
        open(script, "w", encoding="utf-8").close()
        try:
            install_shim(target, shim, apply=True)
            raise AssertionError("已有恢复脚本不得覆盖")
        except ShimInstallError as exc:
            assert "拒绝覆盖" in str(exc)
        os.unlink(script)

        from ctpbuddy import shim_install as shim_mod
        real_copy_atomic = shim_mod._copy_atomic
        def fail_second(source, destination):
            if os.path.basename(os.fspath(destination)) == "thostmduserapi_se.dll":
                raise OSError("injected second replacement failure")
            return real_copy_atomic(source, destination)
        with patch("ctpbuddy.shim_install._copy_atomic", side_effect=fail_second):
            try:
                install_shim(target, shim, apply=True)
                raise AssertionError("第二文件失败应抛错")
            except ShimInstallError:
                pass
        assert open(os.path.join(target, "thosttraderapi_se.dll"), "rb").read() == b"td"
        assert os.path.exists(os.path.join(target, ".ctpbuddy-shim-install.json"))
        restore_shim(target)
        assert open(os.path.join(target, "thosttraderapi_se.dll"), "rb").read() == b"original-td"
        assert open(os.path.join(target, "thostmduserapi_se.dll"), "rb").read() == b"original-md"
        install_shim(target, shim, apply=True)
        restore_shim(target)
        install_shim(target, shim, apply=True)
        with open(os.path.join(target, ".ctpbuddy-shim-install.json"), "r", encoding="utf-8") as fh:
            backup = json.load(fh)["backup_dir"]
        with open(os.path.join(backup, "thostmduserapi_se.dll.backup"), "ab") as fh:
            fh.write(b"corrupt")
        try:
            restore_shim(target)
            raise AssertionError("损坏第二备份应拒绝且不改第一文件")
        except ShimInstallError as exc:
            assert "备份文件" in str(exc)
        assert open(os.path.join(target, "thosttraderapi_se.dll"), "rb").read() == b"td"
        restore_dir = os.path.join(target, ".ctpbuddy-backup")
        assert os.path.exists(restore_dir)
        # 重建临时 fixture，验证正常恢复后可再次安装。
        with tempfile.TemporaryDirectory() as retry_root:
            retry_target, retry_shim = fixture(retry_root)
            install_shim(retry_target, retry_shim, apply=True)
            restore_shim(retry_target)
            install_shim(retry_target, retry_shim, apply=True)
            restore_shim(retry_target)

    with tempfile.TemporaryDirectory() as root:
        target, shim = fixture(root)
        marker = os.path.join(target, ".ctpbuddy-shim-install.json")
        open(marker, "w", encoding="utf-8").close()
        try:
            install_shim(target, shim, apply=True)
            raise AssertionError("已有 marker 不得覆盖")
        except ShimInstallError as exc:
            assert "已有" in str(exc)
        os.unlink(marker)
        try:
            os.symlink(os.path.join(root, "outside.dll"), os.path.join(target, "thostmduserapi_se.dll"))
        except (OSError, NotImplementedError):
            pass
        else:
            try:
                install_shim(target, shim, apply=True)
                raise AssertionError("目标 symlink 应拒绝")
            except ShimInstallError as exc:
                assert "符号链接" in str(exc)
    print("[ok] shim install faults: second replace, pre-state restore, marker/script lock, links, reinstall")


def test_sdk_lifecycle():
    from ctpbuddy.sdk.client import Client, CTPError
    from ctpbuddy.wire import Frame, PING, PONG

    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen()
        with Client("127.0.0.1:%d" % listener.getsockname()[1], timeout=0.1) as client:
            peer, _ = listener.accept()
            with peer:
                time.sleep(0.25)
                assert client._reader.is_alive()
                def answer():
                    request = Frame.decode_from(peer)
                    assert request.msg_type == PING
                    peer.sendall(Frame(PONG, request.req_id, b"").encode())
                responder = threading.Thread(target=answer)
                responder.start()
                client.ping()
                responder.join(timeout=1)
                assert not responder.is_alive()
            client._reader.join(timeout=1)
            assert not client._reader.is_alive() and client._closed
            for request in (client.ping, lambda: client._query_once(PING, PONG, b"")):
                try:
                    request()
                    raise AssertionError("closed client accepted request")
                except CTPError as exc:
                    assert exc.error_id == -1
        with Client("127.0.0.1:%d" % listener.getsockname()[1]) as client:
            peer, _ = listener.accept()
            outcome = []
            def pending_request():
                try:
                    client.ping()
                except CTPError as exc:
                    outcome.append(exc.error_id)
            requester = threading.Thread(target=pending_request)
            requester.start()
            with peer:
                assert Frame.decode_from(peer).msg_type == PING
                client.close()
            requester.join(timeout=1)
            client._reader.join(timeout=1)
            assert not requester.is_alive() and outcome == [-1]
    print("[ok] SDK: idle connection, EOF, pending close and closed requests")


def main() -> int:
    test_sdk_lifecycle()
    test_shim_install_security()
    test_shim_install_faults()
    test_assertions_cli()
    test_struct_layout()
    test_frames()
    test_scenario()
    test_dsl_spec()
    test_dsl_errors()
    test_dsl_compile()
    test_calendar()
    test_journal_hash()
    test_journal_projection()
    test_web_projection()
    test_web_replay()
    print("\nPY UNIT: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
