---
title: 组件职责
description: 各层负责什么，以及明确不负责什么。
---

| 组件 | 负责 | 不负责 |
|---|---|---|
| C++ Shim | CTP ABI、结构体编解码、帧收发、SPI 分发 | 业务判断、行情解析 |
| Rust 核心 | 会话路由、虚拟时钟、撮合、账本、结算、持久化 | Web 页面 |
| Python 层 | CLI、SDK、断言、场景 runner、Web、行情插件 | 撮合、账本 |
| 行情源 | 提供用户数据 | 感知柜台协议 |

官方 SDK 头文件不入库；codegen 在本地通过 `CTPBUDDY_SDK` 使用自备版本。
