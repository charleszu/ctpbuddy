# ReqQryCFMMCTradingAccountKey

ReqQryCFMMCTradingAccountKey

请求查询保证金监管系统经纪公司资金账户密钥，**此接口已弃用**，请使用[ReqQueryCFMMCTradingAccountToken](REQQUERYCFMMCTRADINGACCOUNTTOKEN.html)查询。

响应: [OnRspQryCFMMCTradingAccountKey](../CTHOSTFTDCTRADERSPI/ONRSPQRYCFMMCTRADINGACCOUNTKEY.html)

◇ 1. 函数原型

virtual int ReqQryCFMMCTradingAccountKey(CThostFtdcQryCFMMCTradingAccountKeyField *pQryCFMMCTradingAccountKey, int nRequestID) = 0;

◇ 2. 参数

pQryCFMMCTradingAccountKey：请求查询保证金监管系统经纪公司资金账户密钥

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 否 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 否 |

接口弃用

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryCFMMCTradingAccountKeyField a = { 0 };
strcpy(a.BrokerID, "9999");
strcpy(a.InvestorID, "1000001");
m_pUserApi->ReqQryCFMMCTradingAccountKey(&a, nRequestID++);

```

◇ 5. FAQ

无
