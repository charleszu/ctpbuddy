#!/usr/bin/env python
"""Per-lot position details and 先开先平 (notes/04 B2 / E2).

The aggregate `InvestorPosition` row is a *summary*; the numbers a client must
trust are the per-lot rows behind it. Three things are pinned here, each of
which an average-cost implementation gets wrong in a way that stays invisible
until a position mixes lots of different ages:

1. **One detail per opening fill.** Two fills at different prices are two
   lots, not one averaged lot — each carries its own `OpenPrice`, `Margin`
   and `TradeID`.
2. **先开先平.** Closing takes the *oldest* detail, so the realized PnL is
   priced off the oldest entry. Closing the newest one instead would produce a
   different, equally "reasonable" number, which is the bug.
3. **Σ detail PnL == aggregate PnL.** The summary is not computed by a
   different formula than the rows; it is their sum. This is the invariant a
   client can check from outside, so it is the one asserted here.

The 昨仓 half of the rule (昨仓 lots marked against 昨结算 rather than their
entry) needs a settlement boundary to produce a carried lot, which is M3-5
territory; the basis function itself is covered by the `refdata` unit tests
and by the audit in `tools/audit_real_accounts.py` against real statements.

Usage:
    python tests/e2e/m3_detail.py
Env:
    CTPBUDDY_CORE   explicit path to the ctpbuddy-server binary
"""
from __future__ import annotations

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from m3_refdata import (  # noqa: E402
    BROKER,
    INITIAL_FUNDS,
    M,
    RB,
    RB_MARGIN,
    RB_PRE_SETTLE,
    close,
    drain,
    find_core,
    free_port,
    wait_fills,
    wait_idx,
    wait_port,
)

# rb2601's real 合约乘数 from the bundled snapshot.
RB_MULT = 10

from ctpbuddy.sdk import Admin, Client  # noqa: E402
from ctpbuddy.sources import CANONICAL_COLUMNS, write_canonical  # noqa: E402

TRADING_DAY = "20261002"
MARGIN_PER_LOT = RB_PRE_SETTLE * RB_MULT * RB_MARGIN  # 5600.0


def make_two_lot_scenario(dirpath: str) -> None:
    """Three rb2601 prints so two opening fills can happen at two prices.

    Prices are chosen so the second buy lifts the first: lot A fills at 3502,
    lot B at 3504. An implementation that averaged them would report ~3503 for
    both rows, which is what assertions 3 and 5 detect.
    """
    rows = []

    def tick(t, last, bid1, ask1, vol=("10", "10")):
        row = {c: "" for c in CANONICAL_COLUMNS}
        row.update(
            instrument=RB, exchange="SHFE",
            trading_day=TRADING_DAY, update_time=t, update_millisec="0",
            last_price=str(last), volume="200", turnover="700000", open_interest="5000",
            pre_settlement=str(RB_PRE_SETTLE), settlement=str(RB_PRE_SETTLE),
            pre_close=str(bid1), open=str(last), high=str(ask1 + 5), low=str(bid1 - 5),
            close=str(last),
            upper=str(RB_PRE_SETTLE * 1.10), lower=str(RB_PRE_SETTLE * 0.90),
            pre_open_interest="4980", average=str(last),
            bid1=str(bid1), bid2=str(bid1 - 1), bid3=str(bid1 - 2), bid4=str(bid1 - 3),
            bid5=str(bid1 - 4),
            ask1=str(ask1), ask2=str(ask1 + 1), ask3=str(ask1 + 2), ask4=str(ask1 + 3),
            ask5=str(ask1 + 4),
            bidvol1=vol[0], bidvol2="10", bidvol3="10", bidvol4="10", bidvol5="10",
            askvol1=vol[1], askvol2="10", askvol3="10", askvol4="10", askvol5="10",
        )
        rows.append(row)

    tick("09:00:01", 3500.0, 3499.0, 3501.0, ("20", "20"))
    tick("09:00:02", 3501.0, 3500.0, 3502.0, ("20", "20"))
    tick("09:00:03", 3503.0, 3502.0, 3504.0, ("20", "20"))
    tick("09:00:04", 3503.0, 3500.0, 3504.0, ("20", "20"))
    write_canonical(dirpath, rows)


def main() -> int:
    import subprocess
    import tempfile

    core = find_core()
    tmp = tempfile.mkdtemp(prefix="ctpbuddy-detail-")
    scenario = os.path.join(tmp, "scenario")
    os.makedirs(scenario)
    make_two_lot_scenario(scenario)
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
            "--qry-freq", "16",
            "--data-dir", data_dir,
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        wait_port(admin_port)
        admin = Admin("127.0.0.1:%d" % admin_port)
        started = admin.start_scenario(scenario, paused=True)
        assert started["ticks"] == 4, started
        assert started["trading_day"] == TRADING_DAY, started
        admin.step()
        wait_idx(admin, 1)
        wait_port(td_port)

        cli = Client("127.0.0.1:%d" % td_port)
        cli.auth(BROKER, "detail01")
        cli.login(BROKER, "detail01", password="")
        cli.settle_confirm()
        cli.subscribe([RB, M])
        drain(cli)
        admin.step()
        wait_idx(admin, 2)
        drain(cli)

        # -- 1) two opening fills -> two details ----------------------------
        # 买单须越过 ask1 才会成交：tick 1 的 ask1=3501，故报 3502。
        cli.order_insert(RB, direction="0", offset="0", volume=2, limit_price=3502.0,
                         exchange="SHFE", order_ref="L1")
        assert len(wait_fills(cli, "L1", 1)) == 1, "L1 should fill"
        drain(cli)
        admin.step()
        wait_idx(admin, 3)
        drain(cli)
        # tick 3 的 ask1=3504，报 3505 成交在 3504 —— 与第一笔不同价。
        cli.order_insert(RB, direction="0", offset="0", volume=3, limit_price=3505.0,
                         exchange="SHFE", order_ref="L2")
        assert len(wait_fills(cli, "L2", 1)) == 1, "L2 should fill"
        drain(cli)

        rows = cli.qry_investor_position_detail(RB)
        assert len(rows) == 2, ("one detail per opening fill", rows)
        print("[ok] ReqQryInvestorPositionDetail: 2 opening fills -> 2 lots")

        # -- 2) each lot keeps its own entry, not a blended average ---------
        opens = sorted(r["OpenPrice"] for r in rows)
        assert all(close(o, e, 1e-6) for o, e in zip(opens, (3502.0, 3504.0))), opens
        vols = sorted(r["Volume"] for r in rows)
        assert vols == [2, 3], vols
        # A distinct TradeID per lot: the client keys its own book on these.
        assert len({r["TradeID"] for r in rows}) == 2, rows
        assert all(r["OpenDate"] == TRADING_DAY for r in rows), rows
        print("[ok] lots keep their own OpenPrice/Volume/TradeID: %s" % opens)

        # -- 3) Σ lot PnL == aggregate PnL --------------------------------
        detail_sum = sum(r["PositionProfitByDate"] for r in rows)
        agg = cli.qry_investor_position(RB)[0]
        assert close(agg["PositionProfit"], detail_sum, 1e-6), (agg["PositionProfit"], detail_sum)
        assert close(agg["Position"], 5, 1e-9), agg
        assert close(agg["CurrMargin"] if "CurrMargin" in agg else agg["UseMargin"],
                     5 * MARGIN_PER_LOT, 1e-4), agg
        print("[ok] Σ lot PositionProfitByDate == aggregate: %.2f" % detail_sum)

        # -- 4) 先开先平: closing 1 lot takes the OLDEST detail -------------
        admin.step()
        wait_idx(admin, 4)
        drain(cli)
        # 平今 1 手（上期所今仓须报 '3'）：须低于 bid1 才会成交（tick 4 的 bid1=3500）。
        cli.order_insert(RB, direction="1", offset="3", volume=1, limit_price=3499.0,
                         exchange="SHFE", order_ref="L3")
        assert len(wait_fills(cli, "L3", 1)) == 1, "L3 should fill"
        drain(cli)

        after = cli.qry_investor_position_detail(RB)
        assert len(after) == 2, after
        # The 2-lot detail shrank; the 3-lot one is untouched. If the newest
        # lot had been consumed instead, the 3-lot row would read 2 and the
        # closed PnL would be off by (3500-3502)*mult.
        left = {(r["OpenPrice"], r["Volume"]) for r in after}
        assert left == {(3502.0, 1), (3504.0, 3)}, left
        print("[ok] 先开先平: the 1-lot close came off the 3502 detail, not the 3504 one")

        # -- 5) close PnL priced off the consumed lot's own basis ----------
        # The close filled at 3500 (tick 4's bid1) and 先开先平 took the lot
        # that entered at 3502, so that lot's own basis books
        # (3500 - 3502) * 10 = -20.0. An average-cost ledger would use the
        # blended entry across 2@3502 + 3@3504 = 3503.2 instead and report
        # (3500 - 3503.2) * 10 = -32.0 — off by 12.0, which is exactly the
        # per-lot error this suite exists to catch.
        acct = cli.qry_trading_account()
        blended = (2 * 3502.0 + 3 * 3504.0) / 5.0
        avg_pnl = (3500.0 - blended) * RB_MULT
        own_pnl = (3500.0 - 3502.0) * RB_MULT
        assert close(acct["CloseProfit"], own_pnl, 1e-6), (acct["CloseProfit"], own_pnl)
        closed = [r for r in after if close(r["OpenPrice"], 3502.0, 1e-6)][0]
        assert close(closed["CloseProfitByDate"], own_pnl, 1e-6), closed
        # the account total and the lot's own CloseProfitByDate must agree:
        # one arithmetic for the summary and the rows, not two.
        assert close(acct["CloseProfit"], sum(r["CloseProfitByDate"] for r in after), 1e-6), after
        print("[ok] close PnL = %.2f (own entry 3502), not %.2f (average entry %.1f)"
              % (own_pnl, avg_pnl, blended))
        drain(cli)
        cli.close()
    finally:
        try:
            Admin("127.0.0.1:%d" % admin_port).shutdown()
        except Exception:
            proc.terminate()
        out, _ = proc.communicate(timeout=5)
        if "assertion FAILED" in (out or ""):
            print(out[-1500:])

    print("M3 DETAIL: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
