---
title: 连接、认证与会话
description: CTP 客户端初始化和登录链摘要。
---

规范链路为：创建 SPI → 创建 API → 注册 SPI → 注册前置/名字服务器 → 设置公有/私有流 → `Init()` → `OnFrontConnected` → 认证 → 登录 → 结算确认 → `Join()`。

连接建立不等于身份验证。断线后 API 可能自动重连，业务侧仍需重新认证和登录；释放 API 不得发生在 SPI 回调线程。

详细初始化顺序、流文件隔离和断线原因码见原章 [`03_连接登录认证与会话.md`](https://github.com/charleszu/ctpbuddy/blob/main/docs/CTP%E5%BC%80%E5%8F%91%E7%9F%A5%E8%AF%86%E5%BA%93/03_%E8%BF%9E%E6%8E%A5%E7%99%BB%E5%BD%95%E8%AE%A4%E8%AF%81%E4%B8%8E%E4%BC%9A%E8%AF%9D.md)。
