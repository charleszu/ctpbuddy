# CTP 语义知识库（项目存档）

> 本文件是 CTPBuddy 项目的 CTP 语义权威存档，供后续 Agent 与开发者查阅。
> 内容来自 `C:\workspace\src\CTP\docs`（30+ 份官方 SDK 文档/教材/坑指）、SDK 自带 CHM《6.7.13_API接口说明》与参考实现 `LocalCTP` 的系统化通读，逐条注明出处；**冲突处以官方 CHM/API 说明为准**。
> 深度细节（含原文引用、算例、代码行号）在 `docs/notes/` 下五份分册；本文件是索引与结论层。
> 官方资料可读版：SDK《6.7.13_API接口说明》CHM 已转为 Markdown（404 页）在 [`docs/api-doc-md/`](api-doc-md/)，SDK 错误码全集（error.xml 299 条）可读表在 [`docs/错误码全集.md`](错误码全集.md)。

---

## 0. 资料来源与阅读覆盖

| 资料 | 主题 | 状态 |
|---|---|---|
| SDK CHM《6.7.13_API接口说明》（ctpsdk/6.7.13_20260225/docs） | 接口逐函数说明、报单回调规则、各交易所特殊指令、条件单规则、流控 | 已通读（流控页为权威来源）；全量 Markdown 版（404 页）见 [`docs/api-doc-md/`](api-doc-md/) |
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
| CTP程序化交易入门系列之十一/十四 | 费率查询、保证金计算与优惠 | 已通读 |
| CTP行情订阅常见问题解答.pdf | 订阅语义、TradingDay、字段坑 | 已通读 |
| LocalCTP（C:\workspace\src\CTP\LocalCTP） | 进程内仿 CTP 交易柜台参考实现 | 已通读（结构/流程/对账点） |
| 未读（低优先级，FIX/银期/培训类）：CTP FIX接口使用规范说明.docx、CTP FIX网关说明.docx、CTP新版银期转帐TradeApi使用说明.pdf、综合交易平台交易银期功能特别说明.doc、api培训-二期.pptx、20150422_上期技术_API培训-改.pptx、CTP API终端开发交流讨论会 pdf、CTP国际版开发指南.pdf、CTP_API开发文档资料/、综合交易平台简介.pdf | FIX 网关、银期、培训 | **未通读**，需要时再补（见 §12 待办） |

---

## 1. 流控全景（六类）与 CTPBuddy 落点

权威来源：SDK CHM《报单流控、查询流控和会话数控制》。CTP 交易系统有多层流控，分布在客户端 API、交易前置、柜台/核心、交易所多处：

| # | 流控 | 配置位置 | 超限症状 | CTPBuddy 落点 |
|---|---|---|---|---|
| 1 | 报单/撤单每秒笔数 | **CTP 柜台端**【程序化交易频繁报撤单管理】 | `OnRspOrderAction`「CTP:下单频率限制」 | **Core** ✅ M2-4 落地（`order_gate`，`--order-freq`，报撤共享每 (broker,investor) 每秒预算，116 号） |
| 2 | 查询每秒笔数 QryFreq | **交易前置** front_se（穿透式监管版本起，API 连接前置时读取该配置；历史上内置在 API 里 1 笔/秒） | `OnRspError`[90]「CTP：查询未就绪，请稍后重试」，查询不执行 | **Core** `--qry-freq`（默认 2）✅已实现 |
| 3 | 查询在途笔数 = 1 笔 | **客户端 API 内置**（永远存在，与前置配置无关） | 查询函数**返回值 -2**「未处理请求超过许可数」，请求不上线 | **Shim** 在途闸门 ✅已实现 |
| 4 | FTD 报文流控 FTDMaxCommFlux | 交易前置 | 无错误，超限报文在前置缓存延迟到下一秒 | Core TODO |
| 5 | 前置连接数 ConnectFreq | 交易前置（同 IP 每秒连接数；白名单 whiteiplist） | 主动断开 → `OnFrontDisconnected` | Core TODO |
| 6 | 同一用户最大在线会话数 | 柜台/交易核心（6.6.3+ 按用户维度） | `OnRspUserLogin`「CTP:用户在线会话超出上限」 | Core TODO |
| 7 | 交易所 API 流控 | 交易所端 | `OnRtnOrder` 报「CTP：交易所每秒发送请求数超过许可数」/「未处理请求超过许可数」 | Core TODO |

关键语义：
- **查询流控是两侧的**：在途 1 笔在客户端 API（Shim，返回 -2，这正是「1s 只能有一个在途查询」现象的出处）；每秒 QryFreq 在前置侧（Core，90 号错误）。穿透式监管版本后 API 只在连接时读取前置配置，执行仍在前置。**只做一半会让下游客户端行为与真实 CTP 漂移**（掩盖不会处理 NEED_RETRY 的客户端缺陷）。
- **投资者受限、操作员不受限**（查询流控）。
- **所有 `ReqQuery*` 开头的函数不受查询流控限制**——走交易核心不经查询核心。
- 报单流控归柜台（用户判断正确）：实现在 Core 侧 M2 风控规则表；客户端侧表现只有「排队不报错」（见 §3.5）。

历史口径（2009 FAQ 时代）：查询每秒 1 次 + 在途 1 个，交易指令默认每会话 6 笔/秒、同账户最多 6 会话，超限**排队不报错**（【技术指南】Q19、客户端指南 4.14）。注意这是「无错误」与「90/-2」两种形态的区别：查询超限有明确拒绝，交易指令超限只排队。

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

登录链：`OnFrontConnected → ReqAuthenticate → ReqUserLogin → 每交易日首次交易前 ReqSettlementInfoConfirm`。连接建立只代表链路可达，**无身份验证**；后台开启强制认证时必须先过认证。

释放顺序（【技术指南】Q22，防死锁）：`RegisterSpi(NULL) → Release() → 置空指针 → delete SPI`。**禁止在 SPI 回调线程内 Release**。创建与释放最好在同一线程。

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

### 4.5 报单流控

柜台端【程序化交易频繁报撤单管理】配置每秒最大报撤笔数，超限 `OnRspOrderAction`「CTP:下单频率限制」（官方错误码 **116 ORDER_FREQ_LIMIT**，勿与 91 EXCHANGE_RTNERROR「CTP：交易所返回的错误」混用）。注意与 2009 FAQ「默认 6 笔/秒、超限排队不报错」的历史口径区分——现代柜台是显式拒绝。CTPBuddy 落点：Core M2 风控规则表 ✅（M2-4 落地，报撤共享预算，口径见 DESIGN §8.11）。

---

## 5. 回报时序与状态机（M2 撮合/回报的核心规范）

### 5.1 回调分流表

| 回调 | 触发场景 |
|---|---|
| `OnRspOrderInsert` | **CTP 层拒绝**报单（参数校验/风控失败），ErrorID+ErrorMsg |
| `OnErrRtnOrderInsert` | 报单被 CTP 或交易所拒绝后的**错单回报**（与 OnRspOrderInsert 是两回事，都要挂；不填 UserID 时收不到它） |
| `OnRtnOrder` | 报单状态回报（每笔交易所状态迁移推「前态+新态」两条） |
| `OnRtnTrade` | 成交回报（每笔成交一次；无 FrontID/SessionID，用 OrderSysID 反查订单；`Volume` 只是本笔，总量看 `OnRtnOrder.VolumeTraded`） |
| `OnRspOrderAction` | 撤单被 CTP 层拒绝 |
| `OnErrRtnOrderAction` | 撤单被交易所拒绝（与 OnRspOrderAction **成对出现**，先响应后回报） |

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

### 6.2 持仓明细与今昨仓

- 持仓明细单条 = (OpenDate, TradeID, Direction, OpenPrice, Volume, Margin, ...)，按 (OpenDate, TradeID) 排序实现**先开先平**。
- `YdPosition = Position − TodayPosition`（昨持仓不随当日平仓减少的字段口径）。
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
- 优惠：上期所品种内大单边（`MaxMarginSideAlgorithm` 判断）、中金所跨品种大单边、大商/郑商套利合约取高；期权见 §6.5。
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
  - **昨仓明细持仓价 = 昨结算价；今仓明细持仓价 = 开仓价**——顺序错了盈亏就算错。
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
- **会话**：FrontID=进程启动 Unix 秒、SessionID 进程自增、OrderRef 空则取会话内 max+1（与真实 CTP 一致，可作验收基准）；**无会话数限制、无踢线**；不校验密码与结算单确认；同账户多实例互相覆盖（已知缺陷）。
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

1. **报单流控** ✅（M2-4 落地）：Core `order_gate` 每 (broker, investor) 每秒报撤共享预算（`--order-freq` 默认 20，墙钟 1s 窗口），超限 116「CTP:下单频率限制」（§1 #1、§4.5）；官方错误码全集（error.xml 299 条）对账表见 [`docs/错误码全集.md`](错误码全集.md)，核心常量已按官方逐条修正（详见 DESIGN §8.11）。
2. **订单状态机** ✅（M2-4 落地）：OrderStatus 九态复刻 + OrderSubmitStatus 七态细化——初始 'a' 推送 OSS '0'、其后 '3'；指令级 '4'/'5' 由 journal `submit_status` 承载（accepted insert '0' / rejected '4' / accepted cancel '3' / rejected cancel '5'）。IsAutoSuspend 恒 0（§5.3-5.4）。
3. **回报时序**（部分）：前态+新态两笔、Trade 后置、大商所自补全部成交特例（含「进簿必返 '3'」与 ExchangeID 回填保按所规则）✅ M2-4 落地；**FAK 按交易所分流未落地**（官方场景 8/9/10：上期所/能源/中金 FAK 部成部撤=['a','5','5']+Trade 无 '1' 行；大商所/郑商所/广期所 FAK=['a','3','3','1']——现引擎全所统一「每笔成交前态+新态」与官方不符，m2_book C3 断言需按所重核，列为独立 TODO）。双推送面（insert 的「错单响应」半面、cancel 的 `OnErrRtnOrderAction`）待 error.xml 全集对账任务补全（§5.1-5.2）。
4. **撤单语义** ✅（M2-4 落地）：ActionFlag 仅 Delete；OrderSysID 空值不撤；撤单失败 25/26 区分（终态「已全成交或已撤销」vs「找不到相应报单」）已按官方场景 6/7 实现（`terminal_refs` 终态记忆）；双回调成对部分（`OnErrRtnOrderAction` 半面待补，同 3）（§4.4、§5.1）。
5. **FAK/FOK** ✅（M2-1 落地）：TC+VC 组合语义、FOK=整单成交否则全撤、FAK 最小成交量整笔撤销规则；条件单触发转新报单未做（§4.1）。
6. **交易所差异归一化** ✅（M2-1 落地）：平今转换（非上期所一律 Close）、市价单按所语义；郑商所 FAK-only 规则表待注入；TradingDay 各所混乱见 §7（§4.1-4.3、§7）。
7. **成交开平标志 ≠ 报单开平标志** ✅（M2-1 落地）：非上期所平仓回 '1'（§4.3）。
8. **结算流程**：ReqSettlementInfoConfirm 前置已校验（M1）；结算字段重置、长假识别 TODO（§8）。
9. **LEDGER 扩展**：MarginPriceType 配置、平今/平昨费率、FrozenCommission TODO；期权权利金（§6）。
10. LocalCTP 对账基准：OrderRef 生成规则、撮合规则、结算字段重置清单可直接抄（§9）。

### 10.3 其他 TODO

FTD 报文流控（无错误仅延迟缓存）、前置连接数流控、同用户最大在线会话数、交易所 API 流控（§1 #4-#7）；FIX 网关、银期转账（未通读资料，需要时先读 notes 未覆盖清单里的文档）。

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

*分册索引：`docs/notes/01~05`；官方资料可读版：`docs/api-doc-md/`（API 接口说明 404 页）、`docs/错误码全集.md`（error.xml 299 条）。原始 PDF/CHM/LocalCTP 源码位置见 §0。本文件随项目演进更新；与官方文档冲突时以 SDK CHM 为准。*
