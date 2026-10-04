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

## 仍在推进

- ZeroMQ 传输适配，替换当前 TCP 占位。
- 完整 Web 管理后台和更多断言 CLI。
- M4 三渠道发布与正式文档站发布流程。

完整状态和每项实现边界见 [`README.md`](https://github.com/charleszu/ctpbuddy/blob/main/README.md) 与 [`DESIGN.md`](https://github.com/charleszu/ctpbuddy/blob/main/DESIGN.md)。
