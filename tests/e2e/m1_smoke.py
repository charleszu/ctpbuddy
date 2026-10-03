#!/usr/bin/env python
"""M1 end-to-end smoke: scenario -> login -> crossing fill -> close -> cancel.

Runs the real Rust `ctpbuddy-server` binary against the real wire protocol
(no shim involved yet): admin control plane, auth/login/settle, market-data
subscription, immediate-fill order flow, resting order + cancel, query
projections, journal verification.

Usage:
    python tests/e2e/m1_smoke.py
Env:
    CTPBUDDY_CORE   explicit path to the ctpbuddy-server binary
"""
from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import tempfile
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "py"))

from ctpbuddy.sdk import Admin, CTPError, Client  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402
from ctpbuddy.wire import RTN_DEPTH_MD, RTN_ORDER, RTN_TRADE  # noqa: E402

BROKER = "8888"
INVESTOR = "smoke001"
INSTRUMENT = "rb2601"
INITIAL_FUNDS = 2_000_000.0

# rb2601: SHFE, mult 10, tick 1, margin 0.16 (真实合约，随包 refdata 快照)
BUY_PRICE = 3502.0   # = ask1: crosses on insert
SELL_PRICE = 3498.0  # = bid1: crosses on insert
REST_PRICE = 3480.0  # deep below the book: parks
# The bundled ref-data snapshot ships no commission table (a made-up fee
# would be worse than none — see `Catalog::bundled`), so fills are free and
# this suite asserts the fee-free path end to end. A desk supplying
# `commission_rates.jsonl` gets the full 开仓/平昨/平今 split instead.
COMM_BUY = 0.0
COMM_SELL = 0.0
# 保证金 = (ByVolume + ByMoney x Price x Mult) x Volume, priced at 昨结算价
# because the bundled MarginPriceType is '1' (notes/04 C2). Company rate
# 0.16 comes from refdata/margin_rates.jsonl, not from the exchange-rate
# fallback, so this number is the one a client cross-checks against
# ReqQryInstrumentMarginRate.
PRE_SETTLEMENT = 3500.0
MARGIN_PER_LOT = PRE_SETTLEMENT * 10 * 0.16   # 5600.0
MARGIN_OPEN = MARGIN_PER_LOT
LAST_TICK_PRICE = 3501.0                        # scenario's final tick last price
# CTP Balance is dynamic equity: while the position is open it carries the
# unrealized loss against the last tick (3501 vs the 3502 entry).
UNREALIZED_OPEN = (LAST_TICK_PRICE - BUY_PRICE) * 10  # -10.0


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


def make_scenario(dirpath: str) -> None:
    rows = []
    for i, (t, ms) in enumerate((("09:30:00", 0), ("09:30:00", 500), ("09:31:00", 0))):
        row = {c: "" for c in CANONICAL_COLUMNS}
        row.update(
            instrument=INSTRUMENT,
            exchange="SHFE",
            trading_day="20261002",
            update_time=t,
            update_millisec=str(ms),
            last_price="3500" if i < 2 else "3501",
            volume=str(100 + i),
            turnover=str(350000 + i),
            open_interest="5000",
            pre_settlement="3500",
            settlement="3500",
            pre_close="3498",
            open="3499",
            high="3505",
            low="3497",
            close="3501",
            upper="3850",
            lower="3150",
            pre_open_interest="4980",
            average="3500.25",
            bid1="3498", bid2="3497", bid3="3496", bid4="3495", bid5="3494",
            ask1="3502", ask2="3503", ask3="3504", ask4="3505", ask5="3506",
            bidvol1="10", bidvol2="8", bidvol3="6", bidvol4="4", bidvol5="2",
            askvol1="12", askvol2="10", askvol3="8", askvol4="6", askvol5="4",
        )
        rows.append(row)
    write_canonical(dirpath, rows)


def close(a: float, b: float, tol: float = 1e-6) -> bool:
    return abs(a - b) <= tol


def main() -> int:
    core = find_core()
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-smoke-")
    scenario = os.path.join(tmp, "scenario")
    os.makedirs(scenario)
    make_scenario(scenario)
    data_dir = os.path.join(tmp, "data")
    td_port, admin_port = free_port(), free_port()

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
        run_smoke(td_port, admin_port, data_dir, scenario)
        print("\nM1 SMOKE: PASS")
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


def run_smoke(td_port: int, admin_port: int, data_dir: str, scenario: str) -> None:
    # -- admin control plane ------------------------------------------------
    wait_port(admin_port)
    admin = Admin("127.0.0.1:%d" % admin_port)
    pong = admin.ping()
    assert pong["cmd"] == "ping", pong
    st = admin.status()
    assert st["broker_id"] == BROKER, st
    # builtin catalog: SHFE×3 / DCE / CFFEX / CZCE / GFEX (the last two exist
    # so the per-exchange FAK report layouts of 官方场景 8/9/10 are drivable)
    assert st["instruments"] == 789, st  # bundled ref data snapshot (all six CTP groups)
    assert st["playback"]["loaded"] is False, st
    print("[ok] admin ping/status (broker %s, %d instruments, no scenario)" % (st["broker_id"], st["instruments"]))

    # load paused so the subscription is in place before the first tick
    started = admin.start_scenario(scenario, paused=True)
    assert started["ticks"] == 3 and started["paused"] is True, started
    assert started["trading_day"] == "20261002", started
    print("[ok] admin start_scenario: %d ticks, day %s (paused)" % (started["ticks"], started["trading_day"]))

    # -- session -------------------------------------------------------------
    wait_port(td_port)
    with Client("127.0.0.1:%d" % td_port) as cli:
        v = cli.auth(BROKER, INVESTOR)
        assert v["broker_id"] == BROKER, v
        # 绕过 Shim，直接验证服务端拒绝认证身份与登录身份不一致。
        for broker, user in ((BROKER, "other-user"), ("9999", INVESTOR)):
            try:
                cli.login(broker, user, password="")
                raise AssertionError("Auth A -> Login B was accepted")
            except CTPError as e:
                assert e.error_id == 15, e
        print("[ok] AUTH/login BrokerID/UserID mismatch rejected")
        login = cli.login(BROKER, INVESTOR, password="")
        assert login["FrontID"] > 0 and login["SessionID"] > 0, login
        assert login["MaxOrderRef"] == "0", login
        cli.settle_confirm()
        print("[ok] auth/login/settle (front %d session %d)" % (login["FrontID"], login["SessionID"]))

        # auth must reject a foreign broker id
        bad = Client("127.0.0.1:%d" % td_port)
        try:
            bad.auth("9999", INVESTOR)
            raise AssertionError("foreign broker accepted")
        except CTPError as e:
            assert e.kind == "AUTH", e
        finally:
            bad.close()
        print("[ok] foreign BrokerID rejected at AUTH")

        # -- initial account -------------------------------------------------
        acct = cli.qry_trading_account()
        assert close(acct["Balance"], INITIAL_FUNDS), acct
        assert close(acct["Available"], INITIAL_FUNDS), acct
        assert close(acct["CurrMargin"], 0.0), acct
        print("[ok] initial account: balance=%.2f available=%.2f" % (acct["Balance"], acct["Available"]))

        instruments = cli.qry_instrument(INSTRUMENT)
        assert len(instruments) == 1 and instruments[0]["InstrumentID"] == INSTRUMENT, instruments
        assert instruments[0]["VolumeMultiple"] == 10, instruments[0]
        print("[ok] qry instrument: %s mult=%d" % (instruments[0]["InstrumentID"], instruments[0]["VolumeMultiple"]))

        # -- market data ------------------------------------------------------
        cli.subscribe([INSTRUMENT])
        admin.resume()
        deadline = time.time() + 10
        while time.time() < deadline:
            pb = admin.status()["playback"]
            if pb["total"] > 0 and pb["idx"] >= pb["total"]:
                break
            time.sleep(0.05)
        else:
            raise AssertionError("scenario playback did not finish")
        md = None
        for kind, field in cli.events(timeout=5.0):
            if kind == RTN_DEPTH_MD:
                md = field
                break
        assert md is not None and md["InstrumentID"] == INSTRUMENT, md
        assert close(md["BidPrice1"], 3498.0) and close(md["AskPrice1"], 3502.0), md
        print("[ok] RTN_DEPTH_MD: %s bid1=%.0f ask1=%.0f (vt %s)" % (md["InstrumentID"], md["BidPrice1"], md["AskPrice1"], md["UpdateTime"]))

        # -- crossing limit buy: immediate fill at 3502 ----------------------
        cli.order_insert(INSTRUMENT, direction="0", offset="0", volume=1, limit_price=BUY_PRICE, exchange="SHFE")
        order_evt, trade_evt = None, None
        for kind, field in cli.events(timeout=5.0):
            if kind == RTN_TRADE and trade_evt is None:
                trade_evt = field
            elif kind == RTN_ORDER and field["OrderStatus"] == ord("0") and order_evt is None:
                order_evt = field
            if order_evt is not None and trade_evt is not None:
                break
        assert trade_evt is not None, "no RTN_TRADE"
        assert order_evt is not None, "no final RTN_ORDER"
        assert close(trade_evt["Price"], BUY_PRICE), trade_evt
        assert trade_evt["Volume"] == 1 and trade_evt["Direction"] == ord("0"), trade_evt
        assert trade_evt["OffsetFlag"] == ord("0"), trade_evt
        print("[ok] RTN_TRADE: %s %s %.0f x %d" % (trade_evt["InstrumentID"], trade_evt["TradeID"], trade_evt["Price"], trade_evt["Volume"]))

        acct = cli.qry_trading_account()
        assert close(acct["Balance"], INITIAL_FUNDS - COMM_BUY + UNREALIZED_OPEN, 1e-4), acct
        assert close(acct["CurrMargin"], MARGIN_OPEN, 1e-4), acct
        assert close(acct["Available"], INITIAL_FUNDS - COMM_BUY + UNREALIZED_OPEN - MARGIN_OPEN, 1e-4), acct
        print("[ok] after open: balance=%.4f margin=%.2f available=%.4f" % (acct["Balance"], acct["CurrMargin"], acct["Available"]))
        available_after_open = acct["Available"]

        positions = cli.qry_investor_position(INSTRUMENT)
        assert len(positions) == 1, positions
        p = positions[0]
        assert p["PosiDirection"] == ord("2") and p["Position"] == 1 and p["TodayPosition"] == 1, p
        assert p["YdPosition"] == 0, p
        print("[ok] position: %s dir=long pos=%d today=%d" % (p["InstrumentID"], p["Position"], p["TodayPosition"]))

        # -- insufficient funds: park orders until frozen margin eats equity ---
        # The parked size is derived, not magic: it takes exactly as much as the
        # remaining equity can carry, leaving less than one lot's margin so
        # the next insert must fail. Computing it keeps the case honest when
        # the ref data (and therefore the per-lot margin) changes.
        park_lots = int(available_after_open // MARGIN_PER_LOT)
        assert available_after_open - park_lots * MARGIN_PER_LOT < MARGIN_PER_LOT, (
            available_after_open, park_lots)
        cli.order_insert(INSTRUMENT, direction="0", offset="0", volume=park_lots,
                         limit_price=REST_PRICE, exchange="SHFE", order_ref="900")
        parked = None
        for kind, field in cli.events(timeout=5.0):
            # §8.9: the first push is the unknown ('a') order; the queueing
            # confirmation ('3') follows — filter by status, not by arrival
            if kind == RTN_ORDER and field["OrderRef"] == "900" and field["OrderStatus"] == ord("3"):
                parked = field
                break
        assert parked is not None and parked["OrderStatus"] == ord("3"), parked
        frozen = cli.qry_trading_account()
        # Frozen at 昨结算价 x 乘数 x 公司费率 x 手数 — the LimitPrice is
        # deliberately NOT the basis (notes/04 C2: 期货公司算法不一, and the
        # bundled MarginPriceType '1' means 昨结算). A client that assumes the
        # quote price is used would compute a different number here.
        assert close(frozen["FrozenMargin"], park_lots * MARGIN_PER_LOT, 1e-4), frozen
        print("[ok] parked %d lots: frozen_margin=%.2f available=%.2f"
              % (park_lots, frozen["FrozenMargin"], frozen["Available"]))

        try:
            cli.order_insert(INSTRUMENT, direction="0", offset="0", volume=1, limit_price=REST_PRICE, exchange="SHFE")
            raise AssertionError("underserved order accepted")
        except CTPError as e:
            assert e.error_id == 31, e  # ERR_FUNDS: frozen + new > available
        print("[ok] next 1 lot rejected (ErrorID 31, insufficient funds)")

        cli.order_action(INSTRUMENT, order_ref="900")
        for kind, field in cli.events(timeout=5.0):
            if kind == RTN_ORDER and field["OrderRef"] == "900" and field["OrderStatus"] == ord("5"):
                break
        unfrozen = cli.qry_trading_account()
        assert close(unfrozen["FrozenMargin"], 0.0, 1e-6), unfrozen
        print("[ok] parked order cancelled, frozen released")

        # -- resting GFD order + cancel ---------------------------------------
        cli.order_insert(INSTRUMENT, direction="0", offset="0", volume=1, limit_price=REST_PRICE, exchange="SHFE", order_ref="777")
        final = None
        for kind, field in cli.events(timeout=5.0):
            # §8.9: unknown ('a') first, then the queueing confirmation ('3')
            if kind == RTN_ORDER and field["OrderRef"] == "777" and field["OrderStatus"] == ord("3"):
                final = field
                break
        assert final is not None and final["OrderStatus"] == ord("3"), final  # parked in book
        assert final["OrderRef"] == "777", final
        print("[ok] GFD order parked in book (OrderRef 777, status '3')")

        rows = cli.qry_order(INSTRUMENT)
        assert len(rows) == 3, rows  # filled + parked/cancelled 900 + parked 777
        assert any(r["OrderRef"] == "777" and close(r["LimitPrice"], REST_PRICE) for r in rows), rows

        cli.order_action(INSTRUMENT, order_ref="777")
        cancelled = None
        for kind, field in cli.events(timeout=5.0):
            if kind == RTN_ORDER and field["OrderStatus"] == ord("5"):
                cancelled = field
                break
        assert cancelled is not None and cancelled["VolumeTotal"] == 1, cancelled
        print("[ok] order cancelled (status '5', VolumeTotal restored)")

        acct = cli.qry_trading_account()
        assert close(acct["FrozenMargin"], 0.0, 1e-6), acct
        assert close(acct["Available"], INITIAL_FUNDS - COMM_BUY + UNREALIZED_OPEN - MARGIN_OPEN, 1e-4), acct
        print("[ok] cancel released the frozen estimate")

        # -- crossing limit sell close at 3498 --------------------------------
        cli.order_insert(INSTRUMENT, direction="1", offset="1", volume=1, limit_price=SELL_PRICE, exchange="SHFE")
        trade_evt = None
        for kind, field in cli.events(timeout=5.0):
            if kind == RTN_TRADE:
                trade_evt = field
                break
        assert trade_evt is not None and close(trade_evt["Price"], SELL_PRICE), trade_evt
        assert trade_evt["OffsetFlag"] == ord("1"), trade_evt  # close
        print("[ok] RTN_TRADE close: %.0f (realized pnl %.2f)" % (trade_evt["Price"], (SELL_PRICE - BUY_PRICE) * 10))

        acct = cli.qry_trading_account()
        expected = INITIAL_FUNDS - COMM_BUY + (SELL_PRICE - BUY_PRICE) * 10 - COMM_SELL
        assert close(acct["Balance"], expected, 1e-4), (acct, expected)
        assert close(acct["CurrMargin"], 0.0, 1e-6), acct
        assert close(acct["Available"], expected, 1e-4), acct
        assert close(acct["CloseProfit"], -40.0, 1e-6), acct
        positions = cli.qry_investor_position(INSTRUMENT)
        assert positions == [] or all(p["Position"] == 0 for p in positions), positions
        print("[ok] after close: balance=%.4f close_profit=%.2f position flattened" % (acct["Balance"], acct["CloseProfit"]))

        trades = cli.qry_trade(INSTRUMENT)
        assert len(trades) == 2, trades
        print("[ok] qry trade stream: %d trades" % len(trades))

        # -- unknown instrument rejected --------------------------------------
        try:
            cli.order_insert("NOPE9999", direction="0", offset="0", volume=1, limit_price=100.0, exchange="SHFE")
            raise AssertionError("unknown instrument accepted")
        except CTPError as e:
            assert e.error_id == 16, e  # INSTRUMENT_NOT_FOUND
        print("[ok] unknown instrument rejected (ErrorID 16)")

    # -- journal: the authoritative event stream must record the flow --------
    # shut the core down first so the BufWriter flushes (Drop flushes too)
    admin.shutdown()
    time.sleep(0.2)
    journal_dir = os.path.join(data_dir, "journal")
    files = sorted(f for f in os.listdir(journal_dir) if f.endswith(".jsonl"))
    assert files, "no journal file written"
    events = []
    for name in files:  # server_start lands on the wall day; trading on the scenario day
        with open(os.path.join(journal_dir, name), "r", encoding="utf-8") as f:
            for line in f:
                events.append(json.loads(line))
    types = [e["type"] for e in events]
    for want in ("session_auth", "order_insert", "order_update", "fill", "order_cancel"):
        assert want in types, (want, types)
    fills = [e for e in events if e["type"] == "fill"]
    assert len(fills) == 2, fills
    assert fills[0]["broker"] == BROKER and fills[0]["investor"] == INVESTOR, fills[0]
    assert all(e.get("seq", 0) > 0 for e in events), "seq missing"
    print("[ok] journal: %d events, types=%s" % (len(events), ",".join(sorted(set(types)))))


if __name__ == "__main__":
    sys.exit(main())
