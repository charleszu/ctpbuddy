---
title: 架构
linkTitle: 架构
description: Shim、Rust 核心、Python 层与行情源的职责边界。
menus:
  main:
    identifier: architecture
    weight: 20
---

CTPBuddy 的核心链路是：下游系统 → C++ Shim → 帧协议 → Rust 核心服务；Python 层负责 CLI、SDK、场景 runner、Web 控制面与行情源插件。

- [总体架构](overview/)
- [组件职责](components/)
