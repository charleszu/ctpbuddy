# ReqQrySettlementInfo

ReqQrySettlementInfo

请求查询投资者结算结果，对应响应[OnRspQrySettlementInfo](../CTHOSTFTDCTRADERSPI/ONRSPQRYSETTLEMENTINFO.html)。可以查询当天或历史结算单，也可以查询月结算单，但是前提是CTP柜台生成了相应的日或月结算单。

◇ 1. 函数原型

virtual int ReqQrySettlementInfo(CThostFtdcQrySettlementInfoField *pQrySettlementInfo, int nRequestID) = 0;

◇ 2. 参数

pQrySettlementInfo：查询投资者结算结果

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcDateType | TradingDay | 交易日 | 是 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 否 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 否 |

TradingDay：查询某一天的结算单，填写格式为“yyyymmdd”；查询某一月的结算单，填写格式为“yyyymm”

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQrySettlementInfoField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.TradingDay,“20180101”);
m_pUserApi->ReqQrySettlementInfo(&a, nRequestID++);

```

◇ 5. FAQ

为什么我查不到月结单？

| 这有可能是CTP系统里没有生成你的月结算单。CTP的日结算单是需要每天结算的时候业务人员去点击生成的，月结算单也是需要定期生成的。 |
|---|
