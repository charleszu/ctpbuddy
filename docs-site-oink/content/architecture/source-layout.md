---
title: 源码目录与职责
linkTitle: 源码目录
weight: 10
description: CTPBuddy 仓库的目录级导读、数据流和修改入口。
---

## 一次请求的路径

```text
CTP 应用
  │ CThostFtdcTraderApi / MdApi ABI
  ▼
C++ Shim（shim/）
  │ CB 帧协议
  ▼
Rust server（core/ctpbuddy-server）
  ├─ wire：帧、消息号、结构体镜像
  ├─ market：CSV tick、虚拟时钟、播放控制
  ├─ matching：价格/时间优先、FAK/FOK、订单状态
  └─ ledger：持仓明细、资金、保证金、手续费、结算
       │ JSONL journal
       ▼
Python CLI / SDK / Web / SQLite 投影
```

## 目录地图

| 目录 | 责任 | 修改时先看 |
|---|---|---|
| `core/ctpbuddy-wire` | 二进制帧、消息 ID、C 结构体镜像 | `msgs.rs`、生成器和四端编号同步 |
| `core/ctpbuddy-market` | canonical tick、虚拟时间和行情状态 | `CSV_COLUMNS`、播放脉冲 |
| `core/ctpbuddy-matching` | 簿、订单生命周期、交易所回报布局 | `engine.rs`、FAK/FOK 测试 |
| `core/ctpbuddy-ledger` | 账户、PositionDetail、资金和日结 | `lib.rs`、Rust 账本测试 |
| `core/ctpbuddy-server` | TCP/ADMIN、请求处理、场景和配置 | `handlers.rs`、`admin.rs` |
| `py/ctpbuddy` | SDK、CLI、Web、provider、投影 | Python 单测和 e2e |
| `shim/` | 官方 CTP ABI 适配和生成产物 | `codegen/`，不要手改 generated |
| `tests/e2e` | 可复跑的客户端行为量尺 | `wait_idx`、真实退出码 |
| `tools` | 真实资料审计和发布工具 | 统计覆盖范围、不要静默 skip |
| `docs` | 官方原文、考证 notes、项目语义层 | 按资料优先级回查 |

## 修改约束

1. **真实 CTP 优先**：参考 LocalCTP 只能解释结构，不能覆盖真实柜台已证实的缺陷和回报形状。
2. **RefData 不编造**：合约、公司保证金率、手续费和交易参数由用户 provider 提供；缺表意味着 desk 没有这条规则。
3. **生成器优先**：消息 ID 和 Shim 结构体在生成器修改后重生成，不能只改 `generated/`。
4. **确定性优先**：世界循环脉冲是真定时器；journal hash、front/session 对齐和 `vt=0` 不变量不能被方便性破坏。
5. **审计诚实**：真实数据源对齐、受控 ABI 回放、Core 账本重演是三种不同结论，报告必须分开。

## 进一步阅读

- [总体架构]({{< relref "overview" >}})
- [组件职责]({{< relref "components" >}})
- [Wire 与 ADMIN 协议]({{< relref "wire-admin" >}})
- [Web 控制面与安全边界]({{< relref "web-security" >}})
- [项目根 README](https://github.com/charleszu/ctpbuddy/blob/main/README.md)
- [设计文档](https://github.com/charleszu/ctpbuddy/blob/main/DESIGN.md)
