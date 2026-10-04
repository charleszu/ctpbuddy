---
title: API 基础与开发环境
description: CTP Api/Spi 模型、线程边界、请求返回值和流文件。
---

- 交易接口由 `CThostFtdcTraderApi` + `CThostFtdcTraderSpi` 组成，行情接口对应 `MdApi/MdSpi`。
- Api 由静态工厂创建，以 `Release()` 销毁；Spi 由调用方创建并注册。
- 请求接口可多线程调用；SPI 回调运行在 API 工作线程，回调中不要阻塞、睡眠或释放 API。
- `Req*` 返回 0 只代表请求已发出，不代表业务成功；查询需处理 `-2` 在途超限、`-3` 旧版每秒超限。
- `.con` 流文件必须按 API 实例、账号和进程隔离，不能由交易和行情 API 共用。

完整字段和平台差异见原章 [`02_API基础与开发环境.md`](https://github.com/charleszu/ctpbuddy/blob/main/docs/CTP%E5%BC%80%E5%8F%91%E7%9F%A5%E8%AF%86%E5%BA%93/02_API%E5%9F%BA%E7%A1%80%E4%B8%8E%E5%BC%80%E5%8F%91%E7%8E%AF%E5%A2%83.md)。
