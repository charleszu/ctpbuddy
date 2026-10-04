---
title: 总体架构
description: 从仓库 DESIGN.md 提取的架构摘要。
---

```text
下游策略/终端
      │ 同名 DLL 替换
C++ Shim（ABI、帧封装、SPI 分发）
      │ TCP 占位；ZeroMQ 适配待完成
Rust 核心（会话、行情回放、撮合、账本、结算、journal）
      │
Python 层（CLI / SDK / 场景 / Web / 行情源）
```

Shim 不承载业务逻辑；Rust 核心保持确定性单线程撮合和账户单写者边界；Python 控制面不实现撮合与账本。完整设计和 ADR 见仓库根 [`DESIGN.md`](https://github.com/charleszu/ctpbuddy/blob/main/DESIGN.md)。
