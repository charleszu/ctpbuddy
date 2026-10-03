# 10 银期转账 / FIX 网关 / 国际版 / TGate

> 本章覆盖：银期转账（TradeApi）、FIX 网关与 FIX 接口规范、国际版差异、TGate 交易网关接入、中继/看穿式相关接口。
> 方法/结构体/枚举的完整速查表见《13_API方法速查表》等；本章只写流程、规则、坑。

---

## 1 银期转账

### 1.1 要点速查

| 项 | 内容 |
|---|---|
| 指令分类 | ① 非银行交互指令（走查询流，与交易平台直接通讯）② 银期交互指令（涉及交易平台与银行转账服务器，时序类似交易指令） |
| 非银行交互指令 | `ReqQryContractBank`→`OnRspQryContractBank`（查询签约银行，可得银行编号 BankID 与分支号 BankBranchID）；`ReqTradingAccountPasswordUpdate`→`OnRspTradingAccountPasswordUpdate`（资金账户口令更新）；`ReqQryTransferSerial`→`OnRspQryTransferSerial`（查询转账流水） |
| 投资者终端常用银期交互业务 | 204002 期货发起查询银行余额 `ReqQueryBankAccountMoneyByFuture`；202001 银行转期货；202002 期货转银行（见 1.3 说明） |
| 最终成功信息来源 | 一律来自 `OnRtn*`（银行处理完毕后的回报）；`OnRsp*` 仅表示交易核心校验/受理结果 |
| 错误来源 | 校验不通过：`OnRsp*` 返回错误；银行返回错误：`OnErrRtn*` |
| 行分支机构代码 | `BankBranchID` 填 `'0000'` |
| 资金密码核对标志 | `SecuPwdFlag = THOST_FTDC_BPWDF_BlankCheck`（明文核对） |
| 银行密码核对标志 | `BankPwdFlag = THOST_FTDC_BPWDF_NoCheck`（不核对） |
| 证件号核对 | `VerifyCertNoFlag = THOST_FTDC_YNI_No` |
| 币种 | `CurrencyID = "RMB"` |
| 农行/中行 | 涉及农行/中行的指令需要输入银行密码（`BankPassWord`） |
| 6.7.13 变化 | `OnRspQryContractBank` 响应增加 `csrcBankID`（上报 csrc 的银行代码） |

### 1.2 银期交互指令总体时序

```
终端 ──Req──▶ 交易平台(核心 tkernel) ──校验──▶ 不通过: OnRsp*(ErrorID≠0)
                                   └─通过──▶ 转发银行转账服务器
银行处理 ──▶ 交易平台 ──▶ 成功: OnRtn*     失败: OnErrRtn*
```
- 以“期货发起查询银行可转资金”为例：`ReqQueryBankAccountMoneyByFuture` → `OnRspQueryBankAccountMoneyByFuture`（核心响应）→ `OnRtnQueryBankBalanceByFuture`（银行最终结果）/ `OnErrRtnQueryBankBalanceByFuture`（错误）。
- 新版 TradeApi 说明：使用 `OnRsp*` 响应（含更新后的请求包体与 pRspInfo）之后，可以不再使用第 4 步“交易核心错误回报（OnErrRtn*）”——即 `OnRsp*` 的 `pRspInfo->ErrorID != 0` 已能表达核心拒绝。（工程建议：仍同时处理两者，并以 `OnRtn*` 为最终成功依据。）

### 1.3 业务代码与接口对照

| 业务代码 | 业务 | 请求 | 核心响应 | 银行处理后回报 | 错误回报 |
|---|---|---|---|---|---|
| 204002 | 期货发起查询银行余额 | `ReqQueryBankAccountMoneyByFuture`（`CThostFtdcReqQueryAccountField`） | `OnRspQueryBankAccountMoneyByFuture` | `OnRtnQueryBankBalanceByFuture`（`CThostFtdcNotifyQueryAccountField`） | `OnErrRtnQueryBankBalanceByFuture` |
| 202001 | 银行资金转期货（银行→期货） | `ReqFromBankToFutureByFuture`（`CThostFtdcReqTransferField`） | `OnRspFromBankToFutureByFuture` | `OnRtnFromBankToFutureByFuture`（`CThostFtdcRspTransferField`） | `OnErrRtnBankToFutureByFuture` |
| 202002 | 期货资金转银行（期货→银行） | `ReqFromFutureToBankByFuture` | `OnRspFromFutureToBankByFuture` | `OnRtnFromFutureToBankByFuture` | `OnErrRtnFutureToBankByFuture` |

> 版本/来源差异提示：个别资料在业务代码表中把 202001 与 `ReqFromFutureToBankByFuture` 并列、202002 与 `ReqFromBankToFutureByFuture` 并列，与业务描述（“202001=银行资金转期货”“202002=期货资金转银行”）及函数名含义不一致。实现时**以函数名含义为准**：`ReqFromBankToFutureByFuture` 为银行转期货（入金），`ReqFromFutureToBankByFuture` 为期货转银行（出金）；`TradeCode` 以柜台/银行实际配置核对（工程建议：在测试环境用小额验证 TradeCode 与方向的对应关系后再上线）。

### 1.4 请求参数（伪码）

```cpp
// 查询银行余额  TradeCode = "204002"
CThostFtdcReqQueryAccountField fld{};
strcpy(fld.TradeCode, "204002");
strcpy(fld.BrokerID, brokerId);
strcpy(fld.BankID, bankId);
strcpy(fld.BankBranchID, "0000");
fld.RequestID = reqId;
fld.SecuPwdFlag   = THOST_FTDC_BPWDF_BlankCheck; // 资金密码明文核对
fld.BankPwdFlag   = THOST_FTDC_BPWDF_NoCheck;    // 银行密码不核对
fld.VerifyCertNoFlag = THOST_FTDC_YNI_No;
strcpy(fld.AccountID, accountId);   // 资金账号(投资者帐号)
strcpy(fld.Password, fundPwd);      // 资金密码
strcpy(fld.CurrencyID, "RMB");
strcpy(fld.BankPassWord, bankPwd);  // 农行/中行必填
api->ReqQueryBankAccountMoneyByFuture(&fld, nRequestID);

// 银行转期货 TradeCode = "202001"；期货转银行 TradeCode = "202002"（其余参数相同，另加 TradeAmount）
CThostFtdcReqTransferField t{};
strcpy(t.TradeCode, "202001");
/* BrokerID/BankID/BankBranchID("0000")/RequestID/SecuPwdFlag/BankPwdFlag/VerifyCertNoFlag/
   AccountID/Password/CurrencyID("RMB")/BankPassWord 同上 */
t.TradeAmount = 1000.0;             // 转账金额
api->ReqFromBankToFutureByFuture(&t, nRequestID);
```

**必填字段（期货发起转账/查询余额）**：业务功能码 `TradeCode`、银行代码 `BankID`、银行分支机构代码 `BankBranchID`（填 `'0000'`）、期货公司代码 `BrokerID`、银行帐户密码 `BankPassWord`、投资者帐号 `AccountID`、资金帐户密码 `Password`、资金帐户密码核对标志 `SecuPwdFlag`（值为 `BPWDF_BlankCheck`）；转账另需币种 `CurrencyID`、转账金额 `TradeAmount`（查询余额无金额）。

### 1.5 查询转账流水
- 请求：`ReqQryTransferSerial(CThostFtdcQryTransferSerialField*, nRequestID)`，字段 `BrokerID`（必填）、`AccountID`（投资者帐号，必填）、`BankID`（银行编码）。
- 应答：`OnRspQryTransferSerial(CThostFtdcTransferSerialField*, pRspInfo, nRequestID, bIsLast)`；`ErrorID=0` 成功，`bIsLast` 表示是否有后续包体。
- 作用：转账状态不确定（超时/未收到 OnRtn）时，以流水为准做对账。

### 1.6 银行发起 / 冲正类通知（只需接收）
| 通知 | 含义 | 包体 |
|---|---|---|
| `OnRtnFromBankToFutureByBank` | 银行发起银行资金转期货通知 | `CThostFtdcRspTransferField` |
| `OnRtnFromFutureToBankByBank` | 银行发起期货资金转银行通知 | `CThostFtdcRspTransferField` |
| `OnRtnRepealFromBankToFutureByBank` | 银行发起冲正银行转期货通知 | `CThostFtdcRspRepealField` |
| `OnRtnRepealFromFutureToBankByBank` | 银行发起冲正期货转银行通知 | `CThostFtdcRspRepealField` |
| `OnRtnRepealFromBankToFutureByFuture` | 期货发起银行转期货**自动冲正**通知 | `RspRepeal`（银行处理结果） |
| `OnRtnRepealFromFutureToBankByFuture` | 期货发起期货转银行**自动冲正**通知 | `RspRepeal` |
| `OnRtnRepealFromBankToFutureByFutureManual` | 期货发起银行转期货**手工冲正**通知 | `RspRepeal` |
| `OnErrRtnRepealBankToFutureByFutureManual` | 期货发起银行转期货手工冲正**错误**通知 | `CThostFtdcReqRepealField` + RspInfo（交易核心处理结果） |
| `OnRtnRepealFromFutureToBankByFutureManual` | 期货发起期货转银行手工冲正通知 | `RspRepeal` |
| `OnErrRtnRepealFutureToBankByFutureManual` | 期货发起期货转银行手工冲正错误通知 | `ReqRepeal` + RspInfo |

### 1.7 转账状态、错误与处理流程
1. 发起请求，记录 `RequestID`、`TradeCode`、`BankID`、金额与本地时间（本地流水）。
2. `OnRsp*`：`ErrorID≠0` → 核心拒绝（密码、签约、时间窗口、金额等校验），本地流水置失败；`ErrorID=0` → 已受理，等待银行。
3. `OnRtn*`：最终结果；（工程建议）以 `RspTransfer` 中的 `ErrorID`、`TransferStatus`、`BankSerial`/`PlateSerial` 等字段更新流水。
4. `OnErrRtn*`：银行/核心错误，置失败。
5. 超时无回报：调用 `ReqQryTransferSerial` 与 `ReqQryTradingAccount` 对账，禁止盲目重发（避免重复出入金）。
6. 冲正通知出现时，按冲正结果回滚本地流水，再 `ReqQryTradingAccount` 校验资金。

### 1.8 常见坑（现象→原因→处理）
| # | 现象 | 原因 | 处理 |
|---|---|---|---|
| B1 | 只收到 OnRsp 成功却不见资金变化 | OnRsp 仅表示核心受理，最终结果在 OnRtn | 以 OnRtn* 为准；超时对账流水 |
| B2 | 农行/中行查询、转账失败 | 需输入银行密码 | 填 `BankPassWord` |
| B3 | 返回参数错误 | `BankBranchID` 未填 '0000'、`SecuPwdFlag` 未设为明文核对 | 按 1.4 填写 |
| B4 | 不知道 BankID/分支号 | 未查签约银行 | 先 `ReqQryContractBank` |
| B5 | 重复发起导致重复转账 | 超时后无对账直接重发 | 先查流水再决定 |
| B6 | 新版 OnRsp 已报错还等 OnErrRtn | 使用 OnRsp 后可不再依赖 OnErrRtn | 以 OnRsp 的 ErrorID 判断核心失败 |

### 1.9 签约/解约
签约、解约（由银行或期货公司发起）的 API 通知与请求在《13_API方法速查表》中列出；终端侧通常只需 `ReqQryContractBank` 获取已签约银行，并对账户密码更新使用 `ReqTradingAccountPasswordUpdate`（响应 `OnRspTradingAccountPasswordUpdate`）。

---

## 2 FIX 网关与 FIX 接口规范

### 2.1 背景与定位
- CTP FIX 网关是在 CTP 接口上开发的 FIX 接入层，为国外交易终端接入国内期货（背景：INE 推出原油期货）提供标准 FIX 协议入口：接收并解析国外终端的 FIX 请求，并把 CTP 的响应转换成 FIX 消息返回。
- 协议版本：**FIX.4.2**，使用标准头（StandardHeader）与标准尾（StandardTrailer）。
- 网关由 **FIX 行情网关** 与 **FIX 交易网关** 两部分组成（统称 FIX 网关），业务报文不同，其它请求消息保持一致。

### 2.2 交易网关 vs 行情网关
| 能力 | 交易网关 | 行情网关 |
|---|---|---|
| 重发请求 `35=2`（最多重发 **2500** 条响应回报） | 支持 | 不支持 |
| 客户端认证 `35=A, 116=12304` | 支持 | 不支持 |
| 用户口令更新 `35=A, 116=12298` | 支持 | 不支持 |
| 会话层拒绝请求消息 `35=3` 来自终端 | 不支持 | 不支持 |

### 2.3 会话与身份规则
- 同一个 FIX 网关，**不允许同一交易终端用户多个 session 同时在线**。
- 消息流向：请求消息 `49-SenderCompID`=用户代码、`56-TargetCompID`=经纪公司代码；响应回报相反（49=经纪公司代码、56=用户代码）。
- 同一 `35-MsgType` 但目的不同的消息，用 `116-OnBehalfOfSubID` 区分。
- 请求/响应对应：响应的 `57-TargetSubID` 等于请求的 `50-SenderSubID` 时表示属于该请求（可对应多条响应）。`50` 为整型；回报中 `57` 默认 0。
- 重置序号请求成功时网关**不给任何响应**。
- 心跳间隔：通过认证/登录请求的 `108-HeartBtInt` 设置，有效范围 **[30,60] 秒**，否则拒绝；先认证再登录且两者不同，**以登录请求的为准**。
- 口令更新：`96-RawData` 格式“旧口令:新口令”，**口令中不能含“：”**，否则可能产生违背用户意愿的结果。
- 登录后需等待本用户的报单与成交回报全部接收完毕再报单，否则开平标志(77)为空时网关代算的开平可能出错。
- 价格类字段格式如 `54210.05`（参照国内交易所格式）。
- 查询请求：有结果才有响应，无符合条件结果则**不会收到任何响应**；若 CTP 查询系统未准备好，用户被强制登出，`58-Text` 为 `"FIX: CTP Query Engine not ready"`。
- `58-Text` 格式：CTP 返回为 `ErrorID=**，ErrorMsg=CTP:***`；FIX 组件返回为 `FIX:***`。
- 结算单确认：CTP 要求投资者每日首次报单前确认前一日结算单；FIX 网关默认**不**自动确认，可申请由网关在每日首次登录成功后自动确认（风险由申请者承担）；操作员（非投资者）身份不需要确认。

### 2.4 请求/响应类型表（35 + 116）
| 方向 | 35 | 116 | 说明 |
|---|---|---|---|
| 终端→网关 | A | 12304 | 客户端认证请求 |
| 终端→网关 | A | 12288 | 用户登录请求 |
| 终端→网关 | A | 12298 | 用户口令更新请求 |
| 终端→网关 | V | 17409 | 订阅行情请求 |
| 终端→网关 | V | 17411 | 退订行情请求 |
| 网关→终端 | A | 12305 | 客户端认证成功 |
| 网关→终端 | A | 12289 | 用户登录成功 |
| 网关→终端 | A | 12299 | 用户口令更新成功 |
| 网关→终端 | A | 131080 | 用户口令更新失败 |
| 网关→终端 | W | 17410 | 订阅行情响应 |
| 网关→终端 | W | 17412 | 退订行情响应 |
| 网关→终端 | 8 | 61441 | 报单回报（OnRtnOrder） |
| 网关→终端 | 8 | 61442 | 成交回报（OnRtnTrade） |

### 2.5 标准头/尾字段
| Tag | 字段 | 类型 | 说明 |
|---|---|---|---|
| 8 | BeginString | String | `FIX.4.2` |
| 9 | BodyLength | Length | 消息长度 |
| 35 | MsgType | String | 0 心跳；1 测试请求；2 重发请求；3 会话拒绝；4 序号重置；5 登出；8 执行回报；9 撤改单拒绝；A 登录；AB 组合委托；D 单笔委托；G 改单；F 撤单；H 报单状态请求；V 行情订阅；Y 拒绝订阅；W 订阅应答/行情；d 合约应答；c 合约请求；j 业务层拒绝 |
| 49 / 56 | SenderCompID / TargetCompID | String | 见 2.3（长度 12/10） |
| 34 | MsgSeqNum | Int | 消息序号 |
| 50 / 57 | SenderSubID / TargetSubID | Int | 请求编号 / 响应编号 |
| 116 | OnBehalfOfSubID | String | 请求响应类型（见 2.4） |
| 43 | PossDupFlag | Boolean | 重传标识：Y=重传（收到服务器重发请求而重发 6-7 号时置 Y），默认 N |
| 52 | SendingTime | UTCTimestamp | `YYYYMMDD-HH:MM:SS` |
| 122 | OrigSendingTime | UTCTimestamp | 重发时原发送时间 |
| 10 | CheckSum | String | 校验和 |

`Req'd` 列含义：`Y1`=FIX 协议必填，网关不校验；`Y`=CTP 必填，网关校验；`Y1+Y`=FIX 必填且网关校验；`N`=非必填；`C`=条件必填。

### 2.6 会话层消息
| 消息 | 关键字段 |
|---|---|
| 客户端认证请求 (35=A,116=12304) | `1408 DefaultCstmApplVerID`(用户端产品信息,Y,≤10) · `96 RawData`(认证码,Y,16) · `98 EncryptMethod`(0=None/Other,1=PKCS,2=DES,3=PKCS/DES,4=PGP/DES,5=PGP/DES-MD5,6=PEM/DES-MD5) · `141 ResetSeqNumFlag`(N/Y) · `108 HeartBtInt`(Y1+Y) |
| 登录请求 (35=A,116=12288) | `95 RawDataLength`(C) · `96 RawData`(密码,≤40,Y) · 98 · 141 · 108 |
| 登出 (35=5) | `58 Text`(登出原因,≤80) |
| 口令更新 (35=A,116=12298) | `95` · `96 RawData`(≤81,“旧口令:新口令”) · 98 · 108 |
| 心跳 (35=0) | `112 TestReqID`（回复 TestRequest 时需要） |
| 测试请求 (35=1) | `112 TestReqID`(Y1) |
| 重发请求 (35=2) | `7 BeginSeqNo`；`16 EndSeqNo`（单条则相等，之后全部则为 0） |
| 会话拒绝 (35=3) | `45 RefSeqNum`；`372 RefMsgType`；`373 SessionRejectReason`(1=必填tag缺失,4=tag无值,5=值不正确/越界,6=数据格式错误,9=CompID问题,10=SendingTime精度问题,18=不支持的应用版本)；`58 Text` |
| 序号重置 (35=4) | `123 GapFillFlag`(N=重置并忽略MsgSeqNum,Y=GapFill MsgSeqNum有效)；`36 NewSeqNo`(Y1+Y) |

#### 2.6.1 认证 / 登录状态规则
1. 若 CTP 需要客户端认证，必须**先认证、后登录**；不需要则直接登录。
2. 认证成功后，网关拒绝该用户再次认证；登录成功后，拒绝再次认证/登录。
3. 登录成功之前，除认证、登录、登出外的任何消息（序号重置、业务请求等）一律拒绝。
4. 未收到前一条认证/登录响应前，或被登出且未收到登出响应前，发来的请求被**丢弃**：不响应、不累加请求序号。

#### 2.6.2 序号检查规则（tag34）
- 序号**高于**期望：检查过高时，网关发 ResendRequest，终端须重发指定区间，且 `43=Y`；不检查过高的消息（如登录）可以高序号登录，登录后下一条消息到达时网关请求重发前面缺失的业务层报文。
- 序号**低于**期望：`43=N` → 强制登出；`43=Y` → 按消息类型处理，非序号重置(35≠4)时校验 `52` 与 `122`，失败拒绝：条件需要的 tag 缺失 / tag 值为空 / `122 > 52`。
- 序号重置：同一连接只允许通过认证和登录请求成功重置序号**一次**（`141=Y`），此时 `34` 必须为 1 且不被检查过低。“重置序号”指把网关上维护的客户端请求序号列、服务端响应回报序号列各重置为 1。

| 消息 | 是否检查过高/过低 |
|---|---|
| 认证 (A,12304) / 登录 (A,12288) | 不检查过高，检查过低 |
| 序号重置 (4) | 由 `123 GapFillFlag` 决定：Y 则过高过低都检查，N 都不检查 |
| 重发请求 (2) | 都不检查 |
| 登出 (5) | 都不检查 |
| 心跳 (0)、测试请求 (1)、口令更新 (A,12298)、业务请求 | 过高过低都检查 |

#### 2.6.3 客户端请求序号累加规则
- 认证/登录成功：序号 +1；失败（被拒绝或被登出）：序号不变。
- 序号重置请求成功：客户端请求序号变为 `36-NewSeqNum`，下一条消息的 `34` 也应为该值；失败则不变。
- 其它请求（口令更新、重发、登出、心跳、测试、业务请求），不论成功或失败（被拒绝/被登出），只要该条消息的 `34` 与客户端请求序号相符，序号 +1；不符则不变。

#### 2.6.4 链路监测（定时器）
- 2.4 倍心跳间隔内未收到连接发来的报文 → 网关断开连接。
- 1.2~2.4 倍心跳之间未收到 → 网关主动发测试请求响应，`112-TestReqID="Test"`。
- 1~1.2 倍心跳之间 → 网关主动发心跳。

#### 2.6.5 拒绝类型
- 会话层拒绝 `35=3`：未通过会话层校验（版本不支持、必需 tag 缺失、tag 为空/值不正确、重发消息的发送时间晚于原发送时间等）。
- 业务层拒绝 `35=j`：通过会话层但被业务层规则拒绝（消息类型不支持、条件需要的 tag 缺失、合约代码不正确、会话层未覆盖情形等）。`380 BusinessRejectReason`：0 Other；2 Unknown Security；3 Unsupported Message Type；4 Application not available；5 Conditionally Required Field Missing。

### 2.7 业务层消息与字段映射

#### 2.7.1 报单 NewOrderSingle (35=D)
| Tag | 字段 | 说明 |
|---|---|---|
| 1 | Account | 投资者代码(Y,≤12) |
| 55 | Symbol | 合约代码(Y,≤30) |
| 11 | ClOrdID | 报单引用（整型,≤12,Y1） |
| 40 | OrdType | 1 市价 / 2 限价 / 3 市价止损 / 4 限价止损 / K 市价转限价 |
| 54 | Side | 1 买 / 2 卖 |
| 77 | OpenClose | C=平仓(上期所为平今,其他所平仓) / F=FIFO 平昨 / O=开仓；条件必填，可空（网关代算，见 2.8） |
| 44 | Price | 价格(C) |
| 38 | OrderQty | 数量(Y) |
| 59 | TimeInForce | 0 当日有效；1 撤销前有效 GTC；2 集合竞价有效 OPG；3 立即完成否则撤销 IOC；4 全部成交或撤销 FOK；6 指定日期前有效 GTD；7 本节有效 |
| 432 | ExpireDate | GTD 日期 `YYYYMMDD`（TIF=GTD 时必填） |
| 110 | MinQty | 最小成交量(C)；报立即单时只有 77 有值才有效，否则可任意数量成交 |
| 99 | StopPx | 止损价(C) |
| 20002 | IsSwapOrder | 互换单标志 N/Y |
| 207 | SecurityExchange | SHFE / CZCE / DCE / CFFEX / INE |
| 453/448/452 | NoPartyIDs/PartyID/PartyRole | PartyRole=5 投资单元代码(PartyID≤16)；=24 资金账号(PartyID≤12) |
| 15 | Currency | 币种 |
| 60 | TransactTime | 操作时间 |
| 21 | HandlInst | 1 客户自动执行；2 经纪商介入的自动执行；3 手工报单 |
| 20001 | HedgeFlag | 1 投机；2 套利；3 套保；5 做市商 |

报单唯一性：① `11-ClOrdID` + `20004-FrontID` + `20005-SessionID`；② `37-OrderID` + `207-SecurityExchange`（OrderID 由交易所给出，每家交易所一个序列，整型）。**目前只支持方法①撤改单**。`ClOrdID` 为整型，同一连接要求**升序且大于网关维护的本连接最大值**，否则拒绝。

#### 2.7.2 执行回报 ExecutionReport (35=8)
| Tag | 字段 | 说明 |
|---|---|---|
| 37 | OrderID | 空=已通过 CTP 检查、已提交交易所；非空且>0 = 已报入交易所，值为交易所报单编号；其它默认 0 |
| 39 | OrdStatus | 0 新报单/未成交；1 部分成交；2 全部成交；4 撤单；6 等待撤单；8 已拒绝；A 未知(Pending New) |
| 150 | ExecType | 0 新；4 撤单；6 等待撤单；8 拒绝；A Pending New；F 成交；I 报单状态应答 |
| 17 | ExecID | **仅** `116=61442 且 150=F` 时为成交编号，否则默认 0 |
| 14 CumQty / 151 LeavesQty | | OnRtnOrder 中：CTP/交易所拒绝 LeavesQty=0；撤单成功 LeavesQty=0；其它 LeavesQty=报单数量−成交数量；CumQty：交易所拒绝=0，撤单成功及其它=成交数量；OnRspOrderInsert/OnRspCombActionInsert 中 LeavesQty=0、CumQty=0 |
| 20004 / 20005 | FrontID / SessionID | 前置/会话 |
| 58 | Text | 错误信息 |
| 31/32 | LastPx/LastQty | 成交回报价量；`6 AvgPx`/`20 ExecTransType`/`17` 默认 0 |
| 75 / 60 / 168 | TradeDate / TransactTime / EffectiveTime | 日期/时间 |
| 453/448/452 | PartyRole 增加 3=ActiveUserID 操作用户代码 | |
| 442 | MultiLegReportingType | 1 单合约；3 组合合约 |

报单回报分类：
| 场景 | 35 | 39 | 150 |
|---|---|---|---|
| CTP 返回成功 | 8 | A | A |
| CTP/交易所拒绝 | 8 | 8 | 8 |
| 交易所返回成功 | 8 | 0 | 0 |
| 交易所成交回报 | 8 | 1 或 2 | F |
| FIX 组件拒绝 | j | — | `372-RefMsgType=D` |

#### 2.7.3 撤单 (35=F) / 改单 (35=G) / 撤改单拒绝 (35=9)
- 撤单 F：`41 OrigClOrdID`(原报单引用,Y1+Y)、`11 ClOrdID`(报单操作引用)、`20004 FrontID`、`20005 SessionID`（均 Y）、`37 OrderID`、`38 OrderQty`、`55 Symbol`(Y)、`54 Side`、`60 TransactTime`、`207`、PartyID(5=投资单元)。
- 改单 G：另有 `44 Price`、`40 OrdType`、`21 HandlInst`；`37 OrderID`(Y)。**改单只允许修改价格和数量**；CTP 的改单等于先撤单再重新报单，交易所不支持改单；改单时只返回撤单回报和重新报单的报单回报，**没有改单回报**。
- 撤单回报：CTP 成功 `35=8, 39=6, 150=6`；CTP/交易所拒绝 `35=9, 39=8`；交易所成功 `35=8, 39=4, 150=4`；FIX 组件拒绝 `35=j, 372=F`。
- `9` 撤改单拒绝字段：`Account`、`OrigClOrdID`、`ClOrdID`、`OrderID`、`434 CxlRejResponseTo`、`60`、`58 Text`、`39=8`。

#### 2.7.4 报单状态查询 (35=H)
请求字段（均可选）：`1 Account`、`55 Symbol`、`207`、`37 OrderID`。应答为 `35=8, 150=I`（报单状态），其余字段同执行回报。

#### 2.7.5 组合委托 NewOrderMultileg (35=AB)
- 仅中金所支持组合业务；国内组合只有一个组合编号，无单腿详细信息；合约格式 `A&B`（如 `IF1706&IF1709`）。
- 字段：`1 Account`、`55 Symbol`、`11 ClOrdID`(组合引用)、`54 Side`、`38 OrderQty`、`20003 CombDirection`(0 申请组合 / 1 申请拆分)、`207`、`60`、`40`、`20001 HedgeFlag`。
- 回报：CTP 成功 `35=8, 39=A, 150=A`；CTP/交易所拒绝 `35=8, 39=8`；交易所成功 `35=8, 39=0, 150=0`；`442=3`；LeavesQty：拒绝时 0，其它为报单数量；CumQty 默认 0。

#### 2.7.6 查询合约 (35=c / 35=d)
- 请求 c：`321 SecurityRequestType`（1 单个合约，此时 `55` 必填；3 一系列合约，此时 `55` 为空）、`207`、`1151 SecurityGroup`(产品代码)、`320 SecurityReqID`（填与 SenderSubID 相同的值）。
- 应答 d：`55`、`207`、`107 SecurityDesc`(合约名称)、`1151`、`167 SecurityType`(FUT/OPT/MLEG)、`200 MaturityMonthYear`(YYYYMM)、`1130 NoMarketSegments`→`1140 MaxTradeVol`/`562 MinTradeVol`（第一组=市价单，第二组=限价单）、`231 ContractMultiplier`、`969 MinPriceIncrement`、`225 IssueDate`、`873 DatedDate`、`541 MaturityDate`、`864 NoEvents`→`865 EventType`(13 开始交割日, 14 结束交割日)/`866 EventDate`、`965 SecurityStatus`(1 活跃, 2 不活跃)、`311 UnderlyingSymbol`、`202 StrikePrice`、`201 PutOrCall`、`967 StrikeMultiplier`、`762 SecuritySubType`(0 期货组合,1 BUL,2 BER,3 STD,4 STG,5 PRT,6 CLD)、`320`、`322`（均填请求编号）、`393 TotalNumSecurities`(默认1)。

#### 2.7.7 行情订阅 (35=V / 35=W)
- 请求 V：`262 MDReqID`、`263 SubscriptionRequestType`(1 订阅 / 2 退订)、`264 MarketDepth`(默认0)、`265 MDUpdateType`(默认0)、`146 NoRelatedSym`→`55 Symbol`。
- 订阅时 CTP **先返回应答消息 (35=W)，再返回行情 (35=W)**；退订检查通过返回 `35=W`；FIX 组件拒绝 `35=3, 372=V`。
- 行情 W：`75 TradeDate`、`55`、`207`、`31 LastPx`、`332 HighPx`、`333 LowPx`、`1020 TradeVolume`、`Turnover`(待定)、`1149 HighLimitPrice`、`1148 LowLimitPrice`、`811 PriceDelta`(今虚实度)、`779 LastUpdateTime`、`6 AvgPx`、`268 NoMDEntries`→`269 MDEntryType`(0 买,1 卖,4 今开盘,5 今收盘,6 本次结算价)/`270 MDEntryPx`/`271 MDEntrySize`/`1023 MDPriceLevel`。

### 2.8 开平标志为空时的网关拆分规则
当 `77-OpenClose` 为空（普通报单），网关依据**持仓（非报单）**代算：
- 存在反向持仓：平仓数量 = `min(报单数量, 反向持仓数量 − 本网关此前已报入的平仓数量)`；报单数量大于可平数量时拆为一笔平仓 + 一笔剩余数量的开仓。
- 回报拆分：昨仓 3 手空、今仓 2 手空，买入 10 手且不填开平 → 拆为 3 手平昨、2 手平今、5 手反向开仓（共 3 笔）。

示例：
| 场景 | 处理 |
|---|---|
| 无空头仓位，先买10手（未成交回报前），再卖5手 | 卖 5 手作为 5 手开仓一笔报入 |
| 持有空头10手，买15手 | min(15,10)=10 → 10 手平仓 + 5 手开仓 |
| 持有空头10手，买3手 | 3 手平仓；随后再买10手：min(10,10−3)=7 → 7 平仓 + 3 开仓；若再买7手：min(7,7)=7 → 7 手平仓一笔 |

### 2.9 FIX 常见坑（现象→原因→处理）
| # | 现象 | 原因 | 处理 |
|---|---|---|---|
| F1 | 连上后被强制登出 | 序号过低且 PossDupFlag=N | 保持序号连续；必要时登录时 `141=Y` 且 `34=1` 重置 |
| F2 | 登录/认证请求无响应 | 前一条认证/登录未收到响应期间发的请求被丢弃 | 串行发送，等待响应 |
| F3 | 开平标志为空时开平错误 | 登录后回报未收完就报单 | 等回报接收完毕再报单，或显式填 77 |
| F4 | 报单被拒 | ClOrdID 不升序/不大于最大值 | 单调递增整型 |
| F5 | 查询无响应 | 无符合条件结果时网关不回包 | 设超时；不要无限等待 |
| F6 | 被登出 text=“FIX: CTP Query Engine not ready” | 柜台查询系统未就绪 | 稍后重连 |
| F7 | 口令更新结果异常 | 口令含“：” | 避免 |
| F8 | 重复登录被拒 | 同用户多 session / 已登录再次认证登录 | 单 session |
| F9 | 无法取到 ExecID | 仅成交回报(116=61442,150=F)有成交编号 | 按此解析 |
| F10 | 心跳设置被拒 | HeartBtInt 不在 [30,60] | 取值 30~60 |
| F11 | 次日首次报单被 CTP 拒绝 | 未确认结算单 | 手工确认或申请网关自动确认 |
| F12 | 改单无改单回报 | 改单=撤单+新报单 | 以撤单回报+新报单回报处理 |

### 2.10 附录：国内合约代码规则
`YY`=两位年份、`Y`=一位年份、`MM`=两位月份、`M`=一位月份。示例：2013年6月沪深300=`IF1306`；2013年6月白砂糖=`SR306`。

| 品种 | 交易所 | 代码规则 | 品种 | 交易所 | 代码规则 |
|---|---|---|---|---|---|
| 沪深300 | 中金所 | IFYYMM | 聚乙烯 | 大商所 | lYYMM |
| 小麦 | 郑商所 | WTYMM | 聚氯乙烯 | 大商所 | vYYMM |
| 白砂糖 | 郑商所 | SRYMM | 棕榈油 | 大商所 | pYYMM |
| 棉一号 | 郑商所 | CFYMM | 黄大豆 | 大商所 | aYYMM |
| PTA | 郑商所 | TAYMM | 黄豆2 | 大商所 | bYYMM |
| 菜籽油 | 郑商所 | ROYMM | 玉米 | 大商所 | cYYMM |
| 早籼稻 | 郑商所 | ERYMM | 豆粕 | 大商所 | mYYMM |
| 强麦 | 郑商所 | WSYMM | 豆油 | 大商所 | yYYMM |
| 线材 | 上期所 | wrYYMM | 铜 | 上期所 | cuYYMM |
| 黄金 | 上期所 | auYYMM | 铝 | 上期所 | alYYMM |
| 锌 | 上期所 | znYYMM | 铅 | 上期所 | pbYYMM |
| 螺纹钢 | 上期所 | rbYYMM | 橡胶 | 上期所 | ruYYMM |
| 燃料油 | 上期所 | fuYYMM | 原油 | 能源中心 | scYYMM |

---

## 3 国际版差异
## 4 TGate 网关接入
## 5 检查清单
## 6 关键词索引

