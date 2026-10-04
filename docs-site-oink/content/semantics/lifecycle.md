---
title: 生命周期与会话
description: 从连接到认证、登录、结算确认和释放的语义。
---

连接、认证、登录和结算确认是不同阶段。`OnFrontConnected` 只表示链路可达；穿透式环境需要按实际柜台要求认证；交易日首次交易前可能要求确认结算结果。断线重连后需恢复会话状态，不能只重发业务请求。

原始结论与来源见语义知识库第 2 节及 [`03_连接登录认证与会话.md`](https://github.com/charleszu/ctpbuddy/blob/main/docs/CTP%E5%BC%80%E5%8F%91%E7%9F%A5%E8%AF%86%E5%BA%93/03_%E8%BF%9E%E6%8E%A5%E7%99%BB%E5%BD%95%E8%AE%A4%E8%AF%81%E4%B8%8E%E4%BC%9A%E8%AF%9E.md)。
