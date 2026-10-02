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
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "py"))

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
        InstrumentID="rb2610",
        LimitPrice=3502.0,
        VolumeTotalOriginal=3,
        Direction="0",
        CombOffsetFlag="0",
    )
    f = structs.unpack("CThostFtdcInputOrderField", buf)
    assert f["BrokerID"] == "8888" and f["InvestorID"] == "inv001", f
    assert f["InstrumentID"] == "rb2610" and f["LimitPrice"] == 3502.0, f
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
            instrument="rb2610", exchange="SHFE", trading_day="20261002",
            update_time="09:30:00", update_millisec="0", last_price="3500",
            volume="10", upper="3850", lower="3150",
            bid1="3498", ask1="3502", bidvol1="5", askvol1="5",
        )
        write_canonical(d, [row])
        assert validate_scenario(d) == [], validate_scenario(d)
        ticks = list(iter_ticks(d))
        assert len(ticks) == 1 and ticks[0]["instrument"] == "rb2610", ticks
        assert ticks[0]["bid1"] == "3498" and ticks[0]["ask1"] == "3502", ticks[0]
        # broken scenarios are rejected
        with open(os.path.join(d, "ticks.csv"), "a", encoding="utf-8") as fh:
            fh.write("rb2610,SHFE,20261002,09:30:01,0\n")
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


def main() -> int:
    test_struct_layout()
    test_frames()
    test_scenario()
    test_dsl_spec()
    test_dsl_errors()
    test_dsl_compile()
    print("\nPY UNIT: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
