# LocalCTP 实现行为对账笔记

> 考证对象：`C:\workspace\src\CTP\LocalCTP`（git 仓库；默认产物为 CTP v6.5.1 头文件编译的 `thosttraderapi_se.dll/.so`）
> 用途：为 CTPBuddy 提供「本地仿 CTP 实现」的逐项行为基准。文中 `文件:行号` 均相对仓库根目录。

---

## 一、目录结构导览

### 1.1 顶层构成

| 路径 | 内容 |
|---|---|
| `LocalCTP.sln` | VS2019 解决方案，2 个工程：`LocalCTP`（交易 API 动态库）、`TestLocalCTP`（DEMO 客户端） |
| `ReadMe.md` | 作者秋水(Aura)的项目说明，接口清单/账户规则/配置项/投喂行情字段映射表 |
| `buildLinux.sh` | 一键多版本构建：遍历 `ctp_file/<ver>` → 复制为 `current` → 跑 `GenScript` → `make` → 改名归档 |
| `GenScript/ParseCTPHeaders.py` | 代码生成器（568 行），解析 CTP 头文件生成 3 个自动代码文件 |
| `LocalCTP/` | 动态库全部源码（核心） |
| `TestLocalCTP/` | DEMO 测试客户端（`TestLocalCTP.cpp` 322 行） |
| `bin/win/x64/Release/`、`bin/linux/` | 预编译产物：`thosttraderapi_se.dll`、`libthosttraderapi_se.so`、`instrument.csv`、`localctp.config` |

### 1.2 `LocalCTP/` 源码文件（按重要性）

| 文件 | 行数 | 职责 |
|---|---|---|
| `LocalTraderApi.cpp` | 2717 | 核心：全部 Req* 实现、行情快照处理、撮合、资金/持仓更新、账户重载 |
| `LocalCTP.cpp` | 1999 | OrderData 行为（状态机/成交回报构造）、结算线程与结算单生成、`instrument.csv` 解析 |
| `LocalTraderApi.h` | 631 | `CLocalTraderApi`、`OrderData`、`PositionData`、`CMessageQueue`、`CSettlementHandler` 声明 |
| `auto_generated_code/CTPApiHugeMacro.h` | 1861 | 自动生成：每个 SPI 回调的 Msg 结构体 + `VARIANT_MSG_TYPE_MACRO` + `MESSAGE_HANDLE_MACRO` + `UNSUPPORTED_CTP_API_FUNC` |
| `auto_generated_code/CTPSQLWrapper.h` | 13061 | 自动生成：每个 CTP 结构体的 `Wrapper`（建表/INSERT/SELECT/反序列化） |
| `auto_generated_code/CTPSQLWrapper.cpp` | 1465 | 自动生成：Wrapper 静态 SQL 字符串 |
| `ctpStatus.h` | 313 | 错误消息文本、状态机中文消息、条件单判定、交易日工具函数、浮点比较宏所在文件的同伴（`stdafx.h`） |
| `CSqliteHandler.cpp/.h` | 420/55 | sqlite3 内存库 + 文件库双库、定时同步、事务 RAII |
| `LeeDateTime.cpp/.h` | 538/152 | 跨平台日期时间 |
| `stdafx.h` | 176 | 导出宏 `ISLIB/LIB_TRADER_API_EXPORT`、`EPS` 浮点比较（EQ/GE/LT…）、配置结构 `LocalCTPConfig`、GBK/UTF8 转换声明 |
| `Variant.hpp` | — | 第三方 `qicosmos/cosmos` 变体类型，用于消息队列元素 |
| `Properties.cpp/.h` | — | 第三方 `quantcast/qfs` 的配置文件解析（读 `localctp.config`） |
| `ctp_file/` | — | 9 套 CTP 头文件 + `error.xml`：`6.3.15 / 6.3.19 / 6.5.1 / 6.6.1 / 6.6.9 / 6.7.0 / 6.7.2 / 6.7.8`，`current` 为当前生效副本 |

### 1.3 定位结论（先回答三个总问题）

1. **它实现了什么**：一个**进程内仿 CTP 交易柜台**，以 `thosttraderapi_se.dll`（Linux 为 `.so`）形态发布，**整体替换客户端原有的 CTP 交易 API 库**。用户在客户端内下单，库在进程内部完成风控、撮合、持仓/资金更新，再通过 SPI 回调把回报推给策略（`ReadMe.md:77-83`）。
2. **不是 FTD 协议服务端**：完全没有 FTD 报文层——没有 TCP 监听、没有 FTDC 序列化/压缩（`TZ`/`LZ`）、没有组播、没有心跳/重传。见 §四.6。
3. **API 版本与真实客户端兼容性**：`ctp_file/` 内置 9 个版本头文件，切换后运行 `GenScript/ParseCTPHeaders.py` 重生成代码即可适配，默认 v6.5.1（`ReadMe.md:50-71`、`buildLinux.sh:52-55`）。任何按对应版本头文件编译的真实 CTP 客户端**都可以直接加载它**——导出方式就是把 `CThostFtdcTraderApi` 类声明改为 dllexport：`stdafx.h:43-44` 定义 `ISLIB/LIB_TRADER_API_EXPORT` 后再包含 CTP 头文件，导出 `CreateFtdcTraderApi`（`LocalTraderApi.cpp:1230`）与 `GetApiVersion`（`LocalTraderApi.cpp:1259`）。不含行情 API；行情需外部从实盘行情 API 取得快照后**投喂**进来。

### 1.4 行情投喂（两个被"魔改"的接口）

- `RegisterFensUserInfo(CThostFtdcFensUserInfoField*)`：参数被强制转换成 `CThostFtdcDepthMarketDataField*` 处理（`LocalTraderApi.cpp:1560-1565`）。
- `ReqQuoteInsert(CThostFtdcInputQuoteField*, nRequestID)`：按 `ReadMe.md:213-248` 的字段映射表（TradingDay→BrokerID、LastPrice→UserID 字符串化等）还原为快照，`nRequestID` 被当作 Volume（`LocalTraderApi.cpp:2677-2717`）。
- 两者最终都汇入 `onSnapshot()`（`LocalTraderApi.cpp:247-528`）：更新行情缓存 → 推进回测时钟 → 检查条件单触发 → 遍历报单撮合 → 重算持仓盈亏/资金。

### 1.5 GenScript 生成什么（`GenScript/ParseCTPHeaders.py`）

解析三份输入（数据类型 typedef、结构体、API 纯虚/SPI 虚函数），产出：

1. **CTPSQLWrapper.h/.cpp**：每个结构体一个 `XXWrapper`，含 `CREATE_TABLE_SQL`（VARCHAR 长度按 GBK→UTF8 翻倍设计）、`generateInsertSql`（`REPLACE INTO`，字符串字段做 GBK/UTF8 转换，见 `needConvertMemberNames` 列表）、`generateSelectSqlByUserID`（按 BrokerID+InvestorID/AccountID/UserID 过滤）；主键来自预定义表 `predefinedTableKey`（`ParseCTPHeaders.py:23-34`）。`CThostFtdcTradeField` 额外附加 Commission/CloseProfit/CashIn 三个 double 字段持久化（`ParseCTPHeaders.py:115-117`）。
2. **CTPApiHugeMacro.h**：每个 SPI 函数一个 `XxxMsg` 快照结构体（拷贝参数值 + 指针空标志），汇总为 `Variant<...>` 消息类型与 `MESSAGE_HANDLE_MACRO`（`ParseCTPHeaders.py:505-552`）。
3. **UNSUPPORTED_CTP_API_FUNC**：凡不在 `exclusiveApiFunctions` 白名单里的 API 一律生成 `return -1` 空实现（验证码登录、询价、报价、执行宣告、期权自对冲、银期转账等约 60 个）；`ReqQryClassifiedInstrument` 有手写特例实现（`CTPApiHugeMacro.h:1812-1856`）。

### 1.6 TestLocalCTP（DEMO 客户端）覆盖场景

`TestLocalCTP/TestLocalCTP.cpp` 是一个完整 SPI 打印客户端，按序覆盖：

- 认证→登录→查结算单→确认结算单（`:186-193`）
- 查合约（ProductID=MA）、查产品（`:195-208`）
- **无行情下单被拒**（预期路径，`:211-212`）
- `RegisterFensUserInfo` 喂 MA509/MA511 快照后，下**组合合约 SPD 单**（负限价、`IsSwapOrder=1` 互换单，`:235-238`）
- 卖出平仓 + `ReqOrderAction` 撤单（`:240-247`）
- **条件单**（LastPrice>StopPrice）+ `ReqQuoteInsert` 喂价触发（`:249-268`）
- **期权** IO2509-C-3300 买开 2 手/卖平 1 手（`:270-285`）
- 四类查询：报单（有数据）、成交（DCE 空）、资金、持仓（`:287-300`）
- 喂一个晚于结算时间的快照触发结算（仅回测模式，`:302-304`）

---

## 二、核心流程时序

### 2.0 SPI 回报总线（所有回调的唯一驱动）

`CMessageQueue`（`LocalTraderApi.h:166-225`）：每个 API 实例一条工作线程 + `std::deque<Variant>`。所有回报都先 `addMsg` 入队，由队列线程统一 `Visit` 调 SPI。为防止 SPI 内再触发 API 操作形成递归死锁，线程每次**成批搬走队列**再逐条派发。行情驱动的报单/成交回调因此**晚于**同步返回的 `Req*`（在 `RegisterFensUserInfo` 调用的线程栈内先完成 `updateByTrade`，再逐条发 `OnRtnTrade`/`OnRtnOrder`）。

### 2.1 登录/认证时序

```
Init()                             -> 入队 OnFrontConnected（LocalTraderApi.cpp:1454）
ReqAuthenticate                    -> 仅校验 UserID/BrokerID 非空 + 与上次一致（:1592-1616）
                                       成功: m_authenticated=true
ReqUserLogin                       -> 要求已认证且 UserID/BrokerID 与认证一致（:1619-1657）
                                       成功: m_logined=true
                                         reloadAccountData()（:1033-1194）
                                         回包 FrontID=m_frontID, SessionID=maxSessionID++
                                         MaxOrderRef 恒为 "1"（:1651）
```

要点：

- **不校验密码、不校验结算单确认**（`ReadMe.md:134-135,175`）。用户不存在则自动建账：初始资金 2000 万、持仓空（`LocalTraderApi.cpp:218-219,1136-1139`）。
- **会话唯一性**：`m_frontID = 构造时的 Unix 秒`（`LocalTraderApi.cpp:214`，每次进程启动不同）；`m_sessionID` 为进程级静态原子自增，从 0 开始（`LocalTraderApi.h:315`、`LocalTraderApi.cpp:1653`）。会话 key = `frontID_sessionID`（`LocalTraderApi.h:336-339`）。
- **OrderRef 生成规则**（`LocalTraderApi.cpp:2053-2090`）：客户端自带 OrderRef 时必须 ≥ 当前会话最大值，否则拒 `ErrMsgDuplicateOrder`；OrderRef 为空则自动取「会话内最大 OrderRef+1」（首单为 1）。注意这与真实 CTP「OrderRef 只要求不重复递增」基本一致。
- **多用户**：同一进程可建多个 API 实例、多个账户；`sqlHandler`（sqlite 内存库 `LocalCTP.db`）为进程级静态共享（`LocalTraderApi.h:330`）。**同一账户两个实例并行登录会互相覆盖**（持仓/资金在实例内存中各一份，却写同一张表）——属实现缺陷，对账时注意。
- **会话数限制/踢线**：**无**。不限制并发会话，无踢线逻辑；`ReqUserLogout` 仅清 `m_authenticated/m_logined` 标志，账户数据保留且仍可被喂行情更新（`LocalTraderApi.cpp:1661-1676`）。
- **交易日**：`GetTradingDay` 无须登录；算法为「当前时间 + 4 小时」再跳过周末（无法识别长假），并取「结算单表最大交易日+1」与原始值较大者（`LocalTraderApi.cpp:1480-1534`、`ctpStatus.h:69-85`）。

### 2.2 报单→风控→（撮合|挂起）时序

```
ReqOrderInsert(pInputOrder)
 ├─ CHECK_LOGIN_INVESTOR: 未登录/UserID 不符 -> return -1（LocalTraderApi.cpp:8-15,1702）
 ├─ 字段校验（详见 §2.6 清单），失败 -> sendRejectOrder：
 │    同时回 OnRspOrderInsert + OnErrRtnOrderInsert 两条（:1704-1707）
 ├─ doRiskCheck(preCheck=true)：只累加冻结需求并验资/验仓（:2044）
 ├─ 生成 OrderData（构造函数内同步回 2 条 OnRtnOrder：未知->未成交/未触发）（:2079/2089, LocalCTP.cpp:298-332）
 ├─ m_orderData[sessionKey].emplace(OrderRef, ...) 失败 -> 拒单（:2093-2098）
 ├─ 条件单：登记 m_contionalOrders 后直接返回（:2100-2104）
 ├─ doRiskCheck(preCheck=false)：真正冻结保证金/持仓（:2107）
 ├─ isMatchTrade(限价 vs 盘口) 命中 -> handleTrade（:2114-2120）
 │      命中: handleTrade -> updateByTrade(逐笔算费/盈亏) -> 每腿 OnRtnTrade -> OnRtnOrder(全部成交)
 └─ 未命中且 IOC+CV -> handleCancel(false)（FOK 语义）（:2125-2133）；GFD 则挂起等待行情
```

**撮合规则 `isMatchTrade`**（`LocalTraderApi.cpp:171-210`，仿 SimNow，只看对手价）：

- 买入成交条件：`LimitPrice >= AskPrice1`；卖出：`LimitPrice <= BidPrice1`。
- 组合合约（`SPD a&b`）：逐腿拆单腿（`GetSingleContractFromCombinationContract`，`:143-167`），偶数腿同向、奇数腿反向，`priceDiff` 按腿累加/累减，与限价比较；成交价取各腿盘口价。**组合合约报单不需要组合合约自身行情，只需两条腿行情**（`LocalTraderApi.cpp:1793-1802`、DEMO 亦如此）。
- **全部成交、无部分成交**：命中即按 `VolumeTotalOriginal` 整单成交（`LocalTraderApi.cpp:398`），`MinVolume`、FAK 未真正实现（IOC 仅表现为「不成交即撤」）。
- 撮合只由**新行情快照**驱动（`onSnapshot` 内遍历全部会话订单，`LocalTraderApi.cpp:357-401`）；报单进入时只尝试一次。

**条件单**（`LocalTraderApi.cpp:308-349`）：支持 4 种「最新价 vs 条件价」比较（`ctpStatus.h:34-40`）；触发时先回一条 `Touched` 的 OnRtnOrder，再以 `OrderRef=""`（自动编号）、`LimitPrice=StopPrice`、`Immediately` 重新走一遍报单流程，新单 `RelativeOrderSysID` 指向条件单（原条件单 OrderSysID 带前缀 `TJBD_`，`ctpStatus.h:30`、`LocalCTP.cpp:309-315`）。条件单**不做风控预校验**（`LocalTraderApi.cpp:1975-1979`），触发时才验资。

**撤单 `ReqOrderAction`**（`LocalTraderApi.cpp:2160-2214`）：

- `ActionFlag` 只允许 Delete（不支持改单，:2163-2167）。
- 两种定位方式：① `OrderRef + FrontID + SessionID`（还需 InstrumentID 一致）；② `OrderSysID + ExchangeID` 全表扫描。
- 撤成功 → `handleCancel`：置 `Canceled`、填 CancelTime/ActiveUserID → `updateByCancel` 解冻（撤开的单解冻保证金、撤平的单解冻持仓，:570-740）→ OnRtnOrder。终态单再撤返回 `ErrMsg_AlreadyDoneOrder`，找不到返回 `ErrMsg_NotExistOrder`。

### 2.3 查询时序（无流控）

所有 `ReqQry*` 都是「立刻遍历内存容器 → 逐条 `OnRspQryXxx` 入队，最后一条 `bIsLast=true`；空结果回一条 `nullptr+bIsLast=true`」。典型见 `ReqQryOrder`（`LocalTraderApi.cpp:2263-2291`）、`ReqQryTradingAccount`（:2354-2361）。

- **无在途查询限制、无每秒频次限制、无 RequestID 去重**（nRequestID 仅透传）。
- 查询走**内存**（`m_orderData/m_positionData/m_mdData`），不走 DB；过滤条件是「字段为空或相等」（`COMPARE_MEMBER_MATCH` 宏，`LocalTraderApi.h:14-15`）。
- 与 CTP 的差异：`ReqQryInstrumentMarginRate/ReqQryInstrumentCommissionRate` 返回**所有**符合条件合约的费率而非单条（`LocalTraderApi.cpp:2377-2422，ReadMe.md:136`）。
- 结算单查询按 `sizeof(Content)-1` 分片推送，DB 中 base64 存储（`LocalTraderApi.cpp:2525-2582`）。

### 2.4 每日结算时序（`CSettlementHandler`）

线程构造即启动（`LocalCTP.cpp:578-625`）：先睡 3s 等 `initInstrMap`，计算下次结算时刻（默认交易日 17:00，`localctp.config` 可配）。每 2 分钟（回测 10s）`checkSettlement`：是交易日 + 已到时刻 + 结算单表无当天记录 → `doSettlement`：

1. `doWorkInitialSettlement`（`:1177-1388`）：持仓明细按结算价重算盈亏/保证金；**到期合约模拟强平**（持仓置 0，期权按结算价计权利金收支，无手续费无成交记录）；**昨仓合并进今仓**（多条字段累加，`:1294-1372`）；SQL 汇总更新资金。
2. 逐账户生成结算单正文存入 `SettlementData` 表（`doGenerateUserSettlement`，`:1495`）。
3. `doWorkAfterSettlement`（`:1390-1458`）：清零当日字段、`PreBalance=Balance`、`PreSettlementPrice=SettlementPrice`、`YdPosition=Position` 等；推进 `tradingDay`；对所有 API 实例 `reloadAccountData()`。
4. 可选结算后退出进程（`exit_after_settlement`）。

回测模式额外行为：启动即**清空**数据库中所有账户数据（`LocalTraderApi.cpp:1235-1250`）；内部时钟取行情时间戳；每 100 个快照才落库一次（`:411-424`）。

---

## 三、关键数据结构

### 3.1 OrderData 订单状态机（`LocalTraderApi.h:270-311`、`LocalCTP.cpp`）

```
                 OrderData 构造(dealTestReqOrderInsertNormal)
                 ┌──────────────────────────────────────────────┐
                 ▼                                              │
  OST_Unknown('a') --提交--> 普通单 OST_NoTradeQueueing('3')      │ 条件单 OST_NotTouched('b')
                              │        ▲                         │        │ 行情触发
                    行情命中  │        │ 未成交挂起                ▼        ▼
                              ▼        │                   OST_Touched('c')→新单(Immediately)
                     OST_PartTradedQueueing('1') → 全部成交 → OST_AllTraded('0')
                     （实际一次命中即全量，无部分成交路径）
  任意未终态 --撤单/IOC未成交--> OST_Canceled('5')
```

- `isDone()`：非 `PartTradedQueueing/NoTradeQueueing/Unknown/NotTouched` 即终态（`LocalCTP.cpp:290-296`）。
- 每态 `StatusMsg` 由 `getStatusMsgByStatus` 给中文（`ctpStatus.h:42-67`）。
- 一条报单的数据：`inputOrder`（不可变）+ `rtnOrder`（可变回报态）+ `rtnTrades`（成交列表）；成交回报 `getRtnTrade` 逐腿生成，`TradeType` 单腿为 Common、组合为 CombinationDerived（`LocalCTP.cpp:389-489`）。
- 编号规则：`OrderLocalID = OrderRef`；`OrderSysID/TradeID` 为**按交易所**的进程级计数器，种子是启动时毫秒时间戳 `initStartTime`（`LocalTraderApi.h:139-141,341-361`），即**跨账户共享、不按用户隔离**；`BrokerOrderSeq = OrderSysID`（`LocalCTP.cpp:325`）；`ZCETotalTradedVolume` 仅 CZCE 维护（:340）。

### 3.2 持仓/资金

| 结构 | 位置 | 要点 |
|---|---|---|
| `PositionData` | `LocalTraderApi.h:239-266` | pos + `posDetailData` 明细向量，按 `(OpenDate, TradeID)` 排序实现**先开先平**（:251-263） |
| 持仓 key | `instrumentID_方向_持仓日期`（:395-406） | 仅 SHFE/INE 区分今昨仓（`isSpecialExchange`，:375-378），其他所平仓一律按今仓处理 |
| `m_tradingAccount` | `LocalTraderApi.h:452` | 内存单份 + DB `CThostFtdcTradingAccountField` 表 |

计算口径（`LocalTraderApi.cpp`）：

- **冻结保证金**（开仓报单时）：`昨结算价 × VolumeMultiple × 手数 × 保证金率 + 手数 × 每手保证金率`（:1834-1836）；基准价是**昨结算价**（:1808-1813 注释）。
- **默认费率**：保证金率 10%（多空同）、手续费开/平/平今各 1 元每手（`Init` 时写入，:1433-1451）；DB 费率表无记录时用它。
- **手续费**：开仓 `价×乘数×量×OpenRatioByMoney + 量×OpenRatioByVolume`（:784-786）；平仓分平昨/平今两段费率（:975-981）。
- **持仓盈亏**（随行情/结算价 mark）：`±(价×乘数×持仓量 − 持仓成本)`；**优先用结算价、无结算价用最新价**（:403-408）。今仓成本用开仓价、昨仓用昨结算价（:996-997）。
- **平仓盈亏**：平昨用 `成交价 − 昨结算价`，平今用 `成交价 − 开仓价`（:904-929）；每笔平仓写 `CloseDetail` 明细表。
- **资金公式**（`updatePNL`，:554-561）：`Balance = PreBalance + Deposit − Withdraw + PositionProfit + CloseProfit + CashIn − Commission`；`Available = Balance − CurrMargin − FrozenMargin − FrozenCash`。
- **权利金**：期权买入冻结/收支 `FrozenCash/CashIn`，卖出按期货公式算保证金（:1821-1833）。
- **简化假设清单**（对账重点）：
  1. 期权**不算持仓盈亏、不算平仓盈亏**（`:444-462,890-940`），保证金直接用期货绝对值公式；
  2. 无 `FrozenCommission` 内存记账（结算 SQL 里有该列，`LocalCTP.cpp:564`，但 `updatePNL` 未扣）；
  3. 无持仓明细级的保证金率调整、无大额单边保证金、无品种/跨品种保证金；
  4. 入金出金无接口，`Deposit/Withdraw` 永 0（只能改 DB）；
  5. 结算=17:00 一次性事件，长假/节假日不识别（`ctpStatus.h:71-77` TODO）；
  6. 报单不校验结算单确认状态。

### 3.3 持久化（`CSqliteHandler`）

- 内存库 + 文件库双库，`Attach` 后 `INSERT OR REPLACE INTO 文件表 SELECT * FROM 内存表` 全量同步，后台线程每 10s 一次、退出时补一次（`CSqliteHandler.cpp:16-29,47-82`）。
- 建表 SQL 由 GenScript 生成；表清单：持仓、持仓明细、委托、成交、资金、合约、保证金率、手续费率、`CloseDetail`、`SettlementData`（`LocalTraderApi.cpp:133-138,1285-1294`）。
- `CSqliteTransactionHandler` 为 RAII 事务（`CSqliteHandler.h:45-55`）。

### 3.4 错误码体系（**不对齐 error.xml**）

- 统一 `m_successRspInfo{0,"success"}` 与 `m_errorRspInfo{-1,"error"}`（`LocalTraderApi.cpp:216`）；**所有失败 ErrorID 恒为 −1**，具体原因写在 `ErrorMsg` 里，文案刻意仿 CTP（`ctpStatus.h:8-31`）：如「CTP:资金不足」「CTP:平仓时持仓不足,当前可平数量:N」「CTP:重复的报单(OrderRef跟当前最大值不符)」「CTP:此合约没有缓存行情数据:XX」「CTP:不支持的价格类型(OrderPriceType)」「CTP:不合法的字段(本系统只支持投机套保)(CombHedgeFlag)」等。
- 好处：只判 `ErrorID==0` 的客户端行为与真实 CTP 完全一致；依赖 error.xml 数值码做业务分支的客户端则无法复用。
- 报单拒绝同时触发 `OnRspOrderInsert` + `OnErrRtnOrderInsert`（双通道，:1704-1707）。

### 3.5 FTD 报文编解码：**不存在**

- 无 FTDC 结构体模板、无类型压缩（`T`/`Z`/`L` 序列）、无多播、无断线重传/心跳（那些都是 CTP **前置/服务端**的职责；本仓库是客户端库替换方案）。
- SPI 回调的「序列化」由 `Variant<...所有 XxxMsg>` 变体完成：消息入队时把参数**值拷贝**进 Msg 结构体（指针参数附带 IsNull 标志），派发时 `MESSAGE_HANDLE_MACRO` 还原为空/值指针调 SPI（`CTPApiHugeMacro.h:1376,1506`）。这是纯内存对象拷贝，不是协议编解码。

---

## 四、对 CTPBuddy 有借鉴/对账意义的点

1. **架构分型**：LocalCTP 是「API 库替换/进程内柜台」，与「FTD 网络服务端」是两条路线。CTPBuddy 若走服务端路线，则 LocalCTP 的会话、流控、踢线、心跳全部不在其实现内，需另找基准；若做客户端兼容，则可直接复用其导出宏技巧（`stdafx.h:43-44`）。
2. **会话与 OrderRef 规则可直接对账**：FrontID=Unix 秒、SessionID 进程自增、会话 key=`front_session`、OrderRef 空则取会话内 max+1、重复/回退拒单——与真实 CTP 行为一致，可作为 CTPBuddy 报单引用的验收标准（`LocalTraderApi.cpp:2053-2098`）。
3. **OrderSysID/TradeID 的坑**：按交易所全局计数、种子为毫秒时间戳、跨用户不隔离；条件单另有 `TJBD_` 前缀。CTPBuddy 若按「用户维度」或「数据库自增」实现，需明确差异（`LocalTraderApi.h:341-361`、`ctpStatus.h:30`）。
4. **撮合逻辑**：限价 vs 对手价（买≥Ask1/卖≤Bid1）、组合合约逐腿反向累加减、整单成交无部单、只在行情驱动时撮合——与 SimNow 同口径，可作为 CTPBuddy 撮合引擎的参照（`LocalTraderApi.cpp:171-210,357-401`）。
5. **风控两段式**：预检（只算不冻）+ 落库（真冻结），条件单跳过预检；开仓冻保证金（昨结算价计价）、平仓冻持仓、期权买入冻权利金——冻结/解冻路径完备（撤单、成交双向冲销，`updateByCancel/updateByTrade`）。
6. **流控为零**：报单、查询均无频次/在途限制，CTPBuddy 若按真实 CTP 加流控（如查询在途 1 个、每秒 N 次查询、每秒 N 笔报单）， LocalCTP 无法作为该维度基准，反而应作为「无流控对照端点」。
7. **盈亏/保证金公式**：§3.2 列表可直接作为单测用例基准；特别注意期权盈亏恒 0、`Available` 不扣 `FrozenCommission`、昨仓成本用昨结算价这三处简化。
8. **错误码策略**：ErrorID=−1 + 仿 CTP 文案。CTPBuddy 需决定：对齐 error.xml 数值码（更严格）还是沿用 −1+文案（更省事、对只判 0 的客户端等价）。
9. **结算流程**：到期强平（无手续费）、昨仓合并今仓、`PreBalance=Balance` 滚存、SHFE/INE 保留昨仓记录（`LocalCTP.cpp:1380-1382`）——结算字段重置清单在 `doWorkAfterSettlement`（:1426-1433）写得很全，可作字段级对照表。
10. **GenScript 的版本适配手法**值得借鉴：解析 CTP 头文件自动生成 SQL Wrapper 与不支持函数占位，换版本零手改。CTPBuddy 若要多版本兼容（6.3.15~6.7.x 结构体差异大），这是现成方案；注意其字符串列长度按 GBK→UTF8 翻倍、`StatusMsg` 等字段跨库做编码转换（`ParseCTPHeaders.py:64-74,286-290`）。
11. **已知缺陷/边界**（移植时避开）：同账户多实例内存态互相覆盖；回测模式启动清库；长假不识别；FAK/部成未实现；`MaxOrderRef` 恒返回 "1"；登出后数据仍在线（`ReadMe.md:160,175`）。
