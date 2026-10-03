# CTP 语义知识库（项目存档）

> 本文件是 CTPBuddy 项目的 CTP 语义权威存档，供后续 Agent 与开发者查阅。
> 内容来自 `C:\workspace\src\CTP\docs`（40+ 份官方 SDK 文档/教材/坑指/入门系列）、SDK 自带 CHM《6.7.13_API接口说明》与参考实现 `LocalCTP` 的系统化通读，逐条注明出处；**冲突处以官方 CHM/API 说明为准**。
> 深度细节（含原文引用、算例、代码行号）在 `docs/notes/` 下分册；本文件是索引与结论层。通用开发指南和速查表见 [`CTP开发知识库/README.md`](CTP开发知识库/README.md)，仅作非官方权威的分层入口；官方 HTML、notes 与真实环境核验优先。
> 官方资料可读版：SDK《6.7.13_API接口说明》CHM 已转为干净 HTML（405 页）在 [`docs/api-doc-html/`](api-doc-html/)，SDK 错误码全集（error.xml 299 条）可读表在 [`docs/错误码全集.md`](错误码全集.md)。
>
> **两类资料的地位不同，必须区分**（这是本库的一条组织原则）：
> - **官方文档类**（SDK CHM、API 特别说明、报单回调规则、爬坑指北、期货公司投教 PDF）——**口径以此为准**。
> - **客户端实操类**（`docs/程序化交易入门/` 全套 17 份、《之十一/十四》费率与保证金）——提供**第一手客户端视角的流程、字段坑与算例**，与官方文档冲突时以官方为准，但**官方未言明的细节（字段级坑、返回码含义、盘中无效值）往往只有这里有**，且已被真实数据验证过的部分（如昨仓盈亏口径）可作为官方背书引用。分册 [`notes/11`](notes/11-入门系列-连接认证与穿透式监管.md)~[`notes/14`](notes/14-入门系列-查询流控与持仓查询更新.md) 属此类。

---

## 0. 资料来源与阅读覆盖

| 资料 | 主题 | 状态 |
|---|---|---|
| SDK CHM《6.7.13_API接口说明》（ctpsdk/6.7.13_20260225/docs） | 接口逐函数说明、报单回调规则、各交易所特殊指令、条件单规则、流控 | 已通读（流控页为权威来源）；全量 HTML 版（405 页）见 [`docs/api-doc-html/`](api-doc-html/) |
| SDK error.xml（td/md 各一份 + TGate 变体 error_tgate.xml） | 客户端必须处理的错误码全集（299 条） | 已解码为可读表 [`docs/错误码全集.md`](错误码全集.md)，核心常量按其对账（§10.2） |
| 综合交易平台API技术开发指南.pdf | API 开发规范、时序、FAQ | 已通读 |
| 综合交易平台交易API特别说明.pdf | 初始化顺序、两阶段响应、多会话 | 已通读 |
| 综合交易平台API开发常见问题列表.pdf | 2009 上期技术 FAQ #1~#45 | 已通读 |
| CTP客户端开发指南.pdf / CTP开发——基础.pdf | 生命周期、流程图、席位制 | 已通读 |
| CTP报单指令详解 - 信易科技.pdf | 各交易所 × 指令类型四字段组合表 | 已通读 |
| 报单回调规则.pdf（CTP 6.7.2 API 说明内置） | 7 场景固定回调顺序、大商所特例 | 已通读 |
| 四家交易所交易指令整理.pdf / 四所_FAK_FOK_文件汇编/ | 市价单/止损止盈/套利差异、FAK-FOK 定义与算例、异常交易监管 | 已通读 |
| 定单状态.png / 行情模式.png | OrderStatus/OrderSubmitStatus 九态七态图、行情传输模式图 | 已通读 |
| CTP开发爬坑指北 一~四 / CTP爬坑指北 五~七 | 连接/报单/资金/行情坑 | 已通读 |
| CTP账户持仓和资金的维护.pdf（2023，开心秋水） | 逐日盯市资金/持仓维护口径 | 已通读 |
| 组合合约持仓计算规则.pdf | 组合持仓 3 记录、打散规则 | 已通读 |
| ctp期权保证金手续费算法说明.pdf（6.3.0，上期技术） | 期权保证金/手续费 | 已通读 |
| CTP程序化交易入门系列之十一/十四 | 费率查询、保证金计算与优惠 | 已通读（原始位置 `docs/程序化交易入门/`） |
| **CTP程序化交易入门系列 之一~十五（全套 17 份，`docs/程序化交易入门/`）** | **连接/认证/穿透式监管、API 架构与初始化、行情切片与 K 线合成、现手增仓开平对手盘、报单字段与回报、撤单、查询流控、成交回报、持仓与持仓明细查询更新、4097 断连原因码** | **已通读**（图像型 PDF，逐页视觉阅读）。分册：[`notes/11`](notes/11-入门系列-连接认证与穿透式监管.md) 连接认证穿透式 / [`notes/12`](notes/12-入门系列-行情现手与开平判定.md) 行情现手开平 / [`notes/13`](notes/13-入门系列-报撤单与成交回报.md) 报撤单成交回报 / [`notes/14`](notes/14-入门系列-查询流控与持仓查询更新.md) 查询流控与持仓查询；实现影响清单见各册末节与 §10.4 |
| CTP行情订阅常见问题解答.pdf | 订阅语义、TradingDay、字段坑 | 已通读 |
| LocalCTP（C:\workspace\src\CTP\LocalCTP） | 进程内仿 CTP 交易柜台参考实现 | 已通读（结构/流程/对账点） |
| 未读（低优先级，FIX/银期/培训类）：CTP FIX接口使用规范说明.docx、CTP FIX网关说明.docx、CTP新版银期转帐TradeApi使用说明.pdf、综合交易平台交易银期功能特别说明.doc、api培训-二期.pptx、20150422_上期技术_API培训-改.pptx、CTP API终端开发交流讨论会 pdf、CTP国际版开发指南.pdf、CTP_API开发文档资料/、综合交易平台简介.pdf | FIX 网关、银期、培训 | **未通读**，需要时再补（见 §12 待办） |

---

## 1. 流控全景（六类）与 CTPBuddy 落点

权威来源：SDK CHM《报单流控、查询流控和会话数控制》。CTP 交易系统有多层流控，分布在客户端 API、交易前置、柜台/核心、交易所多处：

| # | 流控 | 配置位置 | 超限症状 | CTPBuddy 落点 |
|---|---|---|---|---|
| 1 | 报单/撤单每秒笔数 | **CTP 柜台端**【程序化交易频繁报撤单管理】 | `OnRspOrderAction`「CTP:下单频率限制」 | **Core** ✅ M2-4 落地（`order_gate`，`--order-freq`，每 (broker,investor) 每秒预算，116 号；**报/撤两条独立流分开计算**，2026-10-03 修正） |
| 2 | 查询每秒笔数 QryFreq | **交易前置** front_se（穿透式监管版本起，API 连接前置时读取该配置；历史上内置在 API 里 1 笔/秒） | `OnRspError`[90]「CTP：查询未就绪，请稍后重试」，查询不执行 | **Core** `--qry-freq`（默认 2）✅已实现 |
| 3 | 查询在途笔数 = 1 笔 | **客户端 API 内置**（永远存在，与前置配置无关） | 查询函数**返回值 -2**「未处理请求超过许可数」，请求不上线 | **Shim** 在途闸门 ✅已实现 |
| 4 | FTD 报文流控 FTDMaxCommFlux | 交易前置 | 无错误，超限报文在前置缓存延迟到下一秒 | Core TODO |
| 5 | 前置连接数 ConnectFreq | 交易前置（同 IP 每秒连接数；白名单 whiteiplist） | 主动断开 → `OnFrontDisconnected` | Core TODO |
| 6 | 同一用户最大在线会话数 | 柜台/交易核心（6.6.3+ 按用户维度） | `OnRspUserLogin`「CTP:用户在线会话超出上限」 | 已实现：`max_user_sessions`，0 关闭，按 BrokerID+UserID，降低不踢已有会话 |
| 7 | 交易所 API 流控 | 交易所端 | `OnRtnOrder` 报「CTP：交易所每秒发送请求数超过许可数」/「未处理请求超过许可数」 | Core TODO |

关键语义：
- **查询流控是两侧的**：在途 1 笔在客户端 API（Shim，返回 -2，这正是「1s 只能有一个在途查询」现象的出处）；每秒 QryFreq 在前置侧（Core，90 号错误）。穿透式监管版本后 API 只在连接时读取前置配置，执行仍在前置。**只做一半会让下游客户端行为与真实 CTP 漂移**（掩盖不会处理 NEED_RETRY 的客户端缺陷）。
- **投资者受限、操作员不受限**（查询流控）。
- **所有 `ReqQuery*` 开头的函数不受查询流控限制**——走交易核心不经查询核心。
- 报单流控归柜台（用户判断正确）：实现在 Core 侧 M2 风控规则表；客户端侧表现只有「排队不报错」（见 §3.5）。
- **在途超限（-2）用 sleep 治不好，超频（-3/90）用 sleep 才有效**（notes/14 §A.2）。机理：CTP 的 API 是**异步单底线程**架构——请求从缓存读出、经 socket 发出，**socket 读写与 spi 回调由底层同一线程负责**；因此在 spi 回调里 `sleep` 会阻塞读写，查询最终仍由 socket 同时发出。正确解法是**一个 spi 回调只放一个查询并串行等待响应**（或事件队列/独立线程）。**这正是本项目 SDK `_query_stream` 透明重试（串行化 + 90→1.1s 退避）之所以有效的根本原因**，也是「在途闸门」必须复刻而非视作人为限制的依据。返回 `-3`（每秒超限）是**旧穿透式版本**行为，新版本改为后台配置、经登录回报传给动态库（notes/14 §A.2）。
- **报撤单流控的官方口径是「分开计算」**（notes/13/14 §A.3）：`ReqOrderInsert` 与 `ReqOrderAction` **各自**有每秒最大笔数、**分开计算**。M2-4 原实现为共享预算，**2026-10-03 已修正为两条独立预算流**（`GateStream::Insert/Cancel` 各自计数，e2e `m2_flow` Phase 1 锁定）。
- **前置连接数流控与自动重连会互相放大**（notes/14 §A.3-04）：超限时前置**主动断开**并回调 `OnFrontDisconnected`；若原因是连接数超限，立刻重连会**再被立刻踢**，形成循环。诊断时应先排除该情形（按 IP 分别配置，simnow 有多套地址可切换），退避不能只按"网络抖动"处理。

历史口径（2009 FAQ 时代）：查询每秒 1 次 + 在途 1 个，交易指令默认每会话 6 笔/秒、同账户最多 6 会话，超限**排队不报错**（【技术指南】Q19、客户端指南 4.14）。注意这是「无错误」与「90/-2」两种形态的区别：查询超限有明确拒绝，交易指令超限只排队。

### 1.0.1 断连原因码 `nReason`（notes/11 §E.2，SDK 头文件口径）

`OnFrontDisconnected` 的 `nReason` 取值——**诊断 4097 循环的第一依据**：

| 十六进制 | 十进制 | 含义 |
|---|---|---|
| `0x1001` | **4097** | 网络读失败（最常见） |
| `0x1002` | 4098 | 网络写失败 |
| `0x2001` | **8193** | 接收心跳超时 |
| `0x2002` | 8194 | 发送心跳失败 |
| `0x2003` | 8195 | 收到错误报文 |

**持续 4097 循环的首要成因是「版本不匹配」而非网络抖动**（notes/11 §E3）：CTP 要求 API 版本与后台版本一致，版本不对会**不停回调 `OnFrontDisconnected`**，或输出 `Decrypt handshake data failed`，或**毫无反应**；此时**重连无用**。检查 `GetApiVersion()`。已知期货公司生产与 simnow 为 **v6.3.15_20190220**。真实 CTP 上 4097 频繁出现的另一常见原因是 **simnow 用户多导致前置拥堵**，以及 **simnow 7×24 地址不稳定**。


### 1.1 自动重连（用户关注的「连接不上有没有重试」）

权威来源 CHM `OnFrontDisconnected` 页原文：
> 「当发生这个情况后，**API 会自动重新连接，客户端可不做处理**。自动重连地址，可能是原来注册的地址，也可能是系统支持的其它可用的通信地址，它由程序自动选择。**注：重连之后需要重新认证、登录**。**6.7.9 及以后版本中，断线自动重连的时间间隔为固定 1 秒。**」

补充（`RegisterFront` 页）：可多次注册不同前置地址形成地址池，断线时自动择优（最先建立 TCP 连接的地址）；SSL 前置 `ssl://ip:port`、IPv6 `tcp6://`、代理 `socks5://user:pass@host:port`。

断开原因码（nReason 为十进制，需转十六进制比照）：

| 值 | 含义 |
|---|---|
| 0x1001(4097) | 网络读失败 recv=-1 |
| 0x1002(4098) | 网络写失败 send=-1 |
| 0x2001(8193) | 接收心跳超时（前置每 53s 心跳，API 超 120s 未收到任何数据即断） |
| 0x2002(8194) | 发送心跳失败（API 每 15s 心跳，超 40s 无发送即断） |
| 0x2003(8195) | 收到不能识别的错误报文 |

结论：**真实 CTP 的「连接重试」由 API 内置完成**（固定 1s 间隔自动重连，多地址择优），客户端框架的职责是「重连后重新认证+登录」；连接被拒/被踢（如 ConnectFreq 超限）同样表现为 OnFrontDisconnected，之后也会进入自动重连。另注意：SPI 回调线程内禁止调 Release（崩溃），建议 logout→等自动重连→重新登录复用 API 实例（`Release` 页 FAQ）。

---

## 2. 客户端生命周期与连接管理

### 2.1 规范初始化七步（【客户端指南】2.2.4）

```
创建 SPI 实例 → CreateFtdcTraderApi(流文件目录) → RegisterSpi
  → RegisterFront(可多个，地址池) / RegisterNameServer(名字服务器)
  → SubscribePublicTopic(公有流) → SubscribePrivateTopic(私有流)
  → Init() → [TCP连接] → OnFrontConnected / OnFrontDisconnected(nReason)
  → Join()
```

登录链：`OnFrontConnected → ReqAuthenticate → OnRspAuthenticate → ReqUserLogin → 每交易日首次交易前 ReqSettlementInfoConfirm`。连接建立只代表链路可达，**无身份验证**；真实柜台若启用强制认证则必须先过认证。CTPBuddy 当前没有授权配置，不宣称实现柜台侧强制策略。CTPBuddy 的标准 `ReqAuthenticate` 已接入现有 JSON AUTH handshake；显式认证成功后 Shim 保存并绑定 `BrokerID/UserID`，`ReqUserLogin` 必须匹配，服务端还会对连接绑定再次校验，拒绝 Auth A → Login B。`AuthCode/AppID` 仅作协议字段透传，当前核心不伪造柜台授权比对。由 `ReqUserLogin` 自动触发的兼容 AUTH 不额外产生 `OnRspAuthenticate`。本地空字段错误使用官方 `15 BAD_FIELD`，未认证登录使用 `64 NOT_AUTHENT`；`auth_in_flight` 时 `ReqAuthenticate` 同步返回 `-2`，不会先返回 0 再异步 `-3`。已认证重复认证与断线认证失败没有确认精确柜台重复码，统一采用已实现 `63 AUTH_FAILED`（`CTP:客户端认证失败`）兼容策略，网络原因另由 `OnFrontDisconnected` 报告。认证响应回填官方 `UserProductInfo`；`AppType` 未配置，不编造标值。

释放顺序（【技术指南】Q22，防死锁）：`RegisterSpi(NULL) → Release() → 置空指针 → delete SPI`。**禁止在 SPI 回调线程内 Release**。创建与释放最好在同一线程。

**客户端实操侧的两条补充（notes/11 §D3、§C3）**：
- **认证必须在登录之前**："在 `ReqUserLogin` 之前需要先调用 `ReqAuthenticate`"，且**发起位置是 `OnFrontConnected` 回调内**，在 `OnRspAuthenticate` 中判 `ErrorID == 0` 后再 `ReqUserLogin()`，否则不发登录。
- **认证字段是 `AppID` + `AuthCode`，`UserProductInfo` 已废弃**（原文明确"现在 `UserProductInfo` 废弃不用，改为 `AppID` 了"）。`AppID` 由客户自己命名、监控中心给规范（如 `client_abcdf_1.0.0` / `pobo_iee_1.4.0.0` / `poboAPP_ijsse_2.0.0.0`），`AuthCode` 由期货公司发给客户，**两者一一对应**。这是穿透式监管（notes/11 §D）的产物：监控中心采集所有入场客户的本地终端信息，用其下发的公钥在 API 底层加密，**只有监控中心持有私钥**，柜台/期货公司/交易所都看不到明文。
- **异步单底线程架构**（notes/11 §C1）：请求读取（缓存）与 socket 读写由**同一底层线程**负责，这是一系列流控行为的根因（见 §1 关键语义）。
- **注意 `Join()` 会阻塞主线程**：需要非阻塞行为（如 `sleep` 代替）时，调用方必须自行保证连接已建立前不推进业务。

### 2.2 三种通讯模式与流文件（【技术指南】Q10/Q11、客户端指南 1.2/4.13）

| 模式 | 方向 | 可靠性 |
|---|---|---|
| 对话通讯 | 客户端主动、服务端响应（报单/撤单/查询） | 后台不维护状态，故障重置、途中数据可能丢失 |
| 私有通讯 | 后台主动发给特定客户端（报单回报、成交回报） | 可靠流，断线恢复后 restart 重传当日全部 / resume 续传 |
| 广播通讯 | 后台发给所有客户端（合约状态通知） | 可靠流 |

- 流文件 `.con`：接口初始化时本地生成，记录各类报文当日计数供 Resume 续传；**多账号/多实例绝不可共用流文件目录**（数据紊乱：一个账号收到回报其他收不到）；TradeApi 与 MdApi 两个 dll 放同一目录会互相覆盖流文件——必须给各自不同的流文件路径。
- TERT 三模式：`TERT_RESTART` 从本交易日开始重传（登录后重推旧单就是它导致的）/ `TERT_RESUME`（默认）从上次收到的续传 / `TERT_QUICK` 只传登录后的内容。

### 2.3 线程模型

API 与 SPI 在不同线程；**API（Req*）可被多线程同时调用**（线程安全，平台无关）；SPI 回调在独立线程，业务侧应入队转业务线程，不在 SPI 内阻塞；SPI 内禁止 Release。`OnHeartBeatWarning` 永不回调（已对 API 用户屏蔽）。

### 2.4 柜台职责划分（实现兼容柜台的架构基线）

交易前置 = 链路管理/协议转换/数据路由（不含业务）；交易核心 = 事前风控+报单校验+资金仓位实时计算；排队服务 = 请求序列化+精确重演；报盘管理 = 交易所 API 交互。**客户端的所有返回包都由柜台归一化后发出**，交易所差异（平今转换、行情字段校正、组合合约编号）由柜台消化，客户端不应感知。

---

## 3. 账号、会话与序号

- `BrokerID` 业务层隔离经纪公司；`UserID` = 操作员代码（代客下单场景），`InvestorID` = 投资者代码；**投资者自己下单时两者相同，报单都必须填有效 UserID**——不填 UserID 的报单被拒后收不到 `OnErrRtnOrderInsert`（拒单通知按报单 UserID 推送）。
- 登录响应给 `FrontID`（前置编号）、`SessionID`（会话编号）、`MaxOrderRef`。**FrontID+SessionID 变更（重连/登出重登）后 MaxOrderRef 重置**；OrderRef 有符号 32 位整数（读作字符串、写作整数），**同一会话内必须严格递增**（多线程尤其注意），超范围会被判 0 报「重复的报单」。
- 三组序号与撤单 key：
  1. `ExchangeID + OrderSysID`（交易所生成，**推荐**；注意报单第一个回报里 OrderSysID 为空不能拿来撤单；数字字符串可能右对齐带空格，**不要截断**）；
  2. `FrontID + SessionID + OrderRef`（客户端自维护；部分交易所在订单进队列前不支持这种撤单）；
  3. `ExchangeID + TraderID + OrderLocalID`（CTP 核心生成，TraderID 是交易所交易员代码≠TradeID）。
- 请求侧返回码：`0` 发送成功（仅代表发出）/ `-1` 网络原因失败 / `-2` 未处理请求队列总数量超限 / `-3` 每秒发送请求数超限。
- 多会话：同一投资者账号可多个 TradeApi 实例同时登录；回报用 FrontID+SessionID 过滤本会话单子；`RequestID` 允许重复、无法区分同账户不同会话。
- 流缓存订阅时选错 TERT_RESTART 会重推历史回报；客户端的「报单两阶段、成交以 OnRtnTrade 为准」等语义见 §5。

---

## 4. 报单/撤单语义

### 4.1 指令类型 = 四字段组合（信易.pdf 全表照录于 notes/01）

| 指令 | OrderPriceType | TimeCondition | VolumeCondition | 备注 |
|---|---|---|---|---|
| 限价单 | LimitPrice | GFD | AV | |
| 市价单 | AnyPrice | IOC（郑商/大商 GFD；中金撤销型 IOC、转限价型 GFD） | AV | LimitPrice=0；郑商所价格非 0 会被拒 |
| FAK 任意成交数量 | LimitPrice（郑商所可 AnyPrice） | IOC | AV | 能成交多少成交多少，剩余撤销 |
| FAK 指定成交数量 | LimitPrice | IOC | MV + MinVolume | **仅上期所/中金所**；可成交量 < MinVolume 则整笔撤销 |
| FOK | LimitPrice | IOC | CV | 不能全部成交则整笔撤销 |
| 止损/止盈 | AnyPrice | GFD | AV | **仅大商所**：CC_Touch/CC_TouchProfit + StopPrice，触发后另下新报单（CTP 侧 OrderSysID 带 `TJBD_` 前缀） |
| 中金所五档市价/最优价 | FiveLevelPrice/BestPrice | IOC 撤销型 / GFD 转限价型 | AV | 仅中金所 |

- **FAK/FOK 不是单字段而是 TC+VC 组合**；FOK=`IOC+CV`，FAK=`IOC+AV` 或 `IOC+MV`。
- 各交易所特殊指令差异：上期所/大商所/中金所「FAK、FOK、GFD」；郑商所官方口径（郑商函〔2014〕333 号）**期货仅 FAK**（整理表「FOK」格与官方矛盾，以官方为准）；郑商所另有跨品种套利与「一开一平」。
- 市价单差异：上期所**生产环境暂不支持**（测试环境已测）；大商所内部转涨跌停限价撮合，**成交价可能不等于对手价**；郑商所/中金所按最优对手价。
- FAK/FOK：不得用于集合竞价；造成的自成交和撤单**不计入异常交易**（大商所明文；上期/郑商同；中金所仅豁免套利套保编码）。异常交易监管标准（自成交/频繁报撤单 ≥500 次日起步等）见 notes/01 D6。

### 4.2 报单固定值与校验（API特别说明 §1）

`VolumeCondition=AV`、`MinVolume=1`、`ForceCloseReason=FCC_NotForceClose`、`IsAutoSuspend=0`、`UserForceClose=0`；`CombOffsetFlag/CombHedgeFlag` 是长度 5 数组但当前只填 `[0]`。立即市价单 = AnyPrice+LimitPrice=0+IOC。

### 4.3 平仓与开平标志

`CombOffsetFlag` 枚举：Open'0'/Close'1'/ForceClose'2'/CloseToday'3'/CloseYesterday'4'/ForceOff'5'/LocalForceClose'6'。上期所 Close 等同平昨；其他交易所 CloseToday/CloseYesterday 后台统一转为 Close。平仓顺序：四所统一先开先平；郑商所先单腿后组合；除上期所外三家涉平今手续费减免时先平今（后开先平）。**成交回报的开平方向 ≠ 报单开平方向**：大商所/郑商所/中金所平仓一律回 Close('1')，只有上期所/能源中心区分（SimNow 走上期所规则，仿真与实盘不一致的坑）。

### 4.4 撤单

`ReqOrderAction` 的 `ActionFlag` 只支持 `THOST_FTDC_AF_Delete`（**仅撤单**，挂起/激活/修改「无此功能」——即"CTP:无此功能"的一类真实原因）。GTC 过夜挂单**不支持**（国内交易所不支持的语义直接拒绝，不要静默丢弃）。

**撤单的必填字段（notes/13 §D2，客户端实操口径）**：

| 组 | 字段 | 注意 |
|---|---|---|
| 账号 | `BrokerID` + `InvestorID` | 必填 |
| | `UserID` | **非必填**，但**不填会收不到 `OnErrRtnOrderAction` 回报** |
| 定位 | `ExchangeID` + `OrderSysID` | **第 1 次报单回报中 `OrderSysID` 为空，此时只能改用下一组** |
| 定位 | `FrontID` + `SessionID` + `OrderRef` | **用这组撤单时必须填 `InstrumentID`** ← 具体的字段校验规则 |

`InvestUnitID` 虽在结构体中，但因子账号分组功能被判违规、已**废弃不用**（notes/13 §D2）。官方错误码 25（已全成交或已撤销）与 26（找不到相应报单）的区分源于**上期所「报单已经全部成交」**这一特殊情形：单子已在交易所撮合成交但 CTP 尚未收到成交通知，撤单过了 CTP 检查却被交易所拒绝（notes/13 §D4）——这正是 M2-4 需要 `terminal_refs` 终态记忆的成因。

**批量撤单**：`ReqBatchOrderAction` 看似批量撤单，但**仅支持大商所且仅做市商客户可用**，故**当前没有批量撤单**，须客户端自行实现（notes/13 §D3）。此外**撤单数风控（单合约 500 笔）CTP 内并没有**，各交易所按异常交易管理规范，**需策略自行加入**；**FAK/FOK 的撤单不计入该数**，故可能大量撤单的策略建议用 FAK/FOK 报单。

### 4.5 报单流控

柜台端【程序化交易频繁报撤单管理】配置每秒最大报撤笔数，超限 `OnRspOrderAction`「CTP:下单频率限制」（官方错误码 **116 ORDER_FREQ_LIMIT**，勿与 91 EXCHANGE_RTNERROR「CTP：交易所返回的错误」混用）。注意与 2009 FAQ「默认 6 笔/秒、超限排队不报错」的历史口径区分——现代柜台是显式拒绝。

**报单与撤单分开计算**：官方口径为 `ReqOrderInsert` 与 `ReqOrderAction` **各自**有每秒最大笔数、**分开计算**（notes/13/14 §A.3）。CTPBuddy `order_gate` **已按此实现**（两条独立预算流，2026-10-03 修正；此前的共享预算会让混做报撤的客户端被误限）。

CTPBuddy 落点：Core M2 风控规则表 ✅（M2-4 落地，口径待按上述修正）。

---

## 5. 回报时序与状态机（M2 撮合/回报的核心规范）

### 5.1 回调分流表

「CTP 层 / 交易所层」是官方 API 文档的分层口径（#42 落地，见 [DESIGN §8.12](../DESIGN.md)、
[notes/09](notes/09-错单推送面与错误码对账.md)）：**报单被拒推几个回调，取决于在哪一层被拒**。

| 回调 | 触发场景 | CTPBuddy 线上帧 |
|---|---|---|
| `OnRspOrderInsert` | ①**CTP 层拒绝**报单（会话/字段/风控/流控，`pInputOrder` 为 **NULL**）；②**交易所层拒绝**时的**成功响应**（`pInputOrder` 非 NULL、ErrorID=0）——错误在随后的 `OnErrRtnOrderInsert` 里 | ① `RSP_ERROR` only；② `RSP_ORDER_INSERT` 成功帧 |
| `OnErrRtnOrderInsert` | **交易所层拒绝**后的错单回报（163/164/165）；载荷是 composite：客户端自己的 `InputOrderField` ++ `RspInfo`（不填 UserID 时收不到） | `ERR_RTN_ORDER_INSERT`（composite） |
| `OnRtnOrder` | 报单状态回报（每笔交易所状态迁移推「前态+新态」两条） | `RTN_ORDER` |
| `OnRtnTrade` | 成交回报（每笔成交一次；无 FrontID/SessionID，用 OrderSysID 反查订单；`Volume` 只是本笔，总量看 `OnRtnOrder.VolumeTraded`） | `RTN_TRADE` |
| `OnRspOrderAction` | 撤单被拒（25/26/116/23）——**响应半面，先到**，`pInputOrderAction` 为 NULL | `RSP_ERROR` |
| `OnErrRtnOrderAction` | 撤单被拒的**回报半面，紧跟响应**（官方场景 6/7，**成对出现**，缺一挂钩不全）；载荷 composite：`InputOrderActionField` ++ `RspInfo` | `ERR_RTN_ORDER_ACTION`（composite） |

关键陷阱：**只挂 `OnRspOrderInsert` 的客户端会漏掉全部 163/164/165**（那半边是「成功」响应），静默漏单。仿真若把交易所层拒单也做成 `RSP_ERROR`，这个 bug 就会被掩盖。

### 5.2 固定回调顺序（【报单回调规则.pdf】CTP 6.7.2 内置文档，ag1207 场景照录于 notes/01）

「在给定交易所报单回报顺序的前提下，CTP 返回给 API 的回调顺序是固定的。」归纳模式：

1. 报单提交后先推一笔「未知单('a')」`OnRtnOrder`；此后交易所每来一笔状态迁移，CTP 先补推一笔「前状态」`OnRtnOrder`、再推「新状态」——**同一报单会收到重复的前态回报，按条数计数会双倍，必须按 OrderStatus 去重/幂等**。
2. `OnRtnTrade` 总在新状态的 `OnRtnOrder` **之后**。
3. 立即全部成交（未收到未成交回报）时连推两笔未知单再全部成交；先未成交后成交、部分成交、撤单、对已撤单再撤、撤不存在的单共 7 场景的精确序列见 notes/01 B2。

**大商所特例**：全部成交后大商所只返成交回报、不返全部成交报单回报，由 CTP **自补**全部成交报单回报且不重复前态；且大商所只要委托进报单簿必返未成交回报（即使立即成交）。跨所用同一套成交判定逻辑必须为大商所开特例。

### 5.3 OrderStatus 九态（【定单状态】图照录）

| 值 | 含义 | 备注 |
|---|---|---|
| '0' | 全部成交 | 终态 |
| '1' | 部分成交还在队列中 | |
| '2' | 部分成交不在队列中 | 暂不使用 |
| '3' | 未成交还在队列中 | 收到交易所报单确认 |
| '4' | 未成交不在队列中 | 暂不使用 |
| '5' | 撤单 | 撤单成功/废单；终态 |
| 'a' | 未知 | CTP 已收单并发给交易所，未收到确认（含废单） |
| 'b' | 尚未触发 | 条件单保留，暂不使用 |
| 'c' | 已触发 | 条件单保留，暂不使用 |

### 5.4 OrderSubmitStatus 七态（针对指令：报单申报/撤单申报）

'0'报单已提交 / '1'撤单已提交 / '2'修改已提交（暂不使用）/ '3'已经接受（交易所接收）/ '4'报单已被拒绝（CTP 报盘或交易所拒绝）/ '5'撤单已被拒绝 / '6'改单已被拒绝（暂不使用）。

终态四个：AllTraded / Canceled / NoTradeNotQueueing / PartTradedNotQueueing。「部成部撤」终态即 Canceled。「不在队列中」是上期所自动挂起标志的历史产物，**官方要求 IsAutoSuspend 恒设 0，永不挂起**。

### 5.5 成交判定与撤单来源

- **成交判断必须以 `OnRtnTrade` 为准**（核心收到成交回报才更新报单状态）；以 OnRtnOrder 判成交并立即平仓，极小概率平仓指令到达时报单状态未更新导致平仓失败。
- 撤单来源区分：`OrderSysID` 非空 = 进过交易所撮合队列的自撤；空 = 被拒/未进交易所。或看 `ActiveUserID`（自撤=本账户名）、`OrderSubmitStatus`（自撤='3' Accepted）。
- 客户只看得到柜台归一化后的包，无法也不需区分 CTP 包/交易所包（Q57）。
- 报单必要条件：每天首次登录成功后必须 `ReqQrySettlementInfo` 查确认状态 + `ReqSettlementInfoConfirm` 确认结算单才能交易（当天已确认的会话再次登录可直接交易）。

---

## 6. 资金、持仓、保证金、手续费（逐日盯市口径）

权威来源：《CTP账户持仓和资金的维护》2023（开心秋水）等四份，公式与算例完整照录于 notes/04。

### 6.1 资金恒等式（实现 ledger 的核心口径）

```
静态权益     = PreBalance + Deposit − Withdraw
Balance      = 静态权益 + PositionProfit + CloseProfit + CashIn − Commission   ← 即「动态权益」
Available    = Balance − CurrMargin − FrozenMargin − FrozenCommission − FrozenCash − DeliveryMargin
```

**CTP 的 `Balance` 字段 = 动态权益（含当日浮动/平仓盈亏），不是静态权益**——这是 ledger 实现最容易错的口径（CTPBuddy 现有 `to_field=B` 即 dynamic_equity，与此一致）。PositionProfit/CloseProfit/Commission/CurrMargin/FrozenMargin/FrozenCommission/CashIn 等账户字段均为**按持仓汇总的当日值，结算后清零**。持仓盈亏由行情快照驱动：多头 `(最新价−持仓均价)×乘数×手数`，空头反向。

**上述恒等式已被真实柜台导出逐项验证**（M3，`tools/audit_real_accounts.py`）：153 个交易日 × 3 个账号 = **459 个账户日 100% 通过**。

结算单（`ReqQrySettlementInfo` 正文）是**另一套口径**，别与上面混用：
```
期末结存 = 期初结存 + Σ(出入金明细 入金−出金) + 持仓盯市盈亏 + 平仓盈亏
           − 手续费 + 权利金收入 − 权利金支出
客户权益 = 期末结存；可用资金 = 客户权益 − 保证金占用；风险度 = 保证金占用 / 客户权益
```
**983 份真实盯市单中 976 份通过、7 份跳过**（跳过项含期权行权/交割，v1 不实现期权）。注意 983 = 顶层 773 + `2024/` 子目录 210，后者此前被审计脚本的 `os.listdir` 漏掉（见 DESIGN §8.7.1）。三处易错：

- **出入金必须从明细行求和，不能读结算单的汇总字段**。20250123/13200265 汇总「出入金 0.00、银期转账 0.00」，其自身明细却列着一笔 190000 银期转账出金；汇总字段不可信。
- **申报费不是独立字段，而是「出入金」类型的一笔出金**（说明栏写「中金所申报费 出金」）。20260112/13200265 缺这一项时恒等式差**恰好 1.00**，补上分毫不差。§6.4「盘中实时资金不含申报费、只体现在结算单」的正确表述是：**结算时从权益里扣除**。
- **盘中 `Available` ≠ 结算单「可用资金」**。前者含浮动盈亏（盘中口径），后者是结算后权益减保证金。20260112/13200265 前者 1343424.69、后者 1442532.83，差额恰为 `PositionProfit` 17160.00。两边各自自洽，只是时点不同。

### 6.2 持仓明细与今昨仓

- **持仓记录（汇总）的键是 7 字段**（notes/14 §B2）：`InstrumentID` + `ExchangeID` + `BrokerID` + `InvestorID` + `PosiDirection`（`'2'`多/`'3'`空，**同合约多空是不同记录**）+ `HedgeFlag` + **`PositionDate`**（`'1'`今仓/`'2'`昨仓）。**只有上期/能源的合约才可能有 `PositionDate='2'` 的记录**，因其区分今仓与昨仓记录。CTPBuddy 现有键为 (Investor, Instrument, Direction, HedgeFlag)，**缺 `PositionDate`**（列入 §10.4）。
- **`YdPosition` 是「交易日起始的昨仓静态初值」**，不随平昨而减少；**当前真实昨仓 = `Position − TodayPosition`**。两者语义不同不可混用。**对非上期/能源，昨仓与今仓在同一条记录里；上期/能源分成两条记录**（notes/14 §B3）。
- **持仓明细单条 = 每一笔开仓成交形成的一条记录**，官方键为 8 字段（含 `TradeType`），或简写为 **`OpenDate + TradeID`**（出现自成交时再加 `Direction`）——因为 **`TradeID` 每个交易日可能重复编号**。CTPBuddy 的 `PositionDetail` 采用 `(OpenDate, TradeID, Direction, ...)`，与官方简写口径一致 ✅。
- `TradeType`：`'0'` 普通成交、`'4'` **组合单生成成交**（交易组合单或**大商所盘后自动组合**的持仓明细为 `'4'`）。v1 不做组合，暂不建模。
- 持仓查询**无数据时回「`pInvestorPosition` 空指针 + `bIsLast=true`」**，而不是回一条全零记录（notes/14 §B4-5）——回全零记录会让客户端把 `Position=0` 的行当真实持仓。
- **先开先平与今/昨是两个正交的轴**（M3 修正）：消耗明细**只按开仓时间排序**；`平今`/`平昨` 决定的是「可以动哪个年龄桶」，**不是**允许跳到最新那笔。把「平今」实现成「今仓取最新」会按错口径结盈亏。真实结算单可验：IH2501 买 1 手昨仓（开 2626.2、昨结 2607.2）平于 2616.4 → (2607.2−2616.4)×300 = **−2760.00**，与结算单一致；按开仓价算会得 −2940。
- **盯市盈亏是逐笔之和，不是均价 × 手数**。真实结算单同页并列「持仓明细」逐行与「持仓汇总」总计：4 笔 IH2501 明细 2520+5700+5640+3300 = 17160.00 = 汇总行。均价法能过资金恒等式却过不了这条——这正是账本必须持明细的直接原因。
- **不要把 `YdPosition` 写成 `Position − TodayPosition`**：官方查询字段 `YdPosition` 是交易日起始的静态昨仓初值；`Position − TodayPosition` 只能表示查询时当前仍存的昨仓数量。任务41已支持用户逐笔初仓供给并保留静态值；未提供初仓且尚未进入日结时仍不编造静态值。
- 今/昨仓口径比持仓更细的是**手续费**：即使大商所也严格区分平今/平昨费率，统一用平昨费率会有较大偏差。
- 可平数量（防重复平仓）：多头 `Position − ShortFrozen − CombShortFrozen`，空头 `Position − LongFrozen − CombLongFrozen`。
- 冻结规则：买开→多头持仓 LongFrozen+=报单量；买平→空头持仓 LongFrozen+=报单量（挂单即冻，防"还有 1 手可平"误判）；期权买入开仓不占保证金改冻权利金。
- 报单错误 → 按报单量解冻；撤单 → 用 `VolumeTotal`（未成交量）解冻；多端登录须按 FrontID/SessionID 判断是否本端再解冻。

### 6.3 保证金

```
期货保证金 = (按手数保证金费 + 按金额保证金率 × 价格 × 合约乘数) × 手数
          = (MarginRatioByVolume + MarginRatioByMoney × Price × VolumeMultiple) × Volume
```

- **实际计算用公司保证金率**（`ReqQryInstrumentMarginRate` 返回，即最终费率）；`ReqQryInstrument` 返回的是交易所率，**计算不用**；`ReqQryExchangeMarginRate/Adjust` 仅中间过程。同一资金在不同公司可开仓数不同。
- **MarginPriceType**（`ReqQryBrokerTradingParams` 查）：昨仓恒用昨结算价；今仓按公司配置（昨结算'1'/最新'2'/成交均价'3'/开仓价'4'）。只有最新价/成交均价模式下今仓保证金随行情波动。市价单冻结按涨跌停价、占用按价格类型对应价。
- 冻结保证金 = 昨结算价 × 乘数 × 保证金率 × 报单量 + 按手数保证金 × 报单量（公司算法不一，有按报价/昨结/最新价的）。
- 优惠：上期所品种内大单边（`MaxMarginSideAlgorithm` 判断）、中金所跨品种大单边、大商/郑商套利合约取高；期权见 §6.5。**当前实现（2026-10-03）**：品种内大单边和查询已接线，账本/风控冻结/成交/撤单/平仓切边/mark-to-market 共用 `Ledger::product_group_margin`；开关来自用户 RefData，不能按交易所硬编码。账户、交易所、ProductID 隔离，未开启合约维持求和。当前仅投机与空投资单元，其他报单拒绝；跨品种/套利组合/仓单折抵无官方成员映射，明确不支持，IF/IM 不凭空抵消。历史“逐笔相加”的复核和源引用保留于 DESIGN §8.7.2 与 notes/04 C4。规则本身记于 notes/04 C4。
- 期权保证金归纳式（交易所口径）：`每手卖方交易保证金 = MAX(权利金 + 交易所期权合约保证金不变部分, 交易所期权合约最小保证金)`（不变部分/最小保证金由 `ReqQryOptionInstrTradeCost` 查；除上期所外最小保证金为 0）；投资者口径同理换 `FixedMargin`/`MiniMargin`。权利金昨仓 = 结算价×乘数（上期所例外：max(昨收,昨结)×乘数）；委托冻结一律用昨结算价。各交易所分公式见 notes/04 C5。

### 6.4 手续费

```
手续费 = 成交量 × (成交价 × 乘数 × RatioByMoney + RatioByVolume)
```

开仓/平仓/平今各一套 RatioByMoney/RatioByVolume；平今免手续费的算例（c2101 平昨 1.2 元/手、平今 0）。**申报费**（OrderCommRate）：中金所特有，报单+撤单都计，FAK/FOK 的自动撤单也计（一个 FAK/FOK = 2 次信息量）；盘中实时资金不含申报费、只体现在结算单。期权 = 期货口径 + 执行手续费（`ReqQryOptionInstrCommRate`）。

费率查询坑：`BrokerID/InvestorID/HedgeFlag` 必填；`InstrumentID` 留空 = 只返回**当前持仓合约**的费率（设计行为，不是全市场全集）；查 IF2009 可能返回品种 IF 的费率（品种级费率）；`ReqQryInstrumentOrderCommRate` 查不到具体申报费率（空指针）。

### 6.5 盈亏计算（逐日盯市）

- 持仓盈亏（浮动）：多头 `(最新价−持仓均价)×乘数×手数`；持仓均价 = 持仓成本 ÷（持仓数量×乘数）。
- 平仓盈亏 CloseProfit 按**明细**算，N=该明细被平数量：
  - 多头明细 `(成交价 − 持仓价格) × N × 乘数`；空头明细 `(持仓价格 − 成交价) × N × 乘数`；
  - **昨仓明细持仓价 = 昨结算价；今仓明细持仓价 = 开仓价**——顺序错了盈亏就算错。**官方背书**：`ReqQryInvestorPositionDetail` 的 `CloseProfitByDate` 字段说明写明公式为 `(closeprice − openprice **or** preSettlementPrice) × volume × RateMultiple`（notes/14 §D4），"or" 字即区分昨仓用昨结算价、今仓用开仓价。此前该结论仅来自 P1 与真实结算单，现有 SDK 字段说明作第一手依据。
- 算例（c2101，昨结 3005，乘数 10）：2 手昨仓(开 3006) + 2 手今仓(开 3000)，卖平 3 手 @3004，先开先平平 2 昨 + 1 今：昨仓 `(3004−3005)×10×2=−20`、今仓 `(3004−3000)×10×1=+40`，合计 +20。完整算例与剩余持仓字段见 notes/04 E2。
- 期权盈亏 = 期货式价差 + 权利金收支（CashIn 计入动态权益）。

### 6.6 组合合约持仓（3 条记录）

组合开仓后持仓查询生成 **3 条记录**：组合记录（只算持仓量/开仓量/保证金）+ 两条分腿记录（盈亏/成本/手续费含组合形成部分，保证金只计单一持仓部分）。方向规则：近月腿与组合同向、远月腿反向。平仓原则「先单一后组合、先开先平」；用到组合头寸时**打散**：分腿衍生出单一持仓（撮合编号、开仓时间延续原组合，因而常被优先平掉）。6 种平仓情形的字段增减见 notes/04 F2。

---

## 7. 行情订阅语义

- **必须登录后才能订阅**；`OnRspSubMarketData` 无论合约对错都返回 "CTP:No Error"——**只有编码正确的合约才有行情**（上期/能源小写+4 位 rb1909、中金大写+4 位、郑商大写+3 位 TA001、大商小写+4 位；过期合约无行情）。行情 API 无合约查询，靠交易 API `ReqQryInstrument` 或按规则生成。
- **CTP 推的是快照（切片）不是逐笔**：一段时间逐笔合成一个快照，一般 1 秒 2 笔（郑商所可能多笔）；「有更新才推」「首次连接推初始行情」；推送时间不严格 000/500。
- `Volume` 是**当日累计量**，切片成交量 = 本切片 Volume − 上切片 Volume。`UpdateMillisec`：上期/能源/中金只出现 0 和 500、大商所真实毫秒、郑商所恒 0。**无效值 = double 上限 1.7976931348623157e+308**（如盘中 SettlementPrice）。
- `AveragePrice`：除郑商所外其余大所需除以合约乘数才是真实均价（技术指南 Q45）。Turnover 同类校正（郑商×乘数、大商/上期不需——注意与 FAQ #23 的表述互参，实现时以实测为准）。
- `TradingDay`（交易日）vs `ActionDay`（实际日期）：夜盘设计上归属次一交易日（周五夜盘→周一），但各所实现混乱（上期/能源准确；大商所夜盘两日期都是 TradingDay；郑商所日夜盘均当天）——**不可假设统一**，取登录响应/`GetTradingDay()` 的 TradingDay。
- **无历史行情、无回补**：断线/未登录期间丢失不回补；盘后（15:00~15:30）交易所结算完成会推含结算价的快照、郑商所盘后继续推——别当异常。日盘启动会重演夜盘流水可能重推夜盘行情。
- 传输模式：TCP（缺省）/ UDP / 组播（快速行情），注册前置地址都写 `tcp://`（起始字符串固定）；UDP 不可靠——登录、订阅及第一次行情仍走 TCP。

### 7.1 现手、增仓、开平、对手盘（全部客户端自算，notes/12 §D）

行情结构体**没有**这些字段，客户端需用 `Volume`/`OpenInterest`/`LastPrice`/`AskPrice1`/`BidPrice1` **自行计算**：

| 量 | 公式 |
|---|---|
| 现手 | 后一笔 `Volume` − 前一笔 `Volume`（≥0） |
| 增仓 | 后一笔 `OpenInterest` − 前一笔 `OpenInterest`（**可正可负**） |
| 开平（性质，不带方向） | 双开 `现手=增仓>0`；开仓 `现手>增仓>0`；平仓 `现手>−增仓>0`；换手 `现手=增仓=0`；双平 `现手>增仓=0`；未知 `现手=增仓=0`（异常，仅记录） |
| 方向 | 向上 `后LastPrice ≥ 前AskPrice1`；向下 `后LastPrice ≤ 前BidPrice1`；向上 `后LastPrice > 后AskPrice1`；向下 `后LastPrice < 后BidPrice1`；其余不变 |

性质(6) × 方向(3) = **18 种开平组合**（如 换手+向上=多换、换手+向下=空换、双开+向上=双开、平仓+向下=多平）。

> ⚠️ 「换手」与「未知」两行字面条件相同（`现手=增仓=0`），须由前后笔方向区分，**不要机械照抄当函数真值**。

**对手单（单边口径，2020-01-01 起交易所改为单边统计）**：设 S = 对手单中开平方向相同的操作，O = 方向相反的操作：

```
S = abs(增仓) / 2
O = 现手 / 2 − S
```

> **CTPBuddy 落点**：CTPBuddy 不必实现这些派生量，但**下游客户端会在收到 CTPBuddy 行情后自己算**，其正确性依赖 CTPBuddy 提供的 tick 满足三个前提：同一合约、累计量、时间序单调不减。**场景 DSL 的 `ticks` 必须显式给出 `Volume`/`OpenInterest` 累计值**（notes 记忆里已踩过"ticks 是场景行数不是时点数"的坑）。

### 7.2 K 线必须客户端自己合成

**CTP 不推 K 线**。由 tick 的 `UpdateTime`/`LastPrice`/`Volume` 六要素算出：分钟切换以 `UpdateTime` 的**分钟数**判定；`Volume` 是累计量故**周期成交量 = 本帧 − 上一帧**（用 `max(差, 0)` 兜底防回退），新分钟时 OHLC 均置为 `LastPrice`、`volume` 归零。完整原文代码见 notes/12 §B2。

---

## 8. 结算与结算单

- 流程：`ReqQrySettlementInfoConfirm` 查当天是否已确认 → 未确认才 `ReqQrySettlementInfo`（不填日期取上一交易日）+ 确认前展示 → `ReqSettlementInfoConfirm`（只需 BrokerID+InvestorID）。
- 结算单 `Content` **分多条返回**，中文可能在两条交界处被拆半个字符——必须用大 char 数组/byte 数组拼接全部响应后统一解码（CTP 字符串一律 GBK，UTF8 终端显示即乱码）。
- 结算行为：持仓明细按结算价重算盈亏/保证金；到期合约模拟强平；昨仓合并进今仓（LocalCTP 口径）；`PreBalance=Balance`、`PreSettlementPrice=SettlementPrice`、`YdPosition=Position`，当日字段清零；tradingDay 推进。字段级重置清单见 `docs/notes/03`。
- 长假/节假日识别是常见简化点（LocalCTP 未识别；CTPBuddy 亦 TODO）。

---

## 9. LocalCTP 参考实现对账要点

完整笔记（含 文件:行号）见 `docs/notes/03-LocalCTP参考实现对账.md`。结论摘要：

- **定位**：进程内仿 CTP 交易柜台，直接替换客户端 `thosttraderapi_se.dll`，**不是 FTD 服务端**（无 TCP/FTDC 编解码/组播/心跳）；内置 9 套 CTP 头文件（6.3.15~6.7.8）由 `GenScript/ParseCTPHeaders.py` 解析自动生成 SQL Wrapper、SPI 消息结构体、不支持函数占位（默认 v6.5.1 产物）。
- **会话**：FrontID=进程启动 Unix 秒、SessionID 进程自增、OrderRef 空则取会话内 max+1（与真实 CTP 一致，可作验收基准）；柜台参数 `max_user_sessions` 按 BrokerID+UserID 限制在线会话，默认 0 兼容旧行为，超限使用官方 60「CTP:用户在线会话超出上限」，降低上限不踢已有会话；不校验密码。
- **流控为零**：查询/报单均无在途/频次限制——与真实 CTP 相反，**只能当「无流控对照端点」，不能当流控基准**。
- **撮合**：限价 vs 对手价（买≥Ask1/卖≤Bid1），组合逐腿反向累加减，整单成交无部单，FAK/MinVolume 未真正实现，仅行情快照驱动。
- **风控两段式**：预检（只算不冻）+ 成交/挂单后真冻结；条件单跳过预检、触发时以 StopPrice 转普通单重走流程（`TJBD_` 前缀）。
- **资金口径**：`Balance = PreBalance + Deposit − Withdraw + PositionProfit + CloseProfit + CashIn − Commission`（与 §6.1 一致）；默认保证金 10%、手续费 1 元/手；期权盈亏恒 0、Available 不扣 FrozenCommission（简化）。
- **错误码**：统一 ErrorID=−1 + 仿 CTP 文案（成功 0）——**不对齐 error.xml 数值码**；CTPBuddy 需决策：对齐数值码（更严格）或沿用 −1+文案。
- **GenScript 版本适配手法**值得借鉴：换 CTP 版本零手改；注意 GBK→UTF8 列长翻倍。

---

## 10. 对 CTPBuddy 的实现影响清单

### 10.1 M1 已落地（对照本库语义）

- 查询每秒闸门（Core `--qry-freq`，90 号「查询未就绪」）+ 在途闸门（Shim 返回 -2）✅ §1
- SDK `_query_stream` 透明重试（90 → 1.1s 退避重用同一 req 流，8 次）与 demo `qry_with_retry` ✅
- Balance=动态权益口径 + 查询时 mark-on-read ✅ §6.1
- e2e 真实触发两类查询流控（NEED_RETRY 重试 + 在途 -2 拒绝）✅

### 10.2 M2 落地进度（2026-10-03 更新，M2-4 已收官）

1. **报单流控** ✅（M2-4 落地，2026-10-03 修正分流）：Core `order_gate` 每 (broker, investor) 每秒预算（`--order-freq` 默认 20，墙钟 1s 窗口），**报单与撤单两条独立流分开计算**（notes/14 §A.3-02），超限 116「CTP:下单频率限制」（§1 #1、§4.5）；官方错误码全集（error.xml 299 条）对账表见 [`docs/错误码全集.md`](错误码全集.md)，核心常量已按官方逐条修正（详见 DESIGN §8.11）。
2. **订单状态机** ✅（M2-4 落地）：OrderStatus 九态复刻 + OrderSubmitStatus 七态细化——初始 'a' 推送 OSS '0'、其后 '3'；指令级 '4'/'5' 由 journal `submit_status` 承载（accepted insert '0' / rejected '4' / accepted cancel '3' / rejected cancel '5'）。IsAutoSuspend 恒 0（§5.3-5.4）。
3. **回报时序** ✅（M2-4 + #43 落地）：前态+新态两笔、Trade 后置、大商所自补全部成交特例（含「进簿必返 '3'」与 ExchangeID 回填保按所规则）；**FAK 按交易所分流**（官方《报单回调规则》场景 8/9/10）——上期所/能源中心/中金所 `CancelFirst`（`a` → `5` 撤单行**先于**成交、VolumeTraded 已有值 → 每笔成交**一行** `5` + Trade，无 `'3'` 无 `'1'`）；大商所/广期所 `TradeDriven`（`a` → `'3'` → 每笔成交**一行**合成 `'1'` + Trade → `'5'`）；郑商所 `StatusDriven`（`a` → `'3'` → 每笔成交**前态 + `'1'`** + Trade → `'5'`）。三所收尾均为交易所主动撤单 → 终态行只推一行不带前态重复（与客户端 `ReqOrderAction` 的前态+新态刻意不同）；FAK 全成无官方形状，退回 §8.9 场景 2 一般规则。回归：engine.rs 7 项单测 + `m2_ioc.py` 四所并排（§5.3-5.5、DESIGN §8.13）。
4. **双推送面** ✅（#42 落地）：报单拒绝按层分流——CTP 层（会话/字段/风控/流控 116/31/16 等）**仅** `OnRspOrderInsert(NULL, pRspInfo)`；交易所层（163/164/165）先 `OnRspOrderInsert{0}` 成功响应再 `OnErrRtnOrderInsert`；撤单拒绝**双面** `OnRspOrderAction` → `OnErrRtnOrderAction`。`ERR_RTN_*` 载荷为 composite（客户端 input struct ++ RspInfo，input 在前）。e2e `m2_surface.py` 五段锁死（A 段反向断言「CTP 层拒绝无 late 面」，B/C 段正向断言双面与载荷回显）。遗留 `91 EXCHANGE_RTNERROR` 交易所侧拒单转发未接线（§5.1-5.2、DESIGN §8.12、notes/09）。
5. **撤单语义** ✅（M2-4 落地）：ActionFlag 仅 Delete；OrderSysID 空值不撤；撤单失败 25/26 区分（终态「已全成交或已撤销」vs「找不到相应报单」）已按官方场景 6/7 实现（`terminal_refs` 终态记忆）；双回调成对见第 4 条（§4.4、§5.1）。
6. **FAK/FOK** ✅（M2-1 落地）：TC+VC 组合语义、FOK=整单成交否则全撤、FAK 最小成交量整笔撤销规则；条件单触发转新报单未做（§4.1）。
7. **交易所差异归一化** ✅（M2-1 落地）：平今转换（非上期所一律 Close）、市价单按所语义；郑商所 FAK-only 规则表待注入；TradingDay 各所混乱见 §7（§4.1-4.3、§7）。
8. **成交开平标志 ≠ 报单开平标志** ✅（M2-1 落地）：非上期所平仓回 '1'（§4.3）。
9. **结算流程**：ReqSettlementInfoConfirm 前置已校验（M1）；结算字段重置、长假识别、日结仍 TODO（§8）。柜台参数 `settlement_required` 默认开启；登录后未对当前交易日确认时，报单前置返回官方 `42 SETTLEMENT_INFO_NOT_CONFIRMED`「CTP:结算结果未确认」，确认后允许报单；关闭开关仅用于兼容旧测试行为。
10. **错误码全集对账** ✅（#42 落地）：error.xml 299 条逐条标注 → **19 已实现**（推送面全部对齐）/ **51 可落地**（语义在范围内但无代码路径发出，缺口清单见 [`docs/错误码全集.md`](错误码全集.md)）/ **229 暂不可达**（业务域未实现）。状态列由 `tools/fill_errorcode_status.py` 按实际代码面生成，改代码后重跑。
11. **LEDGER 扩展**：MarginPriceType 配置 ✅、平今/平昨费率 ✅、FrozenCommission 报单估算+释放 ✅（M3-2/M3-3 已补齐，2026-10-03 复核）；**品种内保证金优惠已实现**——用户 RefData 的 `MaxMarginSideAlgorithm` 控制，按 broker/investor/exchange/ProductID 聚合；账本与 `ReqQryInvestorProductGroupMargin` 共用唯一计算，冻结计待成交开仓后的增量、成交/撤单/平仓及 mark-to-market 后重算。跨品种映射、套利取高仍不支持；当前仅投机、空投资单元，其他报单明确拒绝（§6.3、notes/04 C4）；期权权利金（§6）。
12. **费率查询接口** ✅（M3-3 落地，2026-10-03）：`ReqQryInstrumentMarginRate` / `ReqQryInstrumentCommissionRate` / `ReqQryInstrumentOrderCommRate` / `ReqQryBrokerTradingParams` 四张由 `unsupported` 转为实装，官方语义逐字复刻——**`InstrumentID` 留空 = 返回该投资者持仓对应合约的费率（不是全市场，「目前无法通过一次查询得到所有合约保证金率」）**，`BrokerID`/`InvestorID`（及 `CurrencyID`）必填、「不填则返回值为空」。定位上四张表与账本计算**共用同一份 `RefData`**，客户端交叉核对 `ReqQryInstrumentMarginRate` 与 `ReqQryTradingAccount.CurrMargin` 时数字必然一致（§9、DESIGN §6.4/§8.6.1）。
13. **保证金/手续费公式落地** ✅（M3-2 落地，2026-10-03）：保证金 `(MarginRatioByVolume + MarginRatioByMoney × Price × VolumeMultiple) × Volume`，**用公司费率**（`ReqQryInstrumentMarginRate` 口径），`ReqQryInstrument` 的交易所费率仅展示；`MarginPriceType` 四值（'1' 昨结算/'2' 最新价/'3' 成交均价/'4' 开仓价），**昨仓恒用昨结算价**不受该设置影响、只有 '2'/'3' 下今仓保证金随行情波动；手续费 `数量 × (成交价 × 乘数 × RatioByMoney + RatioByVolume)`（两项**相加**非取 max），开仓/平昨/平今各一套，一笔 `Close` 吃掉 2 手昨仓 + 1 手今仓时**按两腿分别计价**；冻结按昨结算价（与该挂单限价无关），平仓释放按**开仓价**算（§9、DESIGN §8.6）。
14. **LocalCTP 对账基准**：OrderRef 生成规则、撮合规则、结算字段重置清单可直接抄（§9）。

### 10.3 其他 TODO

FTD 报文流控（无错误仅延迟缓存）、前置连接数流控、同用户最大在线会话数、交易所 API 流控（§1 #4-#7）；FIX 网关、银期转账（未通读资料，需要时先读 notes 未覆盖清单里的文档）。

### 10.4 入门系列带来的待修正项与新增待办（notes/11~14，2026-10-03 登记）

本批 17 份客户端实操资料（`docs/程序化交易入门/` 全套 + 4097 + 穿透式监管）通读后的净结论。完整逐条表（22 条新增待办 + 9 条已获官方背书）在 [`notes/14` §F](notes/14-入门系列-查询流控与持仓查询更新.md)，此处只登记**要动代码的**与分类索引。

**A. 口径冲突（需修或需核）**

| # | 事项 | 官方口径（出处） | CTPBuddy 现状 | 优先级 |
|---|---|---|---|---|
| 1 | **报撤单流控分开计算** | `ReqOrderInsert` 与 `ReqOrderAction` **各自**每秒最大笔数、**分开计算**（notes/13/14 §A.3） | **✅ 已修**（2026-10-03）：`order_gate` 拆为 `GateStream::Insert/Cancel` 两条独立预算流，e2e `m2_flow` Phase 1 锁定 | ~~高~~ 已清 |
| 2 | **只有 `OnRtnTrade` 动持仓/资金** | "以成交回报为准"，否则可能平仓不成功（notes/13 §E） | **✅ 已核**：`dispatch_event` 仅 `EngineEvent::Trade` 调 `Ledger::on_fill`；Order 事件只处理终态剩余冻结。资金冻结/解冻仍可由报单/撤单事件发生 | ~~高~~ 已清 |
| 3 | 撤单走 `FrontID+SessionID+OrderRef` 时**必须填 `InstrumentID`** | 必填（notes/13 §D2；SDK ReqOrderAction HTML） | **✅ 已修**（2026-10-03）：缺 `InstrumentID`，以及 sysid 路线缺 `ExchangeID` 时返回 error.xml 23，并按官方双推送面回调 | ~~中~~ 已清 |
| 4 | `TradeID` 除郑商所外**同号双向**，去重须带 `Direction` | notes/13 §E | **待核**：当前引擎撮合双方共享同一 TradeID；未找到足够可靠的官方 SDK 原文确认郑商所例外，暂不改 | 中 |
| 5 | 持仓记录键含 **`PositionDate`**；上期/能源今昨**拆两条记录** | 7 字段键（notes/14 §B2） | **✅ 已修（查询投影层）**：SHFE/INE 的今昨年龄桶拆两行；其他所保留单行；核心键暂不拆，避免扩大账本/套保维度 | ~~中~~ 已清（核心键仍是后续边界） |
| 6 | 查询无数据回**空指针 + `bIsLast=true`**，不回全零记录 | notes/14 §B4-5 | **✅ 已核**：Core 空流统一发送 `QRY_LAST`；shim dispatch 回调 `nullptr, bIsLast=true`；Python `_query_stream` 返回 `[]`，真实 shim demo 已补断言 | ~~中~~ 已清 |
| 7 | `YdPosition` 是静态昨仓初值，不随平昨减少 | notes/14 §B2 | **✅ 任务41已落地**：逐笔初仓供给初始化静态值；当前昨仓仍由 `Position−TodayPosition` 表示；无初仓时不伪造静态值；**✅ 任务42**：显式日结后 `YdPosition=当前 volume`、`TodayPosition=0`，保留明细 key | 日结已实现；SettlementInfo 查询回报仍不伪造 |
| 8 | `TradeType` `'0'`/`'4'`（组合单生成成交） | notes/14 §B6 | 未建模（v1 不做组合；大商所盘后自动组合会产生 `'4'`） | 低 |
| 9 | `OrderSysID` 第 1 次回报为空、第 2 次才有 | notes/13 §E | 需核（下游方案 01 依赖此形状） | 低 |

**本批复核暴露的最大数值偏差现已修复**：保证金优惠按用户 RefData 的 `MaxMarginSideAlgorithm` 控制，按交易所 + 品种聚合，多空取大；`ReqQryInvestorProductGroupMargin` 与账户 `CurrMargin` 共用同一聚合计算，单边仍保持求和。真实数据量尺 `tools/audit_real_accounts.py::audit_hedged_margin`（64/64）继续用于防回归。跨品种映射、套利取高、仓单折抵仍未实现，见 §10.2 第 11 条、DESIGN §8.7.2、notes/04 C4。

**B. 新增待办（分类索引，逐条见 notes/14 §F 表 2）**

- **连接/认证**（notes/11）：`ReqAuthenticate` 必须先于 `ReqUserLogin` 且发起在 `OnFrontConnected` 内；`AppID`+`AuthCode` 一一对应、`UserProductInfo` 已废弃；断连原因码 `0x1001`~`0x2003` 已收录 §1.0.1，**版本不匹配导致的 4097 循环重连无用**；前置连接数流控被踢后立刻重连会循环被踢，需退避。
- **行情字段**（notes/12）：盘中 `SettlementPrice = DBL_MAX` 为无效值口径；`AveragePrice` 郑商所不除乘数、其余四所要除；订阅任何 id 都返回 No Error（只有对编码才有行情）；夜盘 `TradingDay` 各所对照已收录 §7；`Volume` 累计语义 → **场景 tick 必须给累计值且单调不减**（§7.1，下游现手/开平/对手盘全靠它自算）。
- **报撤单**（notes/13）：`UserID` 不填会收不到 `OnErrRtnOrderAction`（不填不应报错）；`ReqBatchOrderAction` 仅大商所+做市商；撤单数风控（单合约 500 笔）**CTP 内没有**、FAK/FOK 不计入、需策略自加；`InvestUnitID` 已废弃。
- **查询/持仓**（notes/14）：同用户在线会话数按 **`UserID`** 计（文案「CTP:用户在线会话超出上限」）；盘后交易所自动组合持仓产生 `TradeType='4'` 明细 + 只收大单边保证金。

**C. 已获官方背书（无需改动，9 条）**：昨仓盈亏按昨结算/今仓按开仓价（`CloseProfitByDate` 字段说明直接背书）、买平冻结空头 `LongFrozen`、平今转换、TC/VC 组合、流控文案、成交价取对手方价、撤单 25/26 区分成因、报单被拒按量解冻、明细键 `OpenDate+TradeID`。见 notes/14 §F 表 3。

---

## 11. 术语速查

| 术语 | 含义 |
|---|---|
| FTD | 期货交易数据交换协议（CTP API 的应用层协议） |
| FAK | 立即成交剩余自动撤销（IOC+AV/MV） |
| FOK | 立即全部成交否则自动撤销（IOC+CV） |
| 在途查询 | 已发出未收完所有响应的查询；上限 1 笔，再发返回 -2 |
| QryFreq | 前置侧查询每秒频次配置；超限 OnRspError[90] |
| TERT_RESTART/RESUME/QUICK | 私有/公有流重传三模式 |
| MaxOrderRef | 登录响应给的最大报单引用；OrderRef 从此递增 |
| 动态权益 | CTP `Balance` 字段口径（含当日盈亏） |
| 逐日盯市 | 结算价重算持仓盈亏的记账制度（本文默认口径） |
| TJBD_ | 条件单 OrderSysID 前缀 |
| 大单边 | 同品种多空保证金只收大的一边 |

---

*分册索引：`docs/notes/01~05`、`09`、`10`；官方资料可读版：`docs/api-doc-html/`（API 接口说明 405 页）、`docs/错误码全集.md`（error.xml 299 条）。原始 PDF/CHM/LocalCTP 源码位置见 §0。本文件随项目演进更新；与官方文档冲突时以 SDK CHM 为准。*
