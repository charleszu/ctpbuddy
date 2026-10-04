---
title: Web 控制面与安全边界
linkTitle: Web 与安全
weight: 30
description: 从 py/ctpbuddy/web.py 说明本地 Web、投影白名单和写入口保护。
---

实现位于 `py/ctpbuddy/web.py`，核心控制仍经 `sdk.Admin` 走 ADMIN 单一真相。主要来源提交为 `9252b91`（本地 Web）、`7ace6ec`（回环别名）、`04f0edc`（回放状态）和 `4d2af40`（结算/审计管理）。

## 启动与只读端点

```bash
ctpbuddy web --host loopback --port 8080 --admin 127.0.0.1:5561 --db ./data/ctpbuddy.db --workspace .
```

`make_server()` 只接受 loopback bind，ADMIN 解析出的所有地址也必须是 loopback。GET 端点包括静态设置页、`/api/session`、`/api/settings`、`/api/scenarios`、`/api/scenario?path=scenarios/rb_demo`、`/api/replay/status` 和 `/api/projection`。

投影 `table` 只能是 `account`、`position_snapshot`、`account_snapshot`、`order_record`、`trade_record`、`audit_log`、`settlement_report`；查询参数只允许 table/broker/investor/trading_day/limit/offset。未配置 DB 返回 404，SQLite/文件故障只返回 503 的通用错误，不回显绝对路径。

## 写入口

POST 只允许 `/api/settings`、`/api/replay`、`/api/scenario/load`、`/api/admin/settlement_report`、`/api/admin/settle_day`。请求必须是单一 Content-Length、`application/json`、body 1..8192 字节；JSON 拒绝重复键、NaN/Infinity 和未知字段。

写请求要求 Host/Origin 同源和 `X-CSRF-Token`，token 来自 `/api/session`；响应设置 `nosniff`、CSP、`Cache-Control: no-store`。路径只允许 workspace 下 `scenarios/<name>` 的直接子目录，拒绝 `..`、反斜杠、盘符、绝对路径、symlink 和越界 RefData。

场景加载必须 `{"path":"scenarios/rb_demo","confirmed":true}`，且当前 playback 要暂停、无活动订单、无持仓保证金；Web 不自动 reset。回放并发由 BoundedSemaphore(8) 和写锁限制，繁忙时返回 429；运行时 ADMIN 不可用返回 503/502。

结算报告最多 16 条、正文最多 512 KiB，GBK 严格编码；日结必须 `confirmed=true`、结算价为正有限数字并显式提供 `next_trading_day`。Web 不开放 reset、shutdown、任意 ADMIN 或 SQL。

## 边界声明

Web 的 live 摘要和 projection 查询是两种状态：projection 返回 `realtime:false`，场景详情还标注初始持仓不是 live 状态；页面不能被理解为完整交易终端或远程生产管理面。当前设计是本机控制面，未实现认证代理、TLS 或公网部署能力。
