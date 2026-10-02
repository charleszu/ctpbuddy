# ReqOptionSelfCloseInsert

ReqOptionSelfCloseInsert

期权自对冲录入请求、详见[期货期权的行权、自对冲](../../QTYWGZ/QHQQDHQ-ZDCGZ.html)

错误响应: [OnErrRtnOptionSelfCloseInsert](../CTHOSTFTDCTRADERSPI/ONERRRTNOPTIONSELFCLOSEINSERT.html)，[OnRspOptionSelfCloseInsert](../CTHOSTFTDCTRADERSPI/ONRSPOPTIONSELFCLOSEINSERT.html)

正确响应: [OnRtnOptionSelfClose](../CTHOSTFTDCTRADERSPI/ONRTNOPTIONSELFCLOSE.html)

关于接口中的重要序号说明详见[接口中一些重要序号说明](../../QTYWGZ/JKZYXZYXHSM.html)

关于大商所行权二阶段业务详见[大商所行权优化二阶段业务](../../QTYWGZ/DSSHQYHEJDYW.html)

此为业务设置接口，调用后若收到交易所正确回报，即表示业务请求被接收。

若对前次已接收的业务设置进行修改，再次发起自对冲请求（相同合约，相同开平标志），则回报中的OptionSelfCloseRef，OptionSelfCloseLocalID字段不会更新，OptionSelfCloseSysID由交易所更新。

◇ 1. 函数原型

virtual int ReqOptionSelfCloseInsert(CThostFtdcInputOptionSelfCloseField *pInputOptionSelfClose, int nRequestID) = 0;

◇ 2. 参数

pInputOptionSelfClose：输入的期权自对冲

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填 |
| TThostFtdcOrderRefType | OptionSelfCloseRef | 期权自对冲引用 | 无 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcBusinessUnitType | BusinessUnit | 业务单元 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 无 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 无 |
| TThostFtdcClientIDType | ClientID | 客户代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcVolumeType | Volume | 数量 | 必填 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcHedgeFlagType | HedgeFlag | 投机套保标志 | 必填 |
| TThostFtdcOptSelfCloseFlagType | OptSelfCloseFlag | 期权行权的头寸是否自对冲 | 1自对冲期权仓位、2保留期权仓位、3自对冲卖方履约后的期货仓位 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 否 |

OptionSelfCloseRef：需要纯数字递增，不填则ctp自动填写

IPAddress：中继需填写客户IP地址；非中继填写无效，直接取登录成功会话中的IP。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

MacAddress：中继需填写客户MAC地址；非中继填写无效，直接取登录成功会话中的MAC。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcInputOptionSelfCloseField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
strcpy_s(a.UserID, "1000001");
a.Volume = 1;
a.HedgeFlag = THOST_FTDC_HF_Speculation;
a.OptSelfCloseFlag = THOST_FTDC_OSCF_CloseSelfOptionPosition;
strcpy_s(a.ExchangeID, "SHFE");
strcpy_s(a.AccountID, "1000001");
strcpy_s(a.CurrencyID, "CNY");
m_pUserApi->ReqOptionSelfCloseInsert(&a, nRequestID++);

```

◇ 5. FAQ

为什么报入的OptionSelfCloseRef和返回的值不同？

| 根据交易所规则，自对冲报单由首次报入的报单属性确定，之后不论是否对该报单进行任何操作（包括撤单和重新报单），报单的OptionSelfCloseRef都不变。 |
|---|
