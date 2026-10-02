# ReqForQuoteInsert

ReqForQuoteInsert

询价录入请求

错误响应: [OnErrRtnForQuoteInsert](../CTHOSTFTDCTRADERSPI/ONERRRTNFORQUOTEINSERT.html)，[OnRspForQuoteInsert](../CTHOSTFTDCTRADERSPI/ONRSPFORQUOTEINSERT.html)

正确响应: [OnRtnForQuoteRsp](../CTHOSTFTDCTRADERSPI/ONRTNFORQUOTERSP.html)

详见[做市商询价和报价](../../QTYWGZ/BJHXJ.html)

关于接口中的重要序号说明详见[接口中一些重要序号说明](../../QTYWGZ/JKZYXZYXHSM.html)

◇ 1. 函数原型

virtual int ReqForQuoteInsert(CThostFtdcInputForQuoteField *pInputForQuote, int nRequestID) = 0;

◇ 2. 参数

pInputForQuote：输入的询价

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 期权合约名称 |
| TThostFtdcOrderRefType | ForQuoteRef | 询价引用 | 选填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 无 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

ForQuoteRef：需要纯数字递增，不填则ctp自动填写

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
CThostFtdcInputForQuoteField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
strcpy_s(a.UserID, "1000001");
strcpy_s(a.ExchangeID, "SHFE");
m_pUserApi->ReqForQuoteInsert(&a, nRequestID++);

```

◇ 5. FAQ

询价时报：“没有该合约的做市商”？

| 询价合约不对。 |
|---|
