---
title: 报单、撤单与回报
description: 报单入口、订单标识、回报顺序和交易所差异摘要。
---

`ReqOrderInsert` 只表示请求发送成功，不保证柜台接受。订单状态由 `OnRtnOrder` 维护，成交由 `OnRtnTrade` 维护；处理逻辑不能假设最后一笔成交和终态报单的先后顺序。

撤单通常使用 `ExchangeID + OrderSysID`，也要兼容 `FrontID + SessionID + OrderRef + InstrumentID`。FAK/FOK、平今平昨、市价单和错误回报面存在交易所差异，必须回到官方说明与实际环境核验。

详细时序与字段表见原章 [`05_报单撤单与回报.md`](https://github.com/charleszu/ctpbuddy/blob/main/docs/CTP%E5%BC%80%E5%8F%91%E7%9F%A5%E8%AF%86%E5%BA%93/05_%E6%8A%A5%E5%8D%95%E6%92%A4%E5%8D%95%E4%B8%8E%E5%9B%9E%E6%8A%A5.md)。
