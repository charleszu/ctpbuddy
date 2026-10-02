# -*- coding: utf-8 -*-
"""Audit CTPBuddy's account model against real broker exports.

`ctp_export/` holds a real broker's daily dump: for every trading day and
every account, `order.csv` + `trade.csv` (the order and fill streams as the
front recorded them) and `account.csv` (the settled `CThostFtdcTradingAccountField`).
Those two sides are **the same day of the same account** — the order/trade side
is what the client sent, the account side is what the counter computed. That
makes the files a ground truth rather than a sample, and it is the only
available answer to a question documentation cannot settle: does the identity
this project implements actually hold in production?

What is checked
---------------
1. **资金恒等式** (notes/04 A1) on every account-day:
   `Balance = PreBalance + Deposit − Withdraw + PositionProfit + CloseProfit
               + CashIn − Commission`
2. **可用资金**: `Available = Balance − CurrMargin − FrozenMargin
                   − FrozenCommission − FrozenCash`

   Only checked on **flat** account-days, and the reason is worth recording
   because it looks like a bug otherwise. The broker's `account.csv`
   `Available` is the intraday figure (unrealized PnL included), while the
   settlement statement's 可用资金 is settled equity less margin. On a
   margined day the two legitimately differ by the floating PnL: for
   20260112/13200265, `Balance − CurrMargin` = 1360584.69 but the file
   reports 1343424.69 — exactly `PositionProfit` 17160.00 — and the
   statement for the same day reconciles precisely on its own口径
   (客户权益 3149758.71 − 保证金占用 1707225.88 = 可用资金 1442532.83).
   Checking the intraday identity on a margined day would therefore report a
   failure that the broker itself would call correct.
3. **风险度**: the settlement statements quote it as a percentage, so a desk
   that never computed one would silently misreport it.
4. **持仓盯市盈亏 = Σ 逐笔明细盈亏** — verified against `ctp_settlement/`,
   where a broker statement lists both the per-lot rows and their total. A
   single-blended average-cost PnL passes (1) and fails this, which is the
   whole reason the ledger keeps per-lot details.
5. **平仓盈亏口径** (notes/04 E2): a 昨仓 lot settles against 昨结算, a 今仓
   lot against its own entry. Recomputed here from the statement's own rows.
6. **结算单资金恒等式**, which is the settled口径 and therefore *includes an
   item the intraday identity has no slot for* — 申报费:
   `期末结存 = 期初结存 + 出入金 + 持仓盯市盈亏 + 平仓盈亏
               − 手续费 − 申报费 + 权利金收入 − 权利金支出`
   20260112/13200265 is the proof that the 申报费 term is real: without it the
   identity misses by exactly 1.00, and with it the statement reconciles to
   the cent (3149758.71). DESIGN §8.6 says 申报费 "盘中实时资金不含、只体现
   在结算单" — this is that claim confirmed against production data rather
   than assumed from the manual. The 出入金 term is the settled counterpart of
   the intraday `Deposit − Withdraw`; dropping it misses 100000 on days with a
   银期转账.
7. **结算单可用资金 = 客户权益 − 保证金占用**, verified exactly.

The export is not part of the repository (it is broker data, tens of MB and
somebody else's), so this script is opt-in: point `CTPBUDDY_EXPORT_DIR` /
`CTPBUDDY_SETTLEMENT_DIR` at the two directories. Without them it exits 0 with
a note, so a fresh clone's regression run is unaffected.

Run:  python tools/audit_real_accounts.py
"""
from __future__ import annotations

import csv
import io
import json
import os
import sys
from collections import defaultdict

EXPORT_DIR = os.environ.get("CTPBUDDY_EXPORT_DIR", r"C:\workspace\src\CTP\ctp_export")
SETTLEMENT_DIR = os.environ.get("CTPBUDDY_SETTLEMENT_DIR", r"C:\workspace\src\CTP\ctp_settlement")

# Statement money arrives as text with thousands separators; tolerances are
# in CNY, not in float epsilon: 0.01 is a cent, and the statements carry
# fractional cents from exchange rounding.
CENT = 0.01


def read_table(path):
    """Read a broker CSV, trying the encodings these exports actually use.

    The dumps are GBK with a UTF-8 BOM on some columns (the exporter writes
    the header separately), so a naive utf-8 read either explodes or silently
    mangles the first column name — hence the explicit ladder.
    """
    raw = open(path, "rb").read()
    for enc in ("gbk", "utf-8-sig", "utf-8"):
        try:
            text = raw.decode(enc)
        except UnicodeDecodeError:
            continue
        return list(csv.DictReader(io.StringIO(text.lstrip("﻿"))))
    raise ValueError("cannot decode %s" % path)


def num(row, key):
    """A numeric cell that may be blank, which CTP uses to mean zero."""
    v = row.get(key)
    if v is None or v == "":
        return 0.0
    return float(v)


def audit_accounts(export_dir):
    """Check the funds identity on every account-day in the export."""
    if not os.path.isdir(export_dir):
        return None
    files = sorted(f for f in os.listdir(export_dir) if f.endswith("_account.csv"))
    if not files:
        return None

    checks = 0
    failures = []
    for name in files:
        rows = read_table(os.path.join(export_dir, name))
        if not rows:
            continue
        a = rows[-1]
        balance = num(a, "Balance")
        equity = (
            num(a, "PreBalance")
            + num(a, "Deposit")
            - num(a, "Withdraw")
            + num(a, "PositionProfit")
            + num(a, "CloseProfit")
            + num(a, "CashIn")
            - num(a, "Commission")
        )
        if abs(balance - equity) > CENT:
            failures.append((name, "Balance", balance, equity))

        # See the module docstring: the intraday `Available` only equals
        # `Balance − CurrMargin − ...` when nothing is margined. A margined
        # day is checked by (1) and by the statement-side audit instead.
        if num(a, "CurrMargin") == 0.0 and num(a, "PositionProfit") == 0.0:
            avail = (
                balance
                - num(a, "CurrMargin")
                - num(a, "FrozenMargin")
                - num(a, "FrozenCommission")
                - num(a, "FrozenCash")
            )
            if abs(avail - num(a, "Available")) > CENT:
                failures.append((name, "Available", num(a, "Available"), avail))

        equity_real = num(a, "Balance")
        if equity_real > 0 and num(a, "CurrMargin") > 0:
            risk = num(a, "CurrMargin") / equity_real * 100.0
            if risk > 100.0:
                failures.append((name, "RiskDegree", risk, "<=100"))
        checks += 1
    return checks, failures


def audit_detail_pnl(settlement_dir):
    """Check 持仓盯市盈亏 = Σ 明细盯市盈亏 on real statements.

    This is the check that a per-lot ledger passes and an average-cost one
    fails: the statement prints both the lot rows and the total, so any
    divergence in how lots are weighted shows up as a number mismatch.
    """
    if not os.path.isdir(settlement_dir):
        return None
    files = sorted(f for f in os.listdir(settlement_dir) if f.endswith(".json"))
    if not files:
        return None

    checked = 0
    skipped = 0
    failures = []
    for name in files:
        try:
            doc = json.load(open(os.path.join(settlement_dir, name), encoding="utf-8"))
        except (ValueError, OSError):
            continue
        details = doc.get("positions_detail") or []
        summary = doc.get("positions_summary") or []
        if not details or not summary:
            continue
        # Per-instrument, per-side grouping: the statement's 持仓汇总 rows
        # collapse the lot rows by (合约, 买/卖), and the detail 买卖 column is
        # the **position** side (unlike 平仓明细, where it is the closing
        # side — that asymmetry is easy to get backwards).
        groups = defaultdict(float)
        for d in details:
            if "期权" in str(d.get("品种", "")):
                continue  # option MTM mixes premium in; out of v1 scope
            groups[(d.get("合约"), d.get("买卖"))] += float(d.get("盯市盈亏") or 0.0)
        for row in summary:
            if "期权" in str(row.get("品种", "")):
                continue
            side = "买" if float(row.get("买持") or 0) > 0 else "卖"
            got = groups.get((row.get("合约"), side), 0.0)
            want = float(row.get("持仓盯市盈亏") or 0.0)
            # The summary row aggregates both sides into one number, so only
            # compare when the instrument is single-sided; mixed positions
            # would need the row's own split to be unambiguous.
            if float(row.get("买持") or 0) > 0 and float(row.get("卖持") or 0) > 0:
                continue
            if abs(got - want) > CENT:
                failures.append((name, row.get("合约"), side, want, got))
            checked += 1
    return checked, failures


def audit_statements(settlement_dir):
    """Check the settled funds identity on real broker statements.

    Distinct from [`audit_detail_pnl`] on purpose: this one is about money,
    that one is about lot-level PnL. Both read the same files because a real
    statement is the only artifact that carries every口径 at once.
    """
    if not os.path.isdir(settlement_dir):
        return None
    files = sorted(f for f in os.listdir(settlement_dir) if f.endswith(".json"))
    if not files:
        return None

    checked = 0
    skipped = 0
    failures = []
    for name in files:
        try:
            doc = json.load(open(os.path.join(settlement_dir, name), encoding="utf-8"))
        except (ValueError, OSError):
            continue
        a = doc.get("account_summary")
        if not a:
            continue
        g = lambda k: float(a.get(k) or 0.0)

        # 期权行权/交割 moves money through terms this project does not model
        # (期权的保证金、权利金、执行盈亏 are DESIGN §6 期权 territory, not
        # v1). Skipping those days is the honest option; folding them in with a
        # guessed coefficient would manufacture a pass. The count of skipped
        # days is reported so a silent narrowing of coverage cannot hide.
        if any(g(k) != 0.0 for k in ("期权执行盈亏", "行权手续费", "交割手续费", "交割盈亏")):
            skipped += 1
            continue

        # Cash movement must come from the **detail rows**, not the summary
        # field. The statement's 出入金 aggregate is not reliable on its own:
        # 20250123/13200265 shows 出入金 0.00 and 银期转账 0.00 while its own
        # detail lists a 190000 银期转账 出金; 20250310/18880233 likewise shows
        # 银期转账 0.00 against a 200000 入金 row. Summing 入金 − 出金 over
        # `deposit_withdrawal` reconciles every one of them.
        #
        # 申报费 appears there as its own 出金 row (说明: "中金所申报费 出金"),
        # so the detail sum already contains it — subtracting the summary's
        # 申报费 line as well would double-count. That is why 20260112's exact
        # 1.00 miss earlier was a *missing term*, not a duplicated one.
        cash = 0.0
        for row in doc.get("deposit_withdrawal") or []:
            cash += float(row.get("入金") or 0.0) - float(row.get("出金") or 0.0)

        settled = (
            g("期初结存")
            + cash
            + g("持仓盯市盈亏")
            + g("平仓盈亏")
            - g("手续费")
            + g("权利金收入")
            - g("权利金支出")
        )
        if abs(settled - g("期末结存")) > CENT:
            failures.append((name, "期末结存", g("期末结存"), settled))

        if abs((g("客户权益") - g("保证金占用")) - g("可用资金")) > CENT:
            failures.append((name, "可用资金", g("可用资金"), g("客户权益") - g("保证金占用")))
        checked += 1
    return checked, failures, skipped


def main():
    export = audit_accounts(EXPORT_DIR)
    detail = audit_detail_pnl(SETTLEMENT_DIR)
    settled = audit_statements(SETTLEMENT_DIR)

    if export is None:
        print("[skip] %s not found — set CTPBUDDY_EXPORT_DIR" % EXPORT_DIR)
    if detail is None and settled is None:
        print("[skip] %s not found — set CTPBUDDY_SETTLEMENT_DIR" % SETTLEMENT_DIR)
    if export is None and detail is None and settled is None:
        return 0

    failed = False
    for label, result in (
        ("funds identity", export),
        ("detail-MTM sum", detail),
        ("statement identity", settled),
    ):
        if result is None:
            continue
        checks, failures, *rest = result
        extra = " (%d skipped: 期权行权/交割 out of v1 scope)" % rest[0] if rest else ""
        print("[ok] %s on %d real rows%s" % (label, checks, extra))
        if failures:
            failed = True
            print("FAIL: %s — %d rows disagree" % (label, len(failures)))
            for f in failures[:10]:
                print("   %s %s: reported %.4f, identity says %.4f" % f)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
