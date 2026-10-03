# E-API 开发规范、客户端生命周期、行情订阅 语义笔记

> 来源文档：`综合交易平台API技术开发指南.pdf`（下称【技术指南】）、`CTP客户端开发指南.pdf`（下称【客户端指南】）、`CTP开发——基础.pdf`（下称【基础】）、`CTP行情订阅常见问题解答.pdf`（下称【行情FAQ】）、`定单状态.png`（【定单状态】图）、`行情模式.png`（【行情模式】图）。
> 用途：为实现「CTP 兼容柜台」提供行为规范依据。

---

## 1. API 初始化 / 连接 / 登录 / 登出 / 释放的生命周期

### 1.1 规范初始化流程（交易接口）

【客户端指南】2.2.4 与【技术指南】Q12 合并后的标准步骤：

1. 创建 SPI 实现类实例（继承 `CThostFtdcTraderSpi`）与 API 实例（`CThostFtdcTraderApi::CreateFtdcTraderApi(流文件目录)`）。
2. `RegisterSpi(spi)` 向 API 注册 SPI。
3. `RegisterFront(交易前置地址)` 或 `RegisterNameServer(名字服务器地址)`；交易接口不能指定传输协议（仅一个参数），行情接口第二参数可选 UDP。
4. `SubscribePublicTopic(...)` 订阅公有流（仅交易接口）。
5. `SubscribePrivateTopic(...)` 订阅私有流（仅交易接口）。
6. `Init()`：启动工作线程，自动用注册地址向服务端请求建立**无身份验证的连接**。
7. `Join()`：等待线程退出。

【技术指南】Q12 补充的登录前置链：连接成功回调 `OnFrontConnected` → `ReqAuthenticate` 客户端认证（`OnRspAuthenticate` 返回结果；真实柜台若启用强制认证，必须通过认证才能接入）→ `ReqUserLogin`（`OnRspUserLogin` 返回结果）→ 登录成功后当前交易日必须成功执行一次 `ReqSettlementInfoConfirm`（投资者结算结果确认）后才能开始交易。CTPBuddy 当前没有授权配置，不宣称实现柜台侧强制策略。

### 1.2 关键连接语义

- **无身份验证连接**：连接建立（`OnFrontConnected`）只代表链路可达；真实柜台若启用强制认证，登录前需先完成认证；CTPBuddy 当前没有授权配置，不宣称实现该强制策略。
- **登录响应内容**（`OnRspUserLogin` 的 `pRspUserLogin`）：`FrontID`（前置编号）、`SessionID`（会话编号）、`MaxOrderRef`（最大报单引用）、各交易所时间（SHFE/CZCE/DCE/FFEX Time）等。【基础】登录页同样列出这些字段。
- **登出语义**（【客户端指南】3.4/4.2 注意）：`ReqUserLogout` 会先断现有连接，重新登录后系统建立新连接，`SessionID` 重置，因此 `MaxOrderRef` 一般重新从 0 计数（【技术指南】Q51：FrontID+SessionID 变更后 MaxOrderRef 重置）。
- **释放顺序**（【技术指南】Q22，避免 Release 死锁）：先 `RegisterSpi(NULL)` → `Release()` → 置空 API 指针 → 最后 `delete` SPI 实例。

### 1.3 生命周期状态机（文字版）

```
[创建API+SPI实例] → [RegisterSpi] → [RegisterFront/NameServer] → [订阅公/私有流]
    → Init → (TCP连接) → OnFrontConnected / OnFrontDisconnected(nReason)
    → ReqAuthenticate → OnRspAuthenticate（若启用强制认证）
    → ReqUserLogin → OnRspUserLogin（拿到 FrontID/SessionID/MaxOrderRef）
    → 每交易日首次交易前: ReqSettlementInfoConfirm → OnRspSettlementInfoConfirm
    → [报单/撤单/查询……] ⇄ 断线时自动重连（OnFrontDisconnected → 重连 → OnFrontConnected，重走认证/登录）
    → ReqUserLogout → OnRspUserLogout → Release → delete SPI
```

### 1.4 API/SPI 线程模型

- 【技术指南】Q21：**api、spi 是不同的线程；api 可以同时被多个线程调用（线程安全），该特性与平台无关**。
- 【客户端指南】3.2：每个接口都有独立工作线程，可视化开发需注意线程冲突。
- 【技术指南】Q49 与【客户端指南】3.1：`OnHeartBeatWarning` 永远不会发生（心跳仅在接口内部实现，已对 API 用户屏蔽）。

---

## 2. 定单状态机（OrderStatus / OrderSubmitStatus）

依据【定单状态】图（上期技术官方 PPT 截图）及【客户端指南】4.7.4、【技术指南】Q50/Q59。

### 2.1 OrderStatus 报单状态（针对报单的）

| 值 | 说明 | 备注（原文） |
|----|------|------|
| '0' | 全部成交 | |
| '1' | 部分成交还在队列中 | 收到部分成交回报，报单还在交易所队列中 |
| '2' | 部分成交不在队列中 | 暂不使用 |
| '3' | 未成交还在队列中 | 收到交易所报单确认，报单在交易所队列中 |
| '4' | 未成交不在队列中 | 暂不使用 |
| '5' | 撤单 | 撤单成功；废单 |
| 'a' | 未知 | CTP 接收报单，并发给交易所，但还没有收到交易所确认信息（包括废单信息） |
| 'b' | 尚未触发 | 条件单保留，暂不使用 |
| 'c' | 已触发 | 条件单保留，暂不使用 |

### 2.2 OrderSubmitStatus 报单提交状态（针对某个指令：报单申报、撤单的）

| 值 | 说明 | 备注（原文） |
|----|------|------|
| '0' | 报单已经提交 | 报单提交给 CTP 报盘 |
| '1' | 撤单已经提交 | 撤单提交给 CTP 报盘 |
| '2' | 修改已经提交 | 暂不使用 |
| '3' | 已经接受 | 交易所接收报单/撤单 |
| '4' | 报单已经被拒绝 | 报单被 CTP 报盘拒绝或者被交易所拒绝 |
| '5' | 撤单已经被拒绝 | 撤单被 CTP 报盘拒绝或者被交易所拒绝 |
| '6' | 改单已经被拒绝 | 暂不使用 |

图注原文：「报单状态：是针对报单的」「报单提交状态：是针对某个指令（报单申报、撤单的）」。

### 2.3 终态与自动挂起

- **终态**（【技术指南】Q59）：`THOST_FTDC_OST_AllTraded`、`THOST_FTDC_OST_Canceled`、`THOST_FTDC_OST_NoTradeNotQueueing`、`THOST_FTDC_OST_PartTradedNotQueueing`——终态不会再改变。
- **自动挂起**（【技术指南】Q50）：CTP 有自动挂起标志（`IsAutoSuspend`），若设置该标志，断线客户的未成交报单将被自动挂起，此时状态即「未成交不在队列中」。原设计挂起报单可撤单、也可通过「激活」指令重新入队；**目前官方要求客户端把自动挂起标志设为 0，永远不挂起**。
- 【基础】报单指令页仍把 IsAutoSuspend 示例为 Yes（与官方 FAQ 要求相悖，实现兼容柜台时以「永不挂起」为准）。

### 2.4 报单/撤单的完整交互时序

报单（【基础】报单流程图、【客户端指南】4.5-4.6、【技术指南】Q55/Q56/Q57）：

```
ReqOrderInsert
  ├─ CTP（交易核心）验证失败 ──► OnRspOrderInsert（含错误码）
  │      （合法时报文不回传给客户端，除交易所拒绝场景）
  ├─ CTP 接受 ──► OnRtnOrder「已提交」(OrderSubmitStatus='0')
  ├─ 提交交易所:
  │    ├─ 交易所拒绝 ──► OnRtnOrder 返回废单，OrderStatusMsg 含报错信息
  │    └─ 交易所接受 ──► OnRtnOrder「未成交/在队列」(OrderStatus='3')
  └─ 撮合成交 ──► OnRtnTrade（每笔成交一次）+ OnRtnOrder「部分/全部成交」
```

- 【技术指南】Q55：超出涨跌停板的判断在交易所处理——CTP 收到报单先新增记录，收到交易所拒绝后修改记录并触发 `OnRtnOrder`；`OnErrRtnOrderInsert` 仅作为交易员侧错误回报，**投资者可以不关心 OnErrRtnOrderInsert**。
- 【技术指南】Q56：每个操作会收到 2 条状态回报——CTP 应答一下，交易所再应答一下，均通过 `OnRtnOrder` 推回。
- 【技术指南】Q57：**对客户端来说，所有返回包都由 CTP 发出**，无法也不需区分 CTP 包/交易所包。
- 【客户端指南】4.6：成交判断应以 `OnRtnTrade` 为准——CTP 交易核心收到成交回报后才更新报单状态；若以 OnRtnOrder 判成交并立即平仓，极小概率平仓指令到达时报单状态未更新导致平仓失败。
- 撤单（【基础】撤单流程图、【客户端指南】4.9）：

```
ReqOrderAction（ActionFlag 只能是 THOST_FTDC_AF_Delete）
  ├─ CTP 验证失败 ──► OnRspOrderAction（含错误信息）
  ├─ CTP 确认合法 ──► 提交交易所，OnRtnOrder 返回新状态
  │    ├─ 交易所拒绝 ──► OnErrRtnOrderAction（转发交易所错误）
  │    └─ 交易所成功 ──► OnRtnOrder「已撤单」(OrderStatus='5')
```

- 【技术指南】Q60：ReqOrderAction 的挂起/激活/修改功能目前未实现，**只支持撤单**。
- 「在队列中」含义（【技术指南】Q44）：报单已在交易所撮合队列中。

---

## 3. 报单/撤单接口字段规范与返回码

### 3.1 报单必填字段（ReqOrderInsert / CThostFtdcInputOrderField）

【技术指南】Q28 列出必须输入字段：`BrokerID、InvestorID、InstrumentID、ExchangeID、OrderPriceType、Direction、VolumeTotalOriginal、TimeCondition、VolumeCondition、ContingentCondition、ForceCloseReason`。
示例赋值的公共字段（【客户端指南】4.7）：`CombOffsetFlag[0]`（开平）、`CombHedgeFlag[0]`（投保）、`MinVolume=1`、`ForceCloseReason=FCC_NotForceClose`、`IsAutoSuspend=0`、`UserForceClose=0`。

### 3.2 报单类型字段组合（【客户端指南】4.7.1/4.7.3、【基础】报单指令页）

| 指令 | OrderPriceType | TimeCondition | VolumeCondition | 附加 |
|------|------|------|------|------|
| 限价单 | LimitPrice | GFD | AV | 指定 LimitPrice |
| 市价单 | AnyPrice | IOC | AV | LimitPrice=0 |
| FAK | LimitPrice | IOC | AV（任意量）/ MV（最小量，配 MinVolume） | |
| FOK | LimitPrice | IOC | CV（全部量） | |
| 条件单 | LimitPrice/AnyPrice | GFD | AV | ContingentCondition 触发条件 + StopPrice |
| 中金所五档市价 | FiveLevelPrice | IOC | AV | |
| 中金所任意价转限价 | AnyPrice | GFD | AV | |

- FAK/FOK 定义（原文）：FOK 全部可成才成交否则全撤；FAK 能成交多少成交多少、剩余立即撤销。
- 条件单触发条件枚举（LastPrice/AskPrice/BidPrice 与 StopPrice 的 >、>=、<、<= 组合，共 12 个值 '5'-~'H'）。
- 条件单 `OrderSysID` 以 "TJBD_" 开头，由 CTP 自定义、当日唯一；**条件单为 CTP 后台系统指令，并非交易所官方支持指令**（大商所止损止盈单除外，交易所支持）。
- 【技术指南】Q38/Q65：国内期货交易所不支持过夜挂单（GTC 不支持）；CTP 提供非交易时段预埋单；条件单 v4.1 后落地。

### 3.3 平仓与开平标志

- `CombOffsetFlag` 是长度 5 的字符数组（【技术指南】Q63）：单腿合约只填 [0]；组合合约各分腿从 [0] 开始每元素对应一腿（市场最长组合 3 腿，预留 5 位）。
- 枚举：Open '0' / Close '1' / ForceClose '2' / CloseToday '3' / CloseYesterday '4' / ForceOff '5' / LocalForceClose '6'。
- 【客户端指南】4.7.1：上期所用 Close 等同 CloseYesterday；其他交易所 CloseToday/CloseYesterday 均等同 Close（【技术指南】Q61：后台对 DCE 和 CZCE 统一转换为平仓）。
- 平仓顺序（业务规则）：四所统一先开先平；郑商所先平单腿再平组合；除上期所外三家涉平今手续费减免时先平今后平昨（后开先平）。

### 3.4 报单序列号（三组）与 OrderRef 规则

| 序列号 | 生成方 | 用途 |
|------|------|------|
| FrontID + SessionID + OrderRef | 客户端自维护 | 唯一标识报单，可用于撤单 |
| ExchangeID + TraderID + OrderLocalID | CTP 交易核心 | 报盘时生成，可用于撤单 |
| ExchangeID + OrderSysID | 交易所 | 接收报单后生成，可用于撤单 |

- 【技术指南】Q6：OrderRef 必须是阿拉伯数字字符；同一会话内后发送的报（撤）单 OrderRef（OrderActionRef）必须大于之前的最大值，**多线程开发需特别注意**。
- 【技术指南】Q52：报单引用由客户端自主管理，后台仅要求递增。
- 预埋单/条件单触发时发送新报单指令，需设置新的 OrderRef 和 OrderSysID（【基础】交易序号页）。

### 3.5 接口返回码与流量控制

请求函数返回值（【客户端指南】3.3、【基础】查询示例页）：
- `0`：发送成功（仅代表发出，不代表被处理）
- `-1`：因网络原因发送失败
- `-2`：未处理请求队列总数量超限
- `-3`：每秒发送请求数量超限

流量限制（【客户端指南】4.14、【基础】要点页、【技术指南】Q19）：
- 查询：每秒最多 1 次；在途查询最多 1 个（有在途查询不允许发新查询）。**只限 ReqQryXXX，对报单撤单等无影响**。
- 报单/交易指令：期货公司可配置；默认每连接会话每秒最多 6 笔交易相关指令；同一账户同时最多 6 个会话。超限**不返回错误，报单排队等待**。
- 【技术指南】Q19：CTP 仅对查询限流，对交易指令本身不限制（期货公司侧可配）。

---

## 4. 行情订阅语义

### 4.1 订阅接口（【客户端指南】3.4、【基础】行情页）

- `SubscribeMarketData(合约数组, 数组长度)`：**登录成功后才可订阅**；可重复订阅。
- 响应 `OnRspSubMarketData`：订阅不合法时返回错误；**合法时也会被调用，返回 "CTP:No Error"**。
- 陷阱【行情FAQ】Q4：CTP 无论订阅什么合约 ID 都返回 No Error，**只有合约编码正确才有行情**；编码规则：上期/能源所小写+4 位数字（rb1909）、中金所大写+4 位、郑商所大写+3 位（TA001）、大商所小写+4 位。过期合约（如当前 8 月订阅 rb1905）无行情。
- 退订 `UnSubscribeMarketData` 对应 `OnRspUnSubMarketData`；登出 `ReqUserLogout` 语义同交易接口（先断连、重连后 SessionID 重置）。
- 合约清单获取（【行情FAQ】Q2）：行情 API 无查询合约功能，需通过交易 API 登录后 `ReqQryInstrument` 查询、或爬交易所网站、或按规则生成编码。

### 4.2 行情推送规则（快照 vs 增量）（【客户端指南】3.6、【行情FAQ】）

- **CTP 提供实时快照（切片）行情，不是逐笔 tick**：切片数据把一段时间内的逐笔数据合成一个快照发出，一般 1 秒 2 笔（郑商所可能 1 秒多笔）。
- 推送规则三条（原文）：「1 秒 2 次快照行情」「有更新才推送」「第一次连接后推送初始行情」。
- 推送时间（毫秒级）不严格 000/500，可能是 300/800——因交易所推送时间不固定，且 CTP 只在有更新时才推。
- 开盘前集合竞价撮合阶段交易所也推行情；开盘时刻行情时间戳可能 >=500 也可能 <500。开盘后 CTP 会推送合约状态（如连续交易），该消息时间与开盘后第一笔行情几乎重合。
- 非交易时段行情（【行情FAQ】Q11）：日盘启动时会重演夜盘流水，可能重推夜盘行情；日盘结束后（一般 3 点~3 点半）交易所结算完成也会发行情（含当日结算价）。建议按交易时间过滤。
- **无历史行情、无回补**（【技术指南】Q8、【行情FAQ】Q9）：断线/未登录期间丢失的行情不回补；需自行补数据（天勤等免费源或闪策等收费源）。

### 4.3 TradingDay / ActionDay 语义（【行情FAQ】Q6）

- `TradingDay` 表示**交易日**，`ActionDay` 表示**当前实际日期**。日夜盘分离时二者不同：2019-08-30（周五）21:00 夜盘属于下一交易日，TradingDay=20190902（周一），ActionDay=20190830。
- 各交易所实际情况（20190830 夜盘举例）：上期/能源 TradingDay=次日、ActionDay=当天；大商所夜盘两日期都是 TradingDay；郑商所日夜盘都是当天日期；中金所无夜盘。**设计初衷是「夜盘归属次一交易日」，但各交易所实现混乱，客户端不可假设统一规则**。

### 4.4 关键字段坑（【行情FAQ】Q13-Q16、【技术指南】Q45）

- `Volume` 是交易日内**累计成交量**；切片内成交量 = 本切片 Volume − 上切片 Volume。
- `UpdateMillisec`：上期/能源/中金只出现 0 和 500；大商所是切片真实毫秒；郑商所恒为 0。
- **无效值**：`1.7976931348623157e+308`（double 上限）表示字段无效，如盘中 `SettlementPrice`。
- `AveragePrice`：除郑商所外，其余四大交易所需除以合约乘数才是真实均价（【技术指南】Q45：郑商所正确；大商所 ÷乘数；上期所 ÷乘数）。
- K 线、现手、增仓、主力合约：CTP 均不提供，需自行合成/筛选。

### 4.5 行情传输模式（【行情模式】图、【技术指南】Q18、【客户端指南】3.1/3.2）

| 行情类型 | bIsUsingUdp | bIsMulticast | 说明 |
|------|------|------|------|
| TCP | false | false | 缺省模式，普通行情前置均为 TCP |
| UDP | true | false | 需向期货公司申请，仅限专线/内网 |
| 快速行情（组播） | true | true | 仅限内网 |

- 注册行情前置无论 TCP/UDP 都写 `tcp://IP:端口`（tcp 字段是起始字符串不可更改）。
- **UDP 不可靠：登录、订阅及接收第一次行情仍走 TCP**，之后行情走 UDP/UDP 组播；UDP 使用相同地址端口，无需额外配置节点。

---

## 5. 实现「CTP 兼容柜台」的行为规范

### 5.1 通讯协议与三种通讯模式（【技术指南】Q10、【客户端指南】1.2）

- CTP-API 基于 TCP 之上的 **FTD 协议**（期货交易数据交换协议）。
- 三种通讯模式：
  1. **对话通讯模式**：客户端主动发起、服务端响应（报单、撤单、查询）；对应 DialogRsp / QueryRsp 数据流，**后台不维护其状态，通讯故障时重置、途中数据可能丢失**。
  2. **私有通讯模式**：后台主动向特定客户端发送（报单回报、成交回报）；对应 Private 私有流——可靠数据流，后台维护每个登录用户的私有流，**交易日内断线恢复后用 restart 重传全部、resume 续传断线期间数据**。
  3. **广播通讯模式**：后台向所有客户端发相同信息（合约交易状态通知）；对应 Public 公共流——同为可靠数据流。
- 一个网络连接可传多种模式报文，一种模式报文也可在多个连接中传送。
- 数据交换模式分请求/应答（Request/Response）与发布/订阅（Pub/Sub）两种（【客户端指南】1.3）。

### 5.2 前置 / 服务端职责划分（【客户端指南】1.4 系统架构）

- **交易前置**：通过 TCP 连交易终端，通过 FIB 总线连后台；只做与业务无关的通讯工作——**链路管理、协议转换、数据路由**，分散交易系统压力。
- **行情前置**：从报盘管理通过 FIB 订阅行情，通过 TCP 转发给订阅了该合约行情的终端。
- **交易核心**：基于持仓/报单/成交/出入金实时计算资金和仓位（**事前风控**）；校验所有报单；驱动交易所报盘接口；发布实时交易结果到 FIB。
- **排队服务**：将交易请求序列化，作为交易核心数据来源（主/从排队机确认+流水落盘，保证精确重演）。
- **报盘管理/报盘**：管理交易和行情报盘，实现交易所 API 接口，发送报单、接收报单回报/成交回报/行情。
- **FIB 信息总线**：通讯底层构件（数据包封装、请求/应答、发布/订阅）。
- 兼容柜台实现时：报单校验与风控在核心、与交易所交互经报盘、客户端只面对前置——**客户端的所有返回包都由柜台发出（Q57），柜台需要自行消化并归一化交易所差异**（如 DCE/CZCE 平今平昨转换、各交易所行情字段差异、组合合约编号规则等）。

### 5.3 流文件 .con 机制（【客户端指南】4.13、【技术指南】Q11）

- 接口初始化时在本地生成流文件，记录当日收到的 DialogRsp/QueryRsp/Private/Public/TradingDay 报文数量，用于 Resume 续传（交易接口 5 个文件，行情接口 3 个：DialogRsp/QueryRsp/TradingDay.con）。
- 路径由 `CreateFtdcTraderApi("flow/")` 指定；**客户端无法决定是否生成**，多实例需注意操作系统文件句柄限制。
- **多账号/多实例绝不可共用流文件目录**（数据紊乱、一个账号能收到回报其他收不到）；同目录放 2 个 dll 会互相覆盖流文件（Q20）。

### 5.4 断线重连与心跳（【客户端指南】4.15）

- `OnFrontDisconnected(nReason)` 原因码：4097(0x1001) 网络读失败、4098(0x1002) 网络写失败、8193(0x2001) 读心跳超时、8194(0x2002) 发送心跳超时、8195(0x2003) 收到不能识别的错误消息。
- 服务端主动断开的两种可能：客户端长时间无报文超时；连接数超限。
- **交易接口断线后自动重连**；重连后 SessionID 重置、MaxOrderRef 重新计数。
- 私有流/公有流重传模式三选一：TERT_RESTART（从本交易日开始重传）/ TERT_RESUME（从上次收到的续传，默认）/ TERT_QUICK（只传登录后的内容）。
- 心跳在接口内部实现，`OnHeartBeatWarning` 不会上报客户端。

### 5.5 其他实现要点

- **FENS 名字服务器**（【技术指南】Q16）：公司可部署名字服务器，客户端 `RegisterNameServer` 自动获得分配的前置地址，免配置多地址；支持多活交易中心「一键切换」。
- **代理/SSL**（【技术指南】Q13/Q17）：支持 socks4/4a/5 代理（`socks5://user:pass@127.0.0.1:10001` 格式）与 SSL 前置（`ssl://IP:端口`）。
- **客户端认证**（【技术指南】Q15）：保证投资者只能使用期货公司认可的客户端产品接入；需向期货公司提交 UserProductInfo 获得 AuthCode，`ReqAuthenticate` 时填写产品信息+认证码。
- **席位与会话**（【基础】要点页）：主席/二席/三席席位制；多终端登录限 6 个连接；查询 1 次/秒、交易 6 笔/秒。
- **账号语义**（【技术指南】Q4/Q5）：BrokerID 从业务层隔离不同经纪公司；UserID 是操作员代码（交易员代客下单场景），InvestorID 是投资者代码，投资者自己下单时两者相同。
- **结算单确认**（【技术指南】Q37、【客户端指南】4.3）：登录后先查结算单（`ReqQrySettlementInfo`，不填日期取上一交易日）再确认（`ReqSettlementInfoConfirm`，只需 BrokerID+InvestorID）；建议确认前先 `ReqQrySettlementInfoConfirm` 查当天是否已确认，避免重复确认。
- **查询分页语义**（【客户端指南】2.2.3）：`nRequestID` 关联请求与响应（频繁操作时同一响应函数可能被多次调用）；`IsLast=true` 表示最后一个数据包；`RspInfo.ErrorID==0` 表示请求被交易核心认可通过，错误码全集在 error.xml。
- **系统性能基线**（【基础】）：交易处理 2000 笔/秒（软件 8000）、正常响应 <200ms、每前置支持 20000 客户/秒同时在线、交易核心单点故障零切换时间、精确重演架构。
