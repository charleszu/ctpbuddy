---
title: CTPBuddy
description: 本地、确定性、可注入的 CTP 兼容仿真交易环境。
---

CTPBuddy 把 CTP 下游系统接入测试柜台的能力搬到本地：下游可通过同名 DLL Shim 接入，行情可回放、场景可注入、撮合与账本可复现。

- [项目](/project/)
- [架构](/architecture/)
- [开发指南](/development/)
- [语义知识库](/semantics/)
- [API 原文索引](/api/)
- [路线图](/roadmap/)
{.cards}

## 源码反向阅读

- [源码目录与职责]({{< relref "/architecture/source-layout" >}})
- [Wire 与 ADMIN 协议]({{< relref "/architecture/wire-admin" >}})
- [场景、行情源与虚拟时间](/development/scenarios-time/)
- [撮合与交易所回报]({{< relref "/semantics/matching-reports" >}})
- [Ledger、FIFO 与显式日结]({{< relref "/semantics/ledger-fifo-settlement" >}})
- [查询、投影与真实审计边界](/development/queries-audit/)
- [Web 控制面与安全边界]({{< relref "/architecture/web-security" >}})
- [Docker、Shim 与发布工具](/development/docker-shim-release/)
- [测试、CI 与贡献入口](/development/testing-ci-contributing/)

> 本站是仓库文档的 OINK 本地内容层，不复制 `docs/notes/assets/compiled_html/` 下的官方 HTML 大文件。官方原文索引只提供本地入口与版本边界。
