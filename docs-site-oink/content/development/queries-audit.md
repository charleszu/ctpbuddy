---
title: 查询、投影与真实审计边界
linkTitle: 查询与审计
weight: 40
description: 说明 SDK 查询流、SQLite 投影和真实数据受控审计的可证范围。
---

源码入口为 `py/ctpbuddy/sdk/client.py`、`py/ctpbuddy/store.py`、`py/ctpbuddy/journal.py` 与 `tools/audit_query_expectations.py`。主要来源提交为 `12d55e8`（投影查询）、`6d2afe2`（真实订单驱动 Core 四查询子集）和 `4d2af40`。

## SDK 查询

`Client` 提供 `qry_instrument(instrument="")`、`qry_trading_account()`、`qry_investor_position(instrument="")`、`qry_investor_position_detail(instrument="")`、`qry_order(instrument="")`、`qry_trade(instrument="")`、三张费率查询、`qry_settlement_info(trading_day="", account_id="", currency_id="")` 等方法。查询使用 `QRY_LAST` 结束；查询频率超过配置预算时核心返回官方 90，SDK 的 `_query_stream()` 按既定次数透明重试，不能把重试当成无界等待。

参考数据查询与账本使用同一 Catalog/RefData 表，便于将 `CurrMargin` 与保证金查询交叉核对。但 provider 是用户供给的鸭子类型；缺表代表该 desk 无此规则，不应在文档中填入猜测值。

## Journal 与 SQLite

Rust `Journal::new(data_dir)` 写入 `data_dir/journal/<trading_day>.jsonl`。`record(day, vt_ms, event_type, broker_id, investor_id, data)` 按世界循环顺序递增 seq；1000 条或 100ms 触发 flush，退出时 flush，但不是每事件 fsync。崩溃可能丢最后一批，启动快照/重放边界必须单独说明。

```bash
ctpbuddy journal rebuild ./data/journal --db ./data/ctpbuddy.db
ctpbuddy journal query account --db ./data/ctpbuddy.db --broker 8888 --limit 100
ctpbuddy journal query order_record --db ./data/ctpbuddy.db --investor test01
```

`Projection.query(table, broker=None, investor=None, trading_day=None, limit=100, offset=0)` 只允许白名单表和参数化 SQL；`limit` 为 1..10000，`offset` 非负。Web 返回 `realtime: false`，因为 SQLite 是 journal 快照投影，不是 live ledger。

## 真实审计口径

`tools/audit_query_expectations.py` 从同交易日、同账号的 `order.csv`、`trade.csv` 和结算单构建 `ReqQryOrder`、`ReqQryTrade`、`ReqQryInvestorPosition`、`ReqQryInvestorPositionDetail` 的可核期望。订单优先使用 `CombOffsetFlag`，成交使用 `OffsetFlag`；OrderSysID 只作 trim 后关联键，OrderRef 兜底必须包含 FrontID/SessionID；多候选为 ambiguity，不能取 first。

`tests/e2e/m4_real_replay.py` 只回放 controlled futures subset：普通期货、RefData 覆盖、OffsetFlag=0、HedgeFlag=1、空投资单元、唯一关联且完全成交。没有外部目录是 SKIP；有数据但没有合法候选是失败。报告仅保存匿名 hash、计数、差异和 skip/fail 原因，不保存真实正文。

没有真实查询回报时状态是 `not_evaluated`，不是通过。该工具是 source alignment/query expectations，不是完整 Core 账本重演；期权只做数量变化校验，不把权利金或期权保证金换算成期货资金。
