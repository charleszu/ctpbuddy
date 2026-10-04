---
title: 撮合与交易所回报
linkTitle: 撮合与回报
weight: 30
description: 从 MatchingEngine、FAK/FOK 和 EngineEvent 说明成交与回报顺序。
---

实现位于 `core/ctpbuddy-matching/src/engine.rs`，服务层把 `EngineEvent::Order` / `EngineEvent::Trade` 转成 wire 回报。主要来源提交为 `e3b988f`（限价簿）、`d371405`（FAK 三所分流）和 `2a47c60`（OrderSysID 接受边界）。

## 入场与簿

`MatchingEngine::check(&OrderIntent)` 返回 `Result<(), (i32, String)>`；典型错误包括合约不存在 16、合约不可交易 17、字段错误 15、交易所不符 148、价格跳动 165、涨跌停 163、数量 164、重复报单 22、资金不足 31，以及报撤单流控 116。`submit(intent, &ClockCtx)` 返回 `SubmitOutcome::Accepted { events }` 或 `Rejected { error_id, msg }`。

每个合约 `Book` 的 bids 按价格降序、arrival_seq 升序，asks 按价格升序、arrival_seq 升序。订单到达时先比较 resting order，再比较 tick 五档深度；同价时 tick 深度优先，因为它代表更早进入队列。静态 tick 深度每次匹配用 `used: [i32; DEPTH]` 消耗，该消耗标记属于单次 matching pass，不是跨 tick 的全局历史成交量。

撮合只发生在订单到达和 tick 到达两个时点；resting order 不直接互相撮合，tick 是外部对手盘。自成交防护按 `(broker, investor)` 跳过同账户 resting 单。

## FAK/FOK 与三种回报布局

CTP 编码在源码注释中固定：FOK 为 `TC_IOC('1') + VC_CV('3')`；FAK 为 IOC 加 `VC_AV('1')` 或 `VC_MV('2')`，后者使用 `MinVolume`；GFD 剩余量入簿，IOC 剩余量撤销。

`ioc_layout(exchange_id)` 返回：

| 交易所 | 布局 | 实际顺序 |
|---|---|---|
| SHFE/INE/CFFEX 与未知 | `CancelFirst` | 撤单行先带成交量，再成交回报 |
| DCE/GFEX | `TradeDriven` | `3` 确认，逐笔成交状态行，再 `5` 撤单 |
| CZCE | `StatusDriven` | `3` 确认，前态 + `1` 部成 + Trade，再 `5` |

客户端不能假定“一笔成交一个统一订单行”。`EngineEvent::Trade` 带 wire trade field 和内部 `Fill`，ledger 使用 `Fill` 更新持仓；服务层负责把订单状态变化按 CTP SPI 顺序推送。

## 状态与系统号

订单第一次对外回报可为空 `OrderSysID`；交易所接受边界后才填充最终系统号。交易所拒单全程为空。内部 `internal_sys_id` 用于 engine/ledger/journal 关联，外部 `order_sys_id` 用于 Trade、QryOrder 和撤单。

终态订单仍留在 `terminal_refs`，所以对已全成/已撤订单再次撤单返回 26 `INSUITABLE_ORDER_STATUS`，而不是把记录当不存在返回 25。撤单查询先按 system id，再按 `(front, session, order_ref)`，最后才在投资者范围内按 order ref 兜底。

验证时应断言完整顺序、错误码和空号边界，而不只断言最终成交数量。对应 e2e 包括 `tests/e2e/m2_book.py`、`m2_ioc.py`、`m3_order_sysid.py`。
