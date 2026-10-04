---
title: Ledger、FIFO 与显式日结
linkTitle: Ledger 与日结
weight: 40
description: 说明账户、持仓明细、先开先平、冻结和日结原子替换的源码边界。
---

实现位于 `core/ctpbuddy-ledger/src/lib.rs`，由 server 世界循环单写者驱动，不在账本内部做 IO。主要来源提交为 `2b69cc7`（持仓明细与 FIFO）、`969432f`（显式日结）和 `4d2af40`（验收闭环）。

## 账户公式

`Account::dynamic_equity()` 为 `balance + deposit - withdraw + position_profit`；`available()` 再扣 `used_margin`、`frozen_margin`、`frozen_commission`；`risk()` 为 `used_margin / dynamic_equity`（权益非正时返回 0）。`Account::to_field(trading_day)` 投影为 `CThostFtdcTradingAccountField`。

`Ledger::freeze()` 在开仓时冻结估算保证金和手续费；成交后按实际价格折算，剩余冻结释放。平仓下单时还冻结 today/yd 可用仓位，避免两个并发平仓超过持仓。错误边界为资金不足 31、持仓不足 30、平今不足 50、平昨不足 51。

## PositionDetail 与 FIFO

每笔开仓成交创建一个 `PositionDetail(OpenDate, TradeID)`。`take_details_filtered(volume, today_only, yd_only, trading_day)` 从最老 detail 开始取量；过滤只限制今仓/昨仓桶，不改变桶内 FIFO 顺序。`detail_pnl()` 使用 detail basis：今仓按开仓价，昨仓按昨结算价。这不是平均持仓成本模型。

SHFE/INE 查询在 `to_query_fields()` 中按 today/yd 生成分行；非 SHFE/INE 保持单行。静态 `YdPosition` 没有初始结算输入时不会从当前仓位反推，零余量 detail 不应伪造成真实开仓流水。

## 日结 API 与原子性

Python：

```python
with Admin(calendar=calendar) as a:
    a.settle_day({"rb2601": 3501.0}, next_trading_day="20261005")
```

Rust `World::admin_settle_day` 先要求 playback 暂停、活动订单为 0、价格对象合法、字段无重复；随后 clone ledger/engine/playback，在 staged ledger 上调用 `settle_trading_day()`，成功后才整体替换。`Ledger::settle_trading_day()` 还要求两个日期合法且严格递增、所有持仓有正有限结算价、没有未释放冻结。失败不会部分改变 live ledger。

成功后：

1. 结算价盯市并计算最终动态权益；
2. 所有存量 detail 转为昨仓，today 清零，昨仓初始量更新；
3. `pre_balance` 与 `balance` 滚存，日内 deposit/withdraw/盈亏/手续费和冻结清零；
4. detail 的昨结算价更新，旧日订单、成交、确认状态清理；
5. playback 丢弃旧日未播放行情并暂停。

该能力是显式用户供价的最小模型：不会自动按固定时刻触发，也不伪造官方完整结算单。`settlement_report` 保存的是用户供给或项目生成的最小报告，正文明确标注 `modeled_ledger_only`。

## 明确限制

这不是全品种、全期权、全规则账本。按交易所区分的平今/平昨偏好、期权权利金与复杂保证金、完整初始结算状态仍需 RefData/输入支持。真实数据验收只覆盖受控普通期货子集，不等同于真实账户完整重演。
