---
title: Wire 与 ADMIN 协议源码
linkTitle: Wire 与 ADMIN 协议
weight: 20
description: 从 frame.rs、msgs.rs 与 admin.rs 反向说明 CTPBuddy 的帧、消息号和控制面边界。
---

本文按当前源码说明协议，不是官方 CTP API 的复制品。主要来源提交为 `2ee8b7e`（M1 wire/server 骨架）、`b9b8fbd`（流控与错误码语义）、`4d2af40`（Web/日结/发布收口）。

## 帧格式

实现位于 `core/ctpbuddy-wire/src/frame.rs`。`Frame::encode()` 写入固定 13 字节小端头：

| 偏移 | 长度 | 字段 |
|---|---:|---|
| 0 | 2 | `MAGIC = [0x43, 0x42]`，即 `CB` |
| 2 | 1 | `WIRE_VERSION = 1` |
| 3 | 2 | `msg_type: u16` |
| 5 | 4 | `req_id: u32` |
| 9 | 4 | payload 长度 |
| 13 | n | payload |

`Frame::new(msg_type, req_id, payload)` 不做业务校验；`encoded_len()` 为 `13 + payload.len()`；`write_to()` 使用 `write_all`。读取时 `read_from()` 在完整帧边界 EOF 返回 `Ok(None)`，半截头也按 `UnexpectedEof` 视为结束；错误 magic、版本或超过 `MAX_PAYLOAD = 1 << 20` 返回 `InvalidData`。因此调用方不能把任意大 JSON 当作合法 ADMIN 请求，也不能依靠半包恢复协议。

Python 对应实现为 `py/ctpbuddy/wire.py:Frame`，字段和限制必须保持一致。协议本身不绑定 TCP；当前 server 用 TCP back-to-back 帧，ZeroMQ 适配仍是待办，不能在文档中写成已完成。

## 消息号与回报面

`core/ctpbuddy-wire/src/msgs.rs` 按区间分配，且声明消息号永不复用：`0x00xx` 为 PING/PONG，`0x01xx` 为会话扩展，`0x02xx` 为 ADMIN，`0x10xx` 为 CTP 请求/回报，`0x20xx+` 为生成结构体 ID。`req_id` 必须由响应回显；主动推送 `RTN_ORDER`、`RTN_TRADE`、`RTN_DEPTH_MD` 使用 `req_id=0`。

失败请求统一使用 `RSP_ERROR` 携带 `CThostFtdcRspInfoField`。有等待中的请求时，Shim 将其转换成对应 `OnRsp*(NULL, pRspInfo, true)`；没有等待项时进入 `OnRspError`。报单交换侧错误另有 `ERR_RTN_ORDER_INSERT` / `ERR_RTN_ORDER_ACTION`，其 payload 还带原输入字段，不能只看 RSP_ERROR。

查询流以 `QRY_LAST` 空 payload 结束，对应 `bIsLast=true`。因此查询客户端必须同时处理数据行、错误行和结束帧，而不能用连接关闭判断查询结束。

## ADMIN JSON

`core/ctpbuddy-server/src/admin.rs:World::on_admin` 处理 `ADMIN_REQ/ADMIN_RSP`，Python 入口是 `py/ctpbuddy/sdk/admin.py:Admin.cmd(name, timeout=None, **kwargs)`。默认地址为 `127.0.0.1:5561`。客户端收到 `ok=false` 会抛 `RuntimeError`，连接关闭或消息号不符是 `ConnectionError`。

可见命令包括：`ping`、`status`、`settings_get`、`settings_update`、`start_scenario`、`pause`、`resume`、`step`、`set_speed`、`seek`、`loop`、`reset_account`、`settlement_report`、`settle_day`、`shutdown`。源码会拒绝缺失/未知 `cmd`；`set_speed` 要求有限 `0..=1000`；`seek` 接受 `HH:MM:SS` 或毫秒数字；未加载场景时控制命令返回错误。

示例：

```python
from ctpbuddy.sdk import Admin
with Admin("127.0.0.1:5561") as admin:
    print(admin.status())
    admin.pause()
    admin.seek("09:00:00")
```

`settle_day` 必须带 `settlement_prices` 对象和严格递增的 `next_trading_day`，还要求 playback 已暂停、没有活动订单；实际原子校验见[账本、FIFO 与日结]({{< relref "/semantics/ledger-fifo-settlement" >}})。ADMIN 不是任意 Rust 方法代理，Web 还会进一步缩减可写命令集合。
