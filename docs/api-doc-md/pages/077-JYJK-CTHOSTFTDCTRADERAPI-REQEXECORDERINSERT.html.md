# ReqExecOrderInsert

ReqExecOrderInsert

执行宣告录入请求、详见[期货期权的行权、自对冲](../../QTYWGZ/QHQQDHQ-ZDCGZ.html)

关于接口中的重要序号说明详见[接口中一些重要序号说明](../../QTYWGZ/JKZYXZYXHSM.html)

关于大商所行权二阶段业务详见[大商所行权优化二阶段业务](../../QTYWGZ/DSSHQYHEJDYW.html)

错误响应: [OnErrRtnExecOrderInsert](../CTHOSTFTDCTRADERSPI/ONERRRTNEXECORDERINSERT.html)，[OnRspExecOrderInsert](../CTHOSTFTDCTRADERSPI/ONRSPEXECORDERINSERT.html)

正确响应: [OnRtnExecOrder](../CTHOSTFTDCTRADERSPI/ONRTNEXECORDER.html)

**注：

大商所期权行权接口，不能再报行权后自对冲申请了，即CloseFlag只能填EOCF_NotToClose('1')，否则会报错；

需要实现大商所期权放弃申请，使用此接口。

兼容支持大商所“取消到期自动行权接口”，大商所行权二阶段业务上线前支持申报大商所“取消到期自动行权”接口（按照原有实现方式，调用行权接口，手数为0手实现）；大商所行权二阶段业务上线后支持申报“期权放弃申请”使用此接口**

◇ 1. 函数原型

virtual int ReqExecOrderInsert(CThostFtdcInputExecOrderField *pInputExecOrder, int nRequestID) = 0;

◇ 2. 参数

pInputExecOrder：输入的执行宣告

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填 |
| TThostFtdcOrderRefType | ExecOrderRef | 执行宣告引用 | 选填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcBusinessUnitType | BusinessUnit | 业务单元 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 无 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 无 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 无 |
| TThostFtdcClientIDType | ClientID | 客户代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcVolumeType | Volume | 数量 | 必填 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcOffsetFlagType | OffsetFlag | 开平标志 | 必填 |
| TThostFtdcHedgeFlagType | HedgeFlag | 投机套保标志 | 投机或套保 |
| TThostFtdcActionTypeType | ActionType | 执行类型 | 必填 |
| TThostFtdcPosiDirectionType | PosiDirection | 保留头寸申请的持仓方向 | 多头 |
| TThostFtdcExecOrderPositionFlagType | ReservePositionFlag | 期权行权后是否保留期货头寸的标记,该字段已废弃 | 该字段已废弃，调用时不能为空 |
| TThostFtdcExecOrderCloseFlagType | CloseFlag | 期权行权后生成的头寸是否自动平仓 | 必填 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 否 |

ExecOrderRef：需要纯数字递增，不填则ctp自动填写

IPAddress：中继需填写客户IP地址；非中继填写无效，直接取登录成功会话中的IP。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

MacAddress：中继需填写客户MAC地址；非中继填写无效，直接取登录成功会话中的MAC。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

大商所放弃行权示例

```
CThostFtdcInputExecOrderField OrderInsert = { 0 };
strcpy_s(OrderInsert.BrokerID, "9999");
strcpy_s(OrderInsert.InvestorID, "1000001");
strcpy_s(OrderInsert.InstrumentID, "a2505-C-3950");
strcpy_s(OrderInsert.ExchangeID, "DCE");
strcpy_s(OrderInsert.ExecOrderRef, "00001");
strcpy_s(OrderInsert.UserID, "1000001");
OrderInsert.Volume = 1;
OrderInsert.RequestID = 1;
OrderInsert.OffsetFlag = THOST_FTDC_OF_Close;//开平标志(平仓)
OrderInsert.HedgeFlag = THOST_FTDC_HF_Speculation;//投机套保标志(投机)
OrderInsert.ActionType = THOST_FTDC_ACTP_Abandon;//执行类型类型(取消行权)
OrderInsert.PosiDirection = THOST_FTDC_PD_Long;//持仓多空方向类型(多头)
OrderInsert.ReservePositionFlag = THOST_FTDC_EOPF_Reserve;//期权行权后是否保留期货头寸的标记类型(保留头寸)
OrderInsert.CloseFlag = THOST_FTDC_EOCF_NotToClose;//期权行权后生成的头寸是否自动平仓类型(免于自动平仓)
m_pUserApi->ReqExecOrderInsert(&OrderInsert, nRequestID++);

```

◇ 5. FAQ

盘中通过api进行中金所行权，报错“CTP:不支持的功能”为什么？

| 中金所不支持通过api行权，只能盘后通过会服提交行权申请。 |
|---|

盘中发出大商所的“取消到期日自动行权”指令后(Volume字段填0)后，为什么收到的响应字段和请求字段有所区别？

| 多次发出相同合约的该指令，CTP会将第一次请求的响应结果返回给api端，即每次请求收到的响应都是第一次请求的结果。 |
|---|

各家交易所的行权指令有什么不同？

| 中金所不支持api发起行权
郑商所closeflag必须为nottoclose
上期、能源、大商所不限制 |
|---|
