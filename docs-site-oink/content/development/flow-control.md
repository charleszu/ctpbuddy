---
title: 流控、错误与排障
description: 查询、报撤单、连接和交易所流控的分层摘要。
---

查询流控有两侧：API 侧在途通常为 1 笔，超限返回 `-2`；前置侧按 `QryFreq` 限制每秒查询，超限以 `OnRspError` ErrorID=90 返回。报单与撤单的柜台预算分开计算；连接频率超限会主动断开；用户在线会话数由交易核心限制。

排障时先记录请求返回值，再区分 API 未发出、前置拒绝、柜台拒单和交易所回报。不要把 `sleep` 当作所有流控的解法。

详细表格和 ErrorID 入口见原章 [`09_流控错误与排障.md`](https://github.com/charleszu/ctpbuddy/blob/main/docs/CTP%E5%BC%80%E5%8F%91%E7%9F%A5%E8%AF%86%E5%BA%93/09_%E6%B5%81控错误与排障.md)。
