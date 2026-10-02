#!/usr/bin/env python
"""Python-side unit checks (stdlib only, no pytest needed).

Covers the pieces the e2e smoke depends on:
- generated struct layout (must match ctypes-verified sizes at codegen time)
- frame codec roundtrip incl. split writes and clean EOF
- canonical CSV scenario roundtrip + validation

Run: python tests/test_py.py
"""
from __future__ import annotations

import os
import socket
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(REPO, "py"))

from ctpbuddy.generated import structs  # noqa: E402
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


def main() -> int:
    test_struct_layout()
    test_frames()
    test_scenario()
    print("\nPY UNIT: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
