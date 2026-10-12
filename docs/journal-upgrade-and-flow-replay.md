# 投影升级与墙钟限流重放

需求来源：2026-10-12 用户要求继续完成“已有 SQLite 投影重建”和“墙钟限流自动完整重放”。实现与验收边界为已确认的 canonical journal contract、公开 CLI 和 Core/CB 报文。

## SQLite 投影升级

```powershell
ctpbuddy journal upgrade D:/counter/data/journal --db D:/counter/data/ctpbuddy.db
```

先关闭使用目标库的 Web/查询进程。升级校验完整 journal，再核对它在旧投影 event_count 所描述的前缀 SHA256。来源不匹配、未知未来版本、损坏录制或活动 WAL/SHM 文件均拒绝替换。旧库通过 SQLite backup 保存到同目录唯一 `.pre-v<版本>-*.bak` 文件，再构建临时数据库并原子替换。失败保留旧库及已有备份；原始 journal 不修改。

同一录制已升级且没有新增事件时返回 `upgraded=false`；录制增长时可再次升级，已投影前缀仍需一致。不存在目标库时直接构建。缺少来源元数据或损坏的旧库不能推断来源，应使用已有显式 `journal rebuild JOURNAL --db DB` 修复。

## 流控自动录制与重放

启用 FTDMaxCommFlux、ConnectFreq 或交易所额度即自动录制。`server_start.data.flow_config` 保存 BrokerID 服务列表、三个额度及逐所覆盖，`settings` 保存查询/报撤预算等启动设置。新增契约事件：

- `flow_input`：Core 连接开启/关闭、TCP/ZMQ/CURVE ConnectFreq 调用与结果、原始 TD 请求、FTD 队列放行。`clock_ns` 是从进程单调钟原点起算的十进制字符串，避免 JSON 浮点损失纳秒精度；不修改业务 vt_ms。
- `flow_output`：逐连接响应与推送的 msg_type、req_id、ErrorID、载荷。AUTH 仅保留成功标记；不要求再次持有原始认证秘密。

AUTH 的 auth_code 录成其有效性标记，额外字段不写入；登录 Password/OneTimePassword 在编码载荷中归零。非法 AUTH 原文不落盘。业务 broker、investor、app_id 与 private_resume 保留。

```powershell
ctpbuddy journal replay D:/counter/data/journal --scenario D:/counter/scenario
```

Walker 检测 trace 后自动启动独立、临时的 `--flow-replay` 核心，隔离调用者的 CTPBUDDY 环境变量，恢复启动额度和设置。ADMIN `flow_replay` 在普通服务中拒绝；离线实例拒绝正常 TD 报文。合成连接保持录制的 FrontID，与 ADMIN 的物理连接编号隔离。

请求按 journal 序重新进入同一 admission/dispatch/gate 路径；排队放行按录制的单调时刻重新执行。ConnectFreq 在独立 transport scope 中重新调用同一个 WindowBudget，检查接受/拒绝结果。真实 10ms PULSE 仍驱动行情与 journal 刷盘；离线 FTD 队列只由录制动作驱动。大于 ADMIN 帧限额的 TD 载荷通过受限大小的本地临时文件传递。

成功同时要求：原始与重放业务 core hash 一致、全部流控输入动作一致、每连接的回调顺序/错误码/载荷一致。跨连接广播遍历顺序不影响判定。报告提供 `flow_replay.verified/inputs/outputs`；不把未核对的 trace 表述为通过。

## 兼容与边界

共享 journal 契约升级至 v3，继续读 v1/v2 原始事件；旧投影可直接升级至当前版本。以前录制没有请求与单调窗口 trace，无法恢复缺失的网络 pacing，需要重新录制。录制以单个 journal run 为单位，不混合多个 server_start。

这里验证 Core/CB 边界的窗口、连接准入、FTD 排队和回调；TCP 操作系统握手、CURVE 密钥认证、网络丢包/传输时延仍由传输验收覆盖，不属于 journal 的业务重放。

## 验收

- `tests/test_projection_upgrade.py`：旧契约升级、备份可读、幂等、来源不匹配与坏录制保留旧库。
- `tests/e2e/flow_replay.py`：TCP 连接拒绝/恢复、查询超限、交易所报撤独立额度、FTD FIFO 与队列饱和、多 Broker/投资者隔离、脱敏、业务 hash 和回调一致；改变回调错误码必须失败。
- `tests/e2e/curve.py`：普通 ZMQ 与真实 native CURVE 的连接限额录制及自动重放；保留真实 C++ Shim 验收。
- Rust/Python shared vectors：v1/v2 兼容、v3 trace 版本边界、精确单调钟字符串、回调载荷 schema。

远端 CI 结果需单独确认，本地通过不代表远端已跑。

2026-10-12 本地验收：152 项 Rust 测试、Python 完整单测、13 项查询断言、投影升级/重建、业务 journal 重放、operations、报撤流控、日结、settings 与 review_regressions 通过；CURVE 使用实际 libzmq 4.3.5/libsodium，并通过真实 C++ Shim。最终流控场景核对 1,093 个输入动作、111 个回调，包含 1 MiB 报文与错误码 116/41/90。本地 `data/ctpbuddy.db` 已从 50 条当前 journal 事件构建到契约 v3。

审查以 `000d322` 为基线，分别核对 README/DESIGN/GLOSSARY/ADR 与上述需求。修正多 Broker 启动配置恢复、精确单调时间编码和录制载荷路径校验。两名独立审查子代理因继承模型不受工具支持未能启动，使用手工审查，未宣称独立审查通过。
