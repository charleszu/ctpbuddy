# 本轮四项补缺与验收

本轮沿用 canonical journal contract，完成用户指定的 CURVE/ADMIN 认证、墙钟日结、实时后台和三层新增流控。默认 TCP 与未启用功能的既有行为兼容。

共享契约当前为 v3：v2 登记 `system_announcement`，v3 登记流控输入与回调 trace。新增事件属于 operational noise，不改变业务 core hash。v1/v2 的既有事件继续可读；v3 trace 不能标成 v1/v2。跨语言 golden vectors 验证兼容、字段类型与 canonical 形式。

Core 与 Python 控制面需要使用同一契约版本。已有投影执行 `ctpbuddy journal upgrade JOURNAL --db DB`：先校验原始录制来源，备份旧库，原子重建至当前契约；重复执行相同来源不重复备份。没有数据库时直接构建。原始 v1/v2 journal 不需改写；来源元数据不完整的旧库需显式 `journal rebuild`。详见[投影升级与流控重放](journal-upgrade-and-flow-replay.md)。

## 加密传输与管理认证

TD 使用 `--td zmq://tcp://127.0.0.1:5560` 时，可在**服务端进程**设置：

```text
CTPBUDDY_ZMQ_LIBRARY=<支持 CURVE 的 libzmq 4.x 动态库路径>
CTPBUDDY_ZMQ_CURVE_SECRET_KEY=<服务端 40 字符 Z85 私钥>
CTPBUDDY_ZMQ_CURVE_CLIENT_KEYS=<允许客户端的 Z85 公钥，多个用逗号隔开>
CTPBUDDY_ZMQ_PUB_ADDR=tcp://127.0.0.1:5562
CTPBUDDY_ADMIN_TOKEN=<独立管理令牌>
```

CURVE 通过真实 native libzmq 实现，ROUTER 与 PUB 均启用 ZAP 公钥白名单。无可用库、不支持 CURVE、非法或缺少密钥时启动失败。动态库自身依赖也须可由系统加载；Windows 可把库及依赖所在目录加入 PATH。普通 TCP / 未加密 ZeroMQ 无此依赖。服务端密钥与客户端密钥须分别设置在各自进程，不能把客户端私钥当作服务端私钥。

**Shim 客户端进程**设置同一个 `CTPBUDDY_ZMQ_LIBRARY`，以及 `CTPBUDDY_ZMQ_CURVE_SERVER_KEY`（服务端公钥）、`CTPBUDDY_ZMQ_CURVE_PUBLIC_KEY`、`CTPBUDDY_ZMQ_CURVE_SECRET_KEY`（客户端密钥对）。DEALER/SUB 的密钥选项都在 connect 前应用；配置失败不回落为明文。

PUB 面向可信观测客户端；白名单按客户端公钥授权整个广播端点，**没有按账户细分订阅权限**，因此该名单不能用于互不信任的租户。广播仍为 best-effort，不补发断线历史；CTP 私有流重传仍走 DEALER 会话。

Shim 的行情与私有 SPI 回调统一使用会话 DEALER 推送。可选 SUB 连接不把共享 `md/td` 广播重复注入 SPI，既保留 CTP 合法的前态/新态回报，也遵守当前会话订阅与账户范围；需要观察共享 PUB 的客户端直接使用 ZeroMQ SUB。

配置 ADMIN token 后，每个 TCP 管理连接先发送 `{"cmd":"auth","token":"..."}`，成功后才能执行查询或写命令；`Admin(..., token=...)` SDK 自动完成认证，未显式传入时读取环境变量。Web 后端使用 SDK 连接，浏览器不获得令牌。远程 ADMIN 同时要求 `CTPBUDDY_ALLOW_REMOTE_ADMIN=1` 与非空 token。ADMIN 仍为 TCP 明文，跨机连接通过受信任的隧道；CURVE 不加密 ADMIN。

## 自动日结与到期强平

```text
--auto-settle-at 15:30 --auto-settle-prices D:/counter/prices.json
```

环境变量为 `CTPBUDDY_AUTO_SETTLE_AT`、`CTPBUDDY_AUTO_SETTLE_PRICES`。文件最大 1 MiB，按当日更新（推荐写临时文件后原子替换），内容示例：

```json
{"trading_day":"20261012","settlement_prices":{"rb2601":3501},"next_trading_day":"20261013"}
```

时间使用 Asia/Shanghai 墙钟，仅在当前 TradingDay 等于当日自然日期且达到配置时间时执行。历史场景与提前推进到未来的 TradingDay 不自动跨日；不推断节假日，也不补跑漏过的历史日期。`next_trading_day` 可由已经配置的完整离线交易日历推导，其他情况下必须显式提供。

结算必须暂停 playback、无活动订单，且所有持仓有明确有限正数结算价。文件缺失、日期错误、未暂停等原因使调度等待，每秒重试；ADMIN `auto_settlement.last_error` 显示原因。手动与自动日结共用一个原子结算入口，先在副本校验与强平，再写 journal 与快照。到期条件取自实际 RefData 的 ExpireDate；缺少结算价不会伪造成交价。强平产生真实订单/成交事件，再生成 settlement，保留已有恢复契约。推进后的 TradingDay 与价格文件日期校验阻止重复日结。

## 实时后台

ADMIN 新增 `live_status`，Web 提供 `GET /api/live`；筛选 `broker` / `investor`，`limit=1..200`、`offset>=0`，各集合独立应用同一分页参数。账户、持仓明细、订单最新状态、当日成交均从同一次 World 状态读取，不从 SQLite 推断实时余额。订单按 broker/investor/front/session/order_ref 合并生命周期回报。

页面每两秒刷新，隐藏时暂停；并发请求不重叠、旧请求不覆盖新筛选结果。离线时保留上次表格并明确提示。API 保留回环、Host/Origin 校验、严格参数与有界并发；核心离线返回 503，具体连接信息不回显。历史投影仍通过独立 SQLite 面板浏览。

实时 provenance 包含 `source_kind=realtime_core`、journal_seq、calendar/contract 元数据。实时状态未生成的 journal/core/projection hash 保持 null；不得把序号或随意计算值伪装成 canonical hash。

## 新增流控

| 配置（对应环境变量） | 作用与行为 |
|---|---|
| `--ftd-max-comm-flux` (`CTPBUDDY_FTD_MAX_COMM_FLUX`) | 每 TD 连接每秒总报文预算，超限按到达顺序缓存，PULSE 到新窗口发送；ADMIN 独立。待处理队列最多 1024，队列耗尽返回官方 90；已接受的排队请求不丢弃。 |
| `--connect-freq` (`CTPBUDDY_CONNECT_FREQ`) | TD 每秒新连接预算，TCP 超限关闭；普通 ZMQ 首帧准入超限断开 peer，CURVE 在 ZAP 握手时拒绝。ADMIN 不受此预算限制。 |
| `--exchange-freq` (`CTPBUDDY_EXCHANGE_FREQ`) | 每 broker/investor/exchange 的报单、撤单独立秒级额度。 |
| `--exchange-limits '{"SHFE":20,"DCE":30}'` (`CTPBUDDY_EXCHANGE_LIMITS`) | 六所逐所覆盖统一额度；取消订单省略 ExchangeID 时仍由合约参考数据解析，不绕过限制。超限返回官方 41「CTP:交易所每秒发送请求数超过许可数」；报单先完成前置响应，再发送错单回报与终态拒单，撤单保留双面拒绝回调，均写 journal。 |

以上默认 0，表示关闭新增限流。原有 QryFreq 与 order_freq 保持原有默认和语义。新增参数为启动配置，status 的 `flow_control` 可查看实际值；设置 Web 不提供这些参数的热更新。所有墙钟调度与限流不把墙钟写成 journal 虚拟时间。

启用任一新增流控时，journal 自动记录启动额度、TD 生命周期、脱敏请求、排队放行与回调报文。`journal replay` 自动恢复额度，并以录制单调钟驱动离线预算，无需人工复刻 pacing；同时核对业务 core hash、逐连接回调报文和流控 trace。TCP、普通 ZMQ 与 CURVE ConnectFreq 均覆盖。v1/v2 旧录制缺少这些信息，无法追溯补出完整时序；需重新录制。重放在 Core/CB 边界验证额度行为，不重演操作系统 TCP 握手、ZAP 密钥认证或物理网络延迟。

## 自动验收

- `tests/e2e/operations.py`：真实核心与 HTTP/CB 边界验证令牌、排队时序、连接窗口恢复、交易所/报撤/身份隔离、实时筛选分页、离线状态，以及日期校验、自动强平、单次结算与恢复。
- `tests/e2e/curve.py`：真实 CURVE DEALER/PUB，明文/错误服务端公钥/非白名单客户端拒绝，断线计数与重连；`CTPBUDDY_TEST_CURVE_SHIM=1` 额外运行真实 C++ demo。
- Windows CI 增加 operations；Linux CI 安装 native libzmq 与 pyzmq，执行 CURVE/真实 Shim。缺依赖或验收失败不会静默跳过。本地验收与远端 CI 成功分别记录，不把待跑 CI 称为已成功。

2026-10-12 本地验证：152 项 Rust 测试、clippy/rustfmt、Python 完整单测与 13 项查询断言测试通过；operations、CURVE（含真实 MSVC Shim 与精确回调次数）、m2_flow、m2_journal、m3_settlement、m3_6_projection、settings、m1_shim_e2e、review_regressions 通过。错误码全集生成检查、JavaScript 语法检查通过。Windows CURVE 验收使用实际编译的 libzmq 4.3.5 + libsodium 与 pyzmq；这些临时验证依赖不入库。

规范核对沿用 README/DESIGN、GLOSSARY、ADR 0001 与项目 memory：检查生成器边界、CTP 错误码/回调、真实定时 PULSE、虚拟时间、秘密输出及恢复契约。需求核对按本文件四项和公开 HTTP/CB/CTP 边界验收。`code-review` 要求的两名独立子代理因当前继承模型不受工具支持而未能启动，本次使用手工核对，未宣称独立审查通过。远端 CI 尚未在本轮确认结果。
