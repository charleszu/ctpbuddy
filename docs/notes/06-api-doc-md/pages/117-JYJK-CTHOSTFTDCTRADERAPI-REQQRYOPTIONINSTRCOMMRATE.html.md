# ReqQryOptionInstrCommRate

ReqQryOptionInstrCommRate

请求查询期权合约手续费

响应: [OnRspQryOptionInstrCommRate](../CTHOSTFTDCTRADERSPI/ONRSPQRYOPTIONINSTRCOMMRATE.html)

◇ 1. 函数原型

virtual int ReqQryOptionInstrCommRate(CThostFtdcQryOptionInstrCommRateField *pQryOptionInstrCommRate, int nRequestID) = 0;

◇ 2. 参数

pQryOptionInstrCommRate：期权手续费率查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryOptionInstrCommRateField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "ag2311C1200");
m_pUserApi->ReqQryOptionInstrCommRate(&a, nRequestID++);

```

◇ 5. FAQ

合约填空，是否能查询所有期权合约手续费率？

| 不能，填空只能返回持仓的期权合约手续费率。查询所有期权合约需要一个个请求查询。 |
|---|
