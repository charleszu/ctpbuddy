#!/usr/bin/env python
"""Reference-data queries (notes/04 G): the four rate lookups a client uses to
cross-check what CTPBuddy charged it.

The point of this suite is not "the query returns rows" — it is that
`ReqQryInstrumentMarginRate` and `ReqQryTradingAccount.CurrMargin` describe
**the same rate table**. A desk that finds them disagreeing has found either a
real broker bug or a simulator that kept two copies of the truth; the latter is
the failure this locks down.

Also pinned here, because both are easy to get backwards:
  * an empty `InstrumentID` means "the contracts I hold", not "the whole
    market" (官方接口描述: 目前无法通过一次查询得到所有合约保证金率);
  * `BrokerID` / `InvestorID` omitted yields an empty stream, not an error.

Usage:
    python tests/e2e/m3_refdata.py
Env:
    CTPBUDDY_CORE   explicit path to the ctpbuddy-server binary
"""
from __future__ import annotations

import os
import queue
import socket
import subprocess
import sys
import tempfile
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "py"))

from ctpbuddy.sdk import Admin, Client  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402
from ctpbuddy.wire import (  # noqa: E402
    REQ_QRY_INSTRUMENT_MARGIN_RATE,
    RSP_QRY_INSTRUMENT_MARGIN_RATE,
)

BROKER = "8888"
RB = "rb2601"   # SHFE, mult 10, tick 1 — real contract from the bundled snapshot
M = "jd2602"    # DCE,  mult 10, tick 1
RB_PRE_SETTLE = 3500.0
M_PRE_SETTLE = 2990.0
INITIAL_FUNDS = 2_000_000.0

# 公司保证金率 from refdata/margin_rates.jsonl (NOT the exchange rate that
# ReqQryInstrument carries). These are the numbers the ledger charges with, so
# the assertions below can multiply them out and compare against CurrMargin.
RB_MARGIN = 0.16
M_MARGIN = 0.15

# When the suite is pointed at a data set carrying commission rates (via
# --refdata), the ledger must charge exactly what the query reports. These are
# the ratios that data set declares: 平今 6x 平昨, as SHFE really prices it.
FEES = False
CLOSE_YD_RATIO = 0.0001
CLOSE_TD_RATIO = 0.0006
OPEN_RATIO = 0.0001


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


def close(a: float, b: float, tol: float = 1e-6) -> bool:
    return abs(float(a) - float(b)) <= tol


def make_scenario(dirpath: str) -> None:
    """Two crossings on rb2601 and one on jd2602, so the holdings scope has
    two contracts and the per-instrument filter has something to exclude."""
    rows = []

    def tick(instrument, exchange, t, ms, last, bid1, ask1, pre_settle):
        # 涨跌停必须跟着合约走：写死成 rb2601 的 3600/3400 会让 jd2602 的
        # 900 价位落在停板之外，服务端按 ERR_INSTRUMENT_NOT_TRADING 拒单。
        upper = round(pre_settle * 1.10, 0)
        lower = round(pre_settle * 0.90, 0)
        row = {c: "" for c in CANONICAL_COLUMNS}
        row.update(
            instrument=instrument, exchange=exchange,
            trading_day="20261002", update_time=t, update_millisec=str(ms),
            last_price=str(last), volume="100", turnover="350000", open_interest="5000",
            # 每合约带自己的昨结算：保证金以昨结算为基数，写死一个值会让
            # DCE 腿拿着 SHFE 的昨结算价，整笔 DCE 持仓就此被错误定价。
            pre_settlement=str(pre_settle), settlement=str(pre_settle),
            pre_close=str(bid1), open=str(last), high=str(ask1 + 4),
            low=str(bid1 - 4), close=str(last),
            upper=str(upper), lower=str(lower), pre_open_interest="4980",
            average=str(pre_settle + 0.25),
            bid1=str(bid1), bid2=str(bid1 - 1), bid3=str(bid1 - 2), bid4=str(bid1 - 3),
            bid5=str(bid1 - 4),
            ask1=str(ask1), ask2=str(ask1 + 1), ask3=str(ask1 + 2), ask4=str(ask1 + 3),
            ask5=str(ask1 + 4),
            bidvol1="10", bidvol2="8", bidvol3="6", bidvol4="4", bidvol5="2",
            askvol1="12", askvol2="10", askvol3="8", askvol4="6", askvol5="4",
        )
        rows.append(row)

    tick(RB, "SHFE", "09:00:01", 0, 3500.0, 3499.0, 3501.0, RB_PRE_SETTLE)
    tick(RB, "SHFE", "09:00:02", 0, 3501.0, 3500.0, 3502.0, RB_PRE_SETTLE)
    tick(M, "DCE", "09:00:03", 0, 3000.0, 3000.0, 3002.0, M_PRE_SETTLE)
    write_canonical(dirpath, rows)


def wait_idx(admin: Admin, n: int, timeout: float = 5.0) -> None:
    """Wait until playback has delivered tick n.

    `admin.step()` only raises a flag; the world loop's pulse actually
    delivers the tick. Sleeping a fixed interval instead would race the
    10ms pulse and silently assert against a book that has not been updated
    yet -- the failure then reads as "the order did not fill".
    """
    deadline = time.time() + timeout
    while time.time() < deadline:
        if admin.status()["playback"]["idx"] >= n:
            return
        time.sleep(0.02)
    raise AssertionError("playback did not reach tick %d" % n)


def drain(cli: Client, quiet: float = 0.35) -> None:
    """Collect push events until the queue stays empty for `quiet` seconds."""
    while True:
        try:
            next(cli.events(timeout=quiet))
        except (StopIteration, queue.Empty):
            return


def wait_fills(cli: Client, order_ref: str, expect: int, timeout: float = 5.0):
    end = time.time() + timeout
    while time.time() < end:
        trades = [t for t in cli.qry_trade() if t["OrderRef"] == order_ref]
        if len(trades) >= expect:
            return trades
        time.sleep(0.05)
    return [t for t in cli.qry_trade() if t["OrderRef"] == order_ref]


def run(td_port: int, admin_port: int, scenario: str) -> None:
    wait_port(admin_port)
    admin = Admin("127.0.0.1:%d" % admin_port)
    assert admin.ping()["cmd"] == "ping"
    stt = admin.status()
    assert stt["instruments"] == 789, stt
    started = admin.start_scenario(scenario, paused=True)
    assert started["ticks"] == 3, started
    admin.step()
    wait_idx(admin, 1)

    wait_port(td_port)
    cli = Client("127.0.0.1:%d" % td_port)
    try:
        cli.auth(BROKER, "ref001")
        cli.login(BROKER, "ref001", password="")
        cli.settle_confirm()
        cli.subscribe([RB, M])
        drain(cli)
        admin.step()
        wait_idx(admin, 2)             # rb2601 tick
        admin.step()
        wait_idx(admin, 3)             # jd2602 tick
        drain(cli)

        # -- holdings so the empty-InstrumentID scope has something to say ---
        cli.order_insert(RB, direction="0", offset="0", volume=3, limit_price=3502.0,
                         exchange="SHFE", order_ref="R1")
        assert len(wait_fills(cli, "R1", 1)) == 1, "rb2601 should fill on insert"
        cli.order_insert(M, direction="0", offset="0", volume=2, limit_price=3002.0,
                         exchange="DCE", order_ref="J1")
        assert len(wait_fills(cli, "J1", 1)) == 1, "jd2602 should fill on insert"
        drain(cli)

        positions = cli.qry_investor_position()
        held = {p["InstrumentID"] for p in positions if p["Position"] > 0}
        assert held == {RB, M}, positions
        print("[ok] holdings: %s" % sorted(held))

        # -- 1) 交易参数: MarginPriceType decides the whole margin basis ----
        tp = cli.qry_broker_trading_params()
        assert len(tp) == 1, tp
        row = tp[0]
        assert row["BrokerID"] == BROKER and row["InvestorID"] == "ref001", row
        assert row["CurrencyID"] == "CNY", row
        # The bundled snapshot declares '1' = 昨结算价, which is what makes the
        # margin assertions below independent of the fill prices.
        assert row["MarginPriceType"] == ord("1"), row
        print("[ok] ReqQryBrokerTradingParams: MarginPriceType='1' (昨结算价)")

        # -- 2) 保证金率: 公司费率, and the ledger must agree with it ---------
        rates = cli.qry_instrument_margin_rate()
        assert {r["InstrumentID"] for r in rates} == {RB, M}, rates
        by_id = {r["InstrumentID"]: r for r in rates}
        assert close(by_id[RB]["LongMarginRatioByMoney"], RB_MARGIN, 1e-9), by_id[RB]
        assert close(by_id[M]["LongMarginRatioByMoney"], M_MARGIN, 1e-9), by_id[M]
        # Echoed back with the caller's identity, as a real counter does.
        for r in rates:
            assert r["BrokerID"] == BROKER and r["InvestorID"] == "ref001", r
            assert r["HedgeFlag"] == ord("1"), r
            assert r["ExchangeID"] in ("SHFE", "DCE"), r
        print("[ok] ReqQryInstrumentMarginRate: 公司费率 %s=%.2f %s=%.2f"
              % (RB, by_id[RB]["LongMarginRatioByMoney"], M, by_id[M]["LongMarginRatioByMoney"]))

        # -- 3) THE cross-check: rate x 昨结算 x mult x lots == CurrMargin ----
        acct = cli.qry_trading_account()
        expected = (
            by_id[RB]["LongMarginRatioByMoney"] * RB_PRE_SETTLE * 10 * 3
            + by_id[M]["LongMarginRatioByMoney"] * M_PRE_SETTLE * 10 * 2
        )
        assert close(acct["CurrMargin"], expected, 1e-4), (acct, expected)
        print("[ok] cross-check: queried rates x 昨结算 x mult x lots == CurrMargin (%.2f)" % expected)

        # -- 4) empty InstrumentID == holdings, never the whole market -------
        assert len(rates) == 2, "must not return all 789 bundled contracts"
        all_rates = cli.qry_instrument_margin_rate(RB)
        assert len(all_rates) == 1 and all_rates[0]["InstrumentID"] == RB, all_rates
        print("[ok] InstrumentID filter: per-contract query returns that contract only")

        # -- 5) 手续费: the answer depends on what the desk supplied ---------
        # Bundled data ships no commission table, so an empty stream is the
        # honest answer (inventing a plausible fee would be worse). When the
        # suite is pointed at a data set that DOES carry rates, the same query
        # must return them and the ledger must charge exactly those.
        comms = cli.qry_instrument_commission_rate()
        if FEES:
            assert {c["InstrumentID"] for c in comms} == {RB, M}, comms
            rb_comm = [c for c in comms if c["InstrumentID"] == RB][0]
            # 平今 must be dearer than 平昨 on SHFE; unifying them is the
            # //// deviation notes/04 B3 warns about.
            assert rb_comm["CloseTodayRatioByMoney"] > rb_comm["CloseRatioByMoney"], rb_comm
            assert close(rb_comm["CloseRatioByMoney"], CLOSE_YD_RATIO, 1e-9), rb_comm
            assert close(rb_comm["CloseTodayRatioByMoney"], CLOSE_TD_RATIO, 1e-9), rb_comm
            print("[ok] ReqQryInstrumentCommissionRate: 平昨 %.4f < 平今 %.4f"
                  % (rb_comm["CloseRatioByMoney"], rb_comm["CloseTodayRatioByMoney"]))
        else:
            assert comms == [], "bundled data ships no commission table"
            print("[ok] ReqQryInstrumentCommissionRate: empty = no such rule")
        assert cli.qry_instrument_order_comm_rate() == [], "no 申报费 outside 中金所"

        # -- 6) mandatory fields omitted -> empty stream, not an error ------
        empty = cli._query_stream(
            REQ_QRY_INSTRUMENT_MARGIN_RATE,
            RSP_QRY_INSTRUMENT_MARGIN_RATE,
            b"\x00" * 512,
        )
        assert empty == [], "BrokerID/InvestorID omitted must yield no rows"
        print("[ok] missing BrokerID/InvestorID -> empty stream (官方: 不填则返回值为空)")

        # -- 7) the calc surface and the query surface are the same table ---
        # Re-read after the position changed: a second table would drift here.
        cli.order_insert(M, direction="0", offset="0", volume=1, limit_price=3002.0,
                         exchange="DCE", order_ref="J2")
        assert len(wait_fills(cli, "J2", 1)) == 1, "second jd2602 fill"
        drain(cli)
        rates2 = {r["InstrumentID"]: r for r in cli.qry_instrument_margin_rate()}
        acct2 = cli.qry_trading_account()
        expected2 = (
            rates2[RB]["LongMarginRatioByMoney"] * RB_PRE_SETTLE * 10 * 3
            + rates2[M]["LongMarginRatioByMoney"] * M_PRE_SETTLE * 10 * 3
        )
        assert close(acct2["CurrMargin"], expected2, 1e-4), (acct2, expected2)
        print("[ok] after adding a lot: CurrMargin tracks the queried rate again (%.2f)" % expected2)

        # -- 8) commission actually charged == commission reported ---------
        # Only meaningful with a fee table. This is the second half of "one
        # rate table, two consumers": not just the margin, the fee the client
        # is charged must equal the fee the client can query.
        acct3 = cli.qry_trading_account()
        assert close(acct3["FrozenCommission"], 0.0, 1e-6), acct3
        if FEES:
            # Two ByMoney legs per fill (notes/04 D1): 成交价 x 数量 x 乘数 x
            # 费率, plus the 按手数 part. The fixture sets ByVolume to 0, so
            # the expected fee is pure ByMoney — which is exactly what proves
            # the rate came from the queried table.
            #   rb2601: 3 lots @ 3502.0 (开仓)
            #   jd2602: 2 lots @ 3002.0, then 1 lot @ 3002.0 (开仓, twice)
            expected_fee = (
                3 * 3502.0 * 10 * OPEN_RATIO
                + 3 * 3002.0 * 10 * OPEN_RATIO
            )
            # CThostFtdcTradingAccountField.Commission = 当日手续费合计.
            assert close(acct3["Commission"], expected_fee, 1e-6), (acct3, expected_fee)
            print("[ok] Commission (当日手续费合计) == queried 开仓费率 x 成交额: %.4f"
                  % expected_fee)
        else:
            assert close(acct3["Commission"], 0.0, 1e-6), acct3
            print("[ok] no fee table -> every fill is free, as configured")
    finally:
        cli.close()


def main() -> int:
    global FEES
    core = find_core()
    refdata = sys.argv[1] if len(sys.argv) > 1 else ""
    if refdata:
        # A desk-supplied data set: expect commission rows and verify the
        # ledger charges them.
        fees_file = os.path.join(refdata, "commission_rates.jsonl")
        FEES = os.path.exists(fees_file)
        print("[info] refdata=%s (commission table %s)"
              % (refdata, "present" if FEES else "absent"))
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-refdata-")
    scenario = os.path.join(tmp, "scenario")
    os.makedirs(scenario)
    make_scenario(scenario)
    data_dir = os.path.join(tmp, "data")
    td_port, admin_port = free_port(), free_port()

    cmd = [
        core,
        "--td", "127.0.0.1:%d" % td_port,
        "--admin", "127.0.0.1:%d" % admin_port,
        "--broker-id", BROKER,
        "--speed", "0",
        "--initial-funds", str(INITIAL_FUNDS),
        "--qry-freq", "16",
        "--data-dir", data_dir,
    ]
    if refdata:
        cmd += ["--refdata", refdata]

    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        run(td_port, admin_port, scenario)
        print("\nM3 REFDATA: PASS")
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