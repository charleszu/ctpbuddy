# -*- coding: utf-8 -*-
"""Discriminate CFFEX close-today fee attribution from real settlement statements.

知识库 §10.4 #10（2026-10-08 登记，同日以本脚本对生产数据判别）：中金所
「持仓减扣」与「手续费」是两条轴——指令面无平今指令、按先开先平消耗；但
官方 FAQ（21454 号）称平今手续费数量按成交时间逐笔「先平当日新开仓，再平
历史仓」判定，即**有今仓时先按平今费率收**，与账本现行的「按被消耗明细
年龄计费」（先开先平 → 昨仓先消耗 → 收平昨费）相反。

配对规则（官方 FAQ 情形 1：买开后卖平 → 卖平计平今）：**卖平消耗买开池，
买平消耗卖开池**——平仓消耗的是对手方向的开仓池。

判别法（不需要外部费率表）：对每 (合约,投保,平仓方向) 按成交序号重放当日
对手向开仓池，平仓成交 t 取 take=min(手数,池)：

1. 零池平仓（take=0）：费率 = 平昨率（两模型一致，作费率基准）；
2. 有池平仓（take>0）：官方模型 → 至少 take 手按平今率；
3. 对打计分：在 r_today ≠ r_yd 的判别有效组内，逐笔比较
   官方预测 (take·r_today+rem·r_yd)·额/手 与 年龄模型预测
   (昨仓手·r_yd+今仓手·r_today)·额/手（年龄由 position_closed 的
   开仓日期 join 得到）。

用法：``CTPBUDDY_SETTLEMENT_DIR=D:/... python tools/audit_cffex_close_fee.py``
无该目录时优雅跳过。费率按 (品种, 年份, 投保) 聚类，避免跨年调费混口径。
"""

import json
import os
import re
from collections import defaultdict

SETTLE_DIR = os.environ.get("CTPBUDDY_SETTLEMENT_DIR", "")
# 只取股指期货：IF/IH/IC/IM 的费率是按成交额比例；国债(T/TS/TF/TL)是每手固定
# 值，费率聚类法不适用，本判别不覆盖。
IDX_FUT_RE = re.compile(r"^(IF|IH|IC|IM)\d{4}$")
FEE_TOL = 0.02          # 手续费结算到分
CLUSTER_REL = 0.05      # 费率簇的相对间隔阈值


def iter_settlement_files(root):
    """递归遍历（2024/ 子目录曾被 os.listdir 漏掉，770→976 行的教训）。"""
    for dirpath, _dirnames, filenames in os.walk(root):
        for name in sorted(filenames):
            if name.endswith(".json"):
                yield os.path.join(dirpath, name)


def cluster_median(values):
    """费率聚簇后取最大簇的中位数：排序、相邻相对差 < CLUSTER_REL 归同簇。"""
    if not values:
        return None, 0
    vs = sorted(values)
    best, cur = [vs[0]], [vs[0]]
    for v in vs[1:]:
        if abs(v - cur[-1]) <= max(cur[-1] * CLUSTER_REL, 1e-9):
            cur.append(v)
        else:
            if len(cur) > len(best):
                best = cur
            cur = [v]
    if len(cur) > len(best):
        best = cur
    xs = sorted(best)
    n = len(xs)
    med = xs[n // 2] if n % 2 else 0.5 * (xs[n // 2 - 1] + xs[n // 2])
    return med, len(best)


def main():
    if not SETTLE_DIR or not os.path.isdir(SETTLE_DIR):
        print("[skip] CTPBUDDY_SETTLEMENT_DIR 未设置或不存在，判别未运行")
        return

    recs_by = defaultdict(list)     # (contract, year, hedge, close_side) -> records
    pc_by = defaultdict(list)       # (path, contract, hedge) -> 未消耗平仓明细行
    files = list(iter_settlement_files(SETTLE_DIR))
    cffex_txn_total = 0
    for path in files:
        try:
            with open(path, encoding="utf-8") as f:
                d = json.load(f)
        except (OSError, json.JSONDecodeError):
            continue
        day = os.path.basename(path)[:8]
        year = day[:4]
        acct = os.path.basename(path)[9:17]
        by_group = defaultdict(list)
        for t in d.get("transactions", []):
            c = t.get("合约", "")
            if IDX_FUT_RE.match(c) and t.get("交易所") == "中金所":
                by_group[(c, t.get("投保"))].append(t)
        for (contract, hedge), txns in by_group.items():
            cffex_txn_total += len(txns)
            pc_all = [(float(r["成交价"]), float(r["手数"]), str(r["开仓日期"]))
                      for r in d.get("position_closed", [])
                      if r.get("合约") == contract and r.get("投保") == hedge]
            used = [False] * len(pc_all)
            # 开池：买开/卖开 分开计数（卖平耗买开池，买平耗卖开池）
            pools = {"买": 0.0, "卖": 0.0}
            for t in sorted(txns, key=lambda x: int(x["成交序号"])):
                lots = float(t["手数"])
                side = t.get("买卖")
                if t["开平"] == "开":
                    pools[side] += lots
                    continue
                if "平" not in t["开平"]:
                    continue
                opp = "卖" if side == "买" else "买"
                take = min(lots, pools[opp])
                pools[opp] -= take
                # join 被平明细年龄（按成交价贪心，配不平记 None）
                need, got, ages = lots, 0.0, []
                for i, (p, n_lots, od) in enumerate(pc_all):
                    if got >= need or used[i] or p != float(t["成交价"]):
                        continue
                    take_n = min(n_lots, need - got)
                    got += take_n
                    ages.append((od, take_n))
                    used[i] = True
                if abs(got - need) > 1e-9:
                    ages = None
                recs_by[(contract, year, hedge, side)].append({
                    "day": day, "acct": acct, "seq": int(t["成交序号"]),
                    "price": float(t["成交价"]), "lots": lots,
                    "amount": float(t["成交额"]), "fee": float(t["手续费"]),
                    "take": take, "rem": lots - take,
                    "ages": ages, "hedge": hedge, "side": side,
                })

    # ---- 费率基准：零池 = 平昨率；有池簇的高位 = 平今率 ----
    rates = {}
    for key, recs in sorted(recs_by.items()):
        r_yd, n0 = cluster_median([r["fee"] / r["amount"]
                                   for r in recs if r["take"] == 0])
        r_today, np_ = cluster_median([r["fee"] / r["amount"]
                                       for r in recs if r["take"] > 0])
        rates[key] = (r_yd, r_today)
        if n0 or np_:
            print(f"[{key[0]} {key[1]} {key[2]} {key[3]}平] "
                  f"零池(平昨)={r_yd if r_yd is None else f'{r_yd:.3e}'}×{n0} "
                  f"有池={r_today if r_today is None else f'{r_today:.3e}'}×{np_}")

    # ---- 对打计分（只在 r_today 与 r_yd 显著不同的组内） ----
    off_ok = age_ok = total = 0
    decisive = []
    off_miss = []
    for key, recs in sorted(recs_by.items()):
        r_yd, r_today = rates[key]
        if r_yd is None or r_today is None or r_today <= r_yd * 1.05:
            continue  # 费率同簇或无基准，无判别力
        for r in recs:
            pred_off = (r["take"] * r_today + r["rem"] * r_yd) * r["amount"] / r["lots"]
            if abs(r["fee"] - pred_off) <= FEE_TOL:
                off_ok += 1
            elif len(off_miss) < 5:
                off_miss.append(
                    f"  {r['day']}/{r['acct']} {key[0]} {r['side']}平"
                    f"{r['lots']:.0f}手 价{r['price']} 池{r['take']:.0f} "
                    f"fee={r['fee']:.2f} pred_off={pred_off:.2f} "
                    f"(r_yd={r_yd:.3e} r_today={r_today:.3e})")
            if r["ages"] is None:
                continue
            yd_lots = sum(n for od, n in r["ages"] if od < r["day"])
            today_lots = r["lots"] - yd_lots
            pred_age = (yd_lots * r_yd + today_lots * r_today) * r["amount"] / r["lots"]
            if abs(r["fee"] - pred_age) <= FEE_TOL:
                age_ok += 1
            total += 1
            if len(decisive) < 8 and (
                    (r["take"] > 0 and yd_lots > 0)
                    or (r["take"] == 0 and yd_lots == 0)):
                tag = "平的是昨仓,收平今费" if r["take"] > 0 else "平的是今仓,收平昨费"
                decisive.append(
                    f"  {r['day']}/{r['acct']} {key[0]} {r['side']}平"
                    f"{r['lots']:.0f}手 价{r['price']} 池{r['take']:.0f} "
                    f"fee={r['fee']:.2f} 明细={r['ages']} ← {tag}")

    print()
    if off_miss:
        print("[官方模型未吻合样本]（多为费率调整过渡日的聚类噪声）:")
        for line in off_miss:
            print(line)
    print()
    for line in decisive:
        print("[实锤] " + line)
    print()
    print(f"[scope] files={len(files)} 中金所股指成交行={cffex_txn_total}")
    print(f"[verdict] 判别有效组内平仓成交={total}："
          f"官方时间序模型吻合 {off_ok}/{total}，"
          f"年龄模型吻合 {age_ok}/{total}")
    print("[scope] 判别仅覆盖 IF/IH/IC/IM 期货按比例费率；国债固定费率与期权"
          "不在此判别内；费率值以结算单自身推得，不引外部表。")


if __name__ == "__main__":
    main()
