---
title: 总体架构
description: 从仓库 DESIGN.md 提取的架构摘要。
---

```mermaid
flowchart TB
  C[下游策略 / 终端 / 条件单]
  S[CTPBuddy Shim<br/>C++ DLL/SO<br/>CTP ABI + SPI]
  TD[TCP TD 前置<br/>默认 127.0.0.1:5560]
  A[ADMIN JSON<br/>默认 127.0.0.1:5561]
  P[Python CLI / SDK / Web<br/>场景编译、回放、设置、日历、投影]

  subgraph R[Rust 核心服务]
    SV[server / wire<br/>连接、认证、路由、流控]
    MK[market<br/>CSV + 虚拟时钟]
    RF[Catalog / RefData<br/>合约、费率、交易参数]
    MT[matching<br/>订单簿与成交]
    LG[ledger<br/>账户、持仓、资金、结算]
    SV --> MT
    SV --> LG
    MK --> MT
    RF --> MT
    RF --> LG
    MT --> LG
  end

  D[外部文件<br/>ticks.csv / scenario.json / refdata/*.jsonl]
  J[data/journal/*.jsonl]
  Q[data/ctpbuddy.db<br/>journal 投影]

  C <-->|CTP API| S
  S <-->|CB 帧 over TCP| TD
  TD <--> SV
  P <--> A
  A <--> SV
  D --> MK
  D --> RF
  SV --> J
  LG --> J
  J -->|rebuild| Q
```

TD 是下游 CTP 应用的数据面，ADMIN 是 Python 控制面；两者使用独立端口。当前核心使用明文 TCP 承载 CB 帧，ZeroMQ 仍是后续传输适配方向。Shim 不承载业务逻辑；Rust 核心保持确定性撮合和账户单写者边界；Python 控制面不实现撮合与账本。

完整设计和 ADR 见仓库根 [`DESIGN.md`](https://github.com/charleszu/ctpbuddy/blob/main/DESIGN.md)。
