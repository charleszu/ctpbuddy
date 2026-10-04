---
title: 流控全景
description: API、前置、柜台与交易所的流控边界。
---

CTP 流控不是单一阈值：

1. API 侧查询在途限制，典型返回 `-2`。
2. 前置侧 `QryFreq` 超限，典型 ErrorID=90。
3. 柜台侧报单/撤单频率通常分别计算。
4. FTD 报文、连接频率、在线会话数和交易所 API 还有独立边界。

CTPBuddy 通过 Shim 复刻 API 侧，通过 Core 复刻前置/柜台侧；尚未实现的范围在根 README 和 DESIGN 中明确标注。
