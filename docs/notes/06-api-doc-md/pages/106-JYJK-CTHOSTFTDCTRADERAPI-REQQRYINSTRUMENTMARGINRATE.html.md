# ReqQryInstrumentMarginRate

ReqQryInstrumentMarginRate

请求查询合约保证金率，对应响应[OnRspQryInstrumentMarginRate](../CTHOSTFTDCTRADERSPI/ONRSPQRYINSTRUMENTMARGINRATE.html)。如果InstrumentID填空，则返回持仓对应的合约保证金率，否则返回相应InstrumentID的保证金率。

目前无法通过一次查询得到所有合约保证金率，如果要查询所有，则需要通过多次查询得到。

◇ 1. 函数原型

virtual int ReqQryInstrumentMarginRate(CThostFtdcQryInstrumentMarginRateField *pQryInstrumentMarginRate, int nRequestID) = 0;

◇ 2. 参数

pQryInstrumentMarginRate：查询合约保证金率

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcHedgeFlagType | HedgeFlag | 投机套保标志 | 是 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryInstrumentMarginRateField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
a.HedgeFlag = THOST_FTDC_HF_Speculation;
m_pUserApi->ReqQryInstrumentMarginRate(&a, 1);

```

◇ 5. FAQ

无
