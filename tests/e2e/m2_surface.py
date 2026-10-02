#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""M2 #42 push-surface end-to-end: 错单响应 / 错单回报 halves (DESIGN §8.12).

Real CTP splits an order rejection between two SPI callbacks by layer:
- the front office (报盘机) refuses -> the pending request itself answers
  `OnRspOrderInsert`/`OnRspOrderAction` with pInputOrder NULL + pRspInfo
  (wire: RSP_ERROR). No status return follows.
- the exchange refuses *after* the front office accepted ->
  `OnRspOrderInsert`{0} first, then `OnErrRtnOrderInsert` with the client's
  own input struct (wire: ERR_RTN_ORDER_INSERT).
- a cancel refusal pushes BOTH halves (official 报单回调规则 场景 6/7:
  OnRspOrderAction then OnErrRtnOrderAction).

This suite pins every half against the real core:
  A. front-office insert refuse (116 freq / 31 funds / 16 unknown contract):
     CTPError on the request, kind RSP_ERROR, and NO late ERR_RTN frame.
  B. exchange insert refuse (165 non-minimum-tick): the request SUCCEEDS and
     the refusal lands afterwards on OnErrRtnOrderInsert, echoing the client's
     InputOrderField.
  C. cancel refuse (25 order-not-found): both halves, response first, and the
     ERR_RTN_ORDER_ACTION echoes the client's InputOrderActionField.

Journal recording is unchanged (core side); m2_journal proves the 163 path.

Usage:
    python tests/e2e/m2_surface.py
Env:
    CTPBUDDY_CORE   explicit path to the ctpbuddy-server binary
"""
from __future__ import annotations

import os
import socket
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)  # reuse the smoke harness helpers
sys.path.insert(0, os.path.join(REPO, "py"))

from m1_smoke import (  # noqa: E402
    BROKER,
    INITIAL_FUNDS,
    INSTRUMENT,
    INVESTOR,
    find_core,
    free_port,
    make_scenario,
    wait_port,
)

from ctpbuddy import generated  # noqa: E402
from ctpbuddy.sdk import Admin, Client, CTPError  # noqa: E402
from ctpbuddy.sdk.client import rsp_info_of  # noqa: E402
from ctpbuddy.wire import (  # noqa: E402
    ERR_RTN_ORDER_ACTION,
    ERR_RTN_ORDER_INSERT,
    RSP_ORDER_INSERT,
)

RB = INSTRUMENT  # rb2601: SHFE, mult 10, tick 1, limits 3150..3850
ERR_FREQ = 116       # ORDER_FREQ_LIMIT      CTP:下单频率限制
ERR_FUNDS = 31       # INSUFFICIENT_MONEY    CTP:资金不足
ERR_UNKNOWN_INSTR = 16   # INSTRUMENT_NOT_FOUND CTP:找不到合约
ERR_PRICE_TICK = 165  # PRICE_WRONG_TICK      CTP:报单价格非最小变动价位整数倍
ERR_NO_ORDER = 25     # ORDER_NOT_FOUND       CTP:撤单找不到相应报单


def input_field_of(frame, struct: str) -> dict:
    """The client's own input struct off an ERR_RTN_* payload (input ++ rsp)."""
    n = generated.SIZES[struct]
    return generated.unpack(struct, frame.payload[:n])


def no_late(cli: Client, msg_type: int, why: str) -> None:
    """A front-office refusal must NOT also push the 错单回报 half."""
    late = cli.wait_late(msg_type, timeout=0.4)
    assert late is None, "%s: unexpected half-surface frame %r" % (why, late)


def new_window() -> None:
    """Let the front-office order gate roll over so the next section starts
    with a full per-second budget (it is a wall-clock window, §8.3)."""
    time.sleep(1.05)


def main() -> int:
    core = find_core()
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-m2surf-")
    scenario = os.path.join(tmp, "scenario")
    os.makedirs(scenario)
    make_scenario(scenario)
    data_dir = os.path.join(tmp, "data")
    td_port, admin_port = free_port(), free_port()

    # --order-freq 2: the front-office per-second order budget is lowered so
    # three back-to-back inserts bust it. Real CTP's ceiling is a per-second
    # counter too (here quoted as 6/s on the legacy counter, 20/s default),
    # so this only scales the window, it does not invent a new rejection.
    proc = subprocess.Popen(
        [
            core,
            "--td", "127.0.0.1:%d" % td_port,
            "--admin", "127.0.0.1:%d" % admin_port,
            "--broker-id", BROKER,
            "--speed", "0",
            "--order-freq", "2",
            "--initial-funds", str(INITIAL_FUNDS),
            "--data-dir", data_dir,
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        run(td_port, admin_port, scenario)
        print("\nM2 SURFACE: PASS")
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


def run(td_port: int, admin_port: int, scenario: str) -> None:
    wait_port(admin_port)
    admin = Admin("127.0.0.1:%d" % admin_port)
    admin.start_scenario(scenario, paused=True)  # static checks need no tick

    wait_port(td_port)
    with Client("127.0.0.1:%d" % td_port) as cli:
        cli.auth(BROKER, INVESTOR)
        login = cli.login(BROKER, INVESTOR, password="")
        assert login["FrontID"] > 0 and login["SessionID"] > 0, login
        cli.settle_confirm()

        # -- A1. insert frequency gate -> front-office half only -------------
        cli.clear_late()
        for ref in ("S1", "S2"):
            cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3480.0,
                             exchange="SHFE", order_ref=ref)
        try:
            cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=3480.0,
                             exchange="SHFE", order_ref="S3")
            raise AssertionError("3rd insert must bust the 1s budget")
        except CTPError as e:
            assert e.error_id == ERR_FREQ, e
            assert e.kind == "RSP_ERROR", e   # OnRspOrderInsert, not OnErrRtn
            assert e.msg == "CTP:下单频率限制", e
        no_late(cli, ERR_RTN_ORDER_INSERT, "116")
        print("[ok] insert 116: front-office RSP_ERROR only, no 错单回报 half")

        # -- A2. funds refusal -> front-office half only ---------------------
        new_window()
        cli.clear_late()
        cli.order_insert(RB, direction="0", offset="0", volume=300, limit_price=3502.0,
                         exchange="SHFE", order_ref="F1")
        try:
            cli.order_insert(RB, direction="0", offset="0", volume=300, limit_price=3502.0,
                             exchange="SHFE", order_ref="F2")
            raise AssertionError("second 300-lot order must exceed funds")
        except CTPError as e:
            assert e.error_id == ERR_FUNDS, e
            assert e.kind == "RSP_ERROR", e
            assert e.msg == "CTP:资金不足", e
        no_late(cli, ERR_RTN_ORDER_INSERT, "31")
        print("[ok] insert 31 (funds): front-office RSP_ERROR only, no 错单回报 half")

        # -- A3. unknown contract -> front-office half only ------------------
        new_window()
        cli.clear_late()
        try:
            cli.order_insert("NOPE9999", direction="0", offset="0", volume=1,
                             limit_price=100.0, exchange="SHFE", order_ref="U1")
            raise AssertionError("unknown instrument accepted")
        except CTPError as e:
            assert e.error_id == ERR_UNKNOWN_INSTR, e
            assert e.kind == "RSP_ERROR", e
        no_late(cli, ERR_RTN_ORDER_INSERT, "16")
        print("[ok] insert 16 (unknown contract): front-office RSP_ERROR only")

        # -- B. exchange refuse (非最小变动价位): success Rsp THEN rtn --------
        new_window()
        cli.clear_late()
        off_tick = 3497.5  # rb2601 tick = 1: 3497.5 is not a tick multiple
        f = cli.order_insert(RB, direction="0", offset="0", volume=1, limit_price=off_tick,
                             exchange="SHFE", order_ref="X1")
        assert f.msg_type == RSP_ORDER_INSERT, f  # OnRspOrderInsert{0}: accepted
        late = cli.wait_late(ERR_RTN_ORDER_INSERT)
        assert late is not None, "165 refusal must arrive on OnErrRtnOrderInsert"
        info = rsp_info_of(late.payload)
        assert info["ErrorID"] == ERR_PRICE_TICK, info
        assert info["ErrorMsg"] == "CTP:报单价格非最小变动价位整数倍", info
        echoed = input_field_of(late, "CThostFtdcInputOrderField")
        assert echoed["OrderRef"] == "X1", echoed      # the client's own struct
        assert abs(echoed["LimitPrice"] - off_tick) < 1e-9, echoed
        print("[ok] insert 165: OnRspOrderInsert{0} then OnErrRtnOrderInsert "
              "(echoes OrderRef X1)")

        # -- C. cancel refuse: BOTH halves, response first -------------------
        new_window()
        cli.clear_late()
        try:
            cli.order_action(RB, order_ref="NOPE7777")
            raise AssertionError("cancel of an unknown order must be refused")
        except CTPError as e:
            assert e.error_id == ERR_NO_ORDER, e
            assert e.kind == "RSP_ERROR", e   # OnRspOrderAction first
            assert e.msg == "CTP:撤单找不到相应报单", e
        late = cli.wait_late(ERR_RTN_ORDER_ACTION)
        assert late is not None, "cancel refusal must also push OnErrRtnOrderAction"
        info = rsp_info_of(late.payload)
        assert info["ErrorID"] == ERR_NO_ORDER, info
        assert info["ErrorMsg"] == "CTP:撤单找不到相应报单", info
        echoed = input_field_of(late, "CThostFtdcInputOrderActionField")
        assert echoed["OrderRef"] == "NOPE7777", echoed
        assert echoed["InstrumentID"] == RB, echoed
        print("[ok] cancel 25: OnRspOrderAction then OnErrRtnOrderAction "
              "(echoes OrderRef NOPE7777)")


if __name__ == "__main__":
    sys.exit(main())
