# ReqOptionSelfCloseAction

ReqOptionSelfCloseAction

期权自对冲操作请求、详见[期货期权的行权、自对冲](../../QTYWGZ/QHQQDHQ-ZDCGZ.html)

错误响应: [OnErrRtnOptionSelfCloseAction](../CTHOSTFTDCTRADERSPI/ONERRRTNOPTIONSELFCLOSEACTION.html)，[OnRspOptionSelfCloseAction](../CTHOSTFTDCTRADERSPI/ONRSPOPTIONSELFCLOSEACTION.html)

正确响应: [OnRtnOptionSelfClose](../CTHOSTFTDCTRADERSPI/ONRTNOPTIONSELFCLOSE.html)

关于接口中的重要序号说明详见[接口中一些重要序号说明](../../QTYWGZ/JKZYXZYXHSM.html)

◇ 1. 函数原型

virtual int ReqOptionSelfCloseAction(CThostFtdcInputOptionSelfCloseActionField *pInputOptionSelfCloseAction, int nRequestID) = 0;

◇ 2. 参数

pInputOptionSelfCloseAction：输入期权自对冲操作

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcOrderRefType | OptionSelfCloseRef | 期权自对冲引用 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填 |
| TThostFtdcOrderSysIDType | OptionSelfCloseSysID | 期权自对冲编号 | 必填*1，需要对应要撤的报单 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 无 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcOrderActionRefType | OptionSelfCloseActionRef | 期权自对冲操作引用 | 无 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcFrontIDType | FrontID | 前置编号 | 无 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 无 |
| TThostFtdcActionFlagType | ActionFlag | 操作标志 | 必填 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 否 |

OptionSelfCloseRef：对应要撤销的期权自对冲的引用

FrontID：对应要撤销的期权自对冲的前置编号

SessionID：对应要撤销的期权自对冲的会话编号

ExchangeID：对应要撤销的期权自对冲的交易所编号

ActionFlag：支持删除，不支持修改

InstrumentID：对应要撤销的期权自对冲的合约代码

IPAddress：手工填写本机IP地址，不自动获取。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcInputOptionSelfCloseActionField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.OptionSelfCloseRef, "000000258");//期权自对冲引用
a.FrontID = 1;
a.SessionID = 6442531;
strcpy_s(a.ExchangeID, "SHFE");
a.ActionFlag = THOST_FTDC_AF_Delete;
strcpy_s(a.UserID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
m_pUserApi->ReqOptionSelfCloseAction(&a, nRequestID++);

```

◇ 5. FAQ

无
