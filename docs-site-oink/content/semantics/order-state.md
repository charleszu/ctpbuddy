---
title: 回报时序与状态机
description: OnRtnOrder、OnRtnTrade 和错误回报的处理边界。
---

订单是否结束不能只看一个回调：应结合订单终态、累计成交量和成交回报累计量。成交回报与终态报单的先后需允许两种顺序；交易所拒单、CTP 拒单和 API 请求未发出也可能落在不同回报面。

CTPBuddy 的 M2 状态机规范和逐交易所差异见语义知识库第 5 节及仓库 [`docs/notes/`](https://github.com/charleszu/ctpbuddy/tree/main/docs/notes)。
