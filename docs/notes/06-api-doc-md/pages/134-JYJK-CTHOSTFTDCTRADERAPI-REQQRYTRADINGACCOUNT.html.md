# ReqQryTradingAccount

ReqQryTradingAccount

请求查询资金账户，

响应: [OnRspQryTradingAccount](../CTHOSTFTDCTRADERSPI/ONRSPQRYTRADINGACCOUNT.html)

◇ 1. 函数原型

virtual int ReqQryTradingAccount(CThostFtdcQryTradingAccountField *pQryTradingAccount, int nRequestID) = 0;

◇ 2. 参数

pQryTradingAccount：查询资金账户

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 是 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 否 |
| TThostFtdcBizTypeType | BizType | 业务类型 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryTradingAccountField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.CurrencyID, "CNY");
m_pUserApi->ReqQryTradingAccount(&a, nRequestID++);

```

◇ 5. FAQ

无
