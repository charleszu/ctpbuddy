---
title: 路线图
linkTitle: 路线图
description: 从核心闭环到发布能力的项目路线图。
menus:
  main:
    identifier: roadmap
    weight: 60
---

路线图以仓库根 README 与 DESIGN 为准，本站只保留导航摘要。

## 已完成或已验证

- 仓库骨架、Rust wire/market/matching/ledger/server 基础链路。
- Python SDK、CLI、场景源与端到端冒烟。
- C++ Shim ABI override、布局校验和真实下游 demo。
- 限价簿、FAK/FOK、journal 投影、结算单、OrderSysID 等 M2/M3 项目。
- ZeroMQ ROUTER/DEALER、可选 PUB/SUB/IPC、CURVE/ZAP 白名单与 ADMIN token。
- 墙钟自动日结、到期强平、实时 Web 查询、FTD/连接/交易所限流，以及本机 uv/Docker Desktop 验收。

## 仍在推进

- Web 当前是仅回环、本地控制面：投影只读，写入口白名单化；不等同于完整远程管理后台。
- 完整交易日历仍缺逐日夜盘 ActionDay/TradingDay 权威数据，真实账户整账户回放仍受期权、RefData 与初始结算状态建模限制。
- M4 三渠道发布、正式文档站发布和多角色远程运营仍未完成；Hugo 文档站、Windows release package（含真实产物 opt-in）及 Docker acceptance 已在本机验收，远端 CI 结果本轮暂不核验。

完整状态和每项实现边界见 [`README.md`](https://github.com/charleszu/ctpbuddy/blob/main/README.md) 与 [`DESIGN.md`](https://github.com/charleszu/ctpbuddy/blob/main/DESIGN.md)。
