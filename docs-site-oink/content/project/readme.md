---
title: README 摘要
description: 项目目标、快速入口和当前进展的本地摘要。
---

## 项目解决的问题

SimNow / openctp 依赖远程网络，难以注入极端行情和复现实验。CTPBuddy 以 Shim、Rust 核心和 Python 控制面组成可本地运行的测试柜台。

## 当前状态

M1 核心闭环与 Shim 全链路已完成；限价簿、场景 DSL、journal、查询与结算相关的 M2/M3 能力按根目录 [`README.md`](https://github.com/charleszu/ctpbuddy/blob/main/README.md) 维护。ZeroMQ 适配、完整 Web 后台和 M4 发布工作仍按路线图推进。

## 本地入口

```bash
cd core && cargo build
pip install -e ./py
python tests/test_py.py
```

完整命令、真实数据边界和 SDK 自备要求见仓库根 [README.md](https://github.com/charleszu/ctpbuddy/blob/main/README.md)。
