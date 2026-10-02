# ReqQryOptionInstrTradeCost

ReqQryOptionInstrTradeCost

请求查询期权交易成本，该函数用于查期权保证金，对应响应[OnRspQryOptionInstrTradeCost](../CTHOSTFTDCTRADERSPI/ONRSPQRYOPTIONINSTRTRADECOST.html)。

保证金=max(权利金+FixedMargin,MiniMargin)，用户可根据此公式计算实时保证金。

◇ 1. 函数原型

virtual int ReqQryOptionInstrTradeCost(CThostFtdcQryOptionInstrTradeCostField *pQryOptionInstrTradeCost, int nRequestID) = 0;

◇ 2. 参数

pQryOptionInstrTradeCost：期权交易成本查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcHedgeFlagType | HedgeFlag | 投机套保标志 | 是 |
| TThostFtdcPriceType | InputPrice | 期权合约报价 | 是 |
| TThostFtdcPriceType | UnderlyingPrice | 标的价格,填0则用昨结算价 | 是 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryOptionInstrTradeCostField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "ad2503C13800");
a.HedgeFlag = THOST_FTDC_HF_Speculation;
a.InputPrice = 300;
m_pUserApi->ReqQryOptionInstrTradeCost(&a, nRequestID++);

```

◇ 5. FAQ

为什么我用这个接口算出来的保证金跟资金查询得到的保证金占用不一致？

| ReqQryOptionInstrTradeCost计算出的期权保证金跟资金查询里的期权保证金的计算方式不一样。
ReqQryOptionInstrTradeCost只是估计计算，因为其使用的公式（保证金=max(权利金+FixedMargin,MiniMargin)）中的权利金部分在计算时使用的期权价格是InputPrice。
而资金查询里的期权保证金计算公式中的期权价格是使用max算法（max(昨结算，最新价)）得到的。
所以两者存在差别。 |
|---|
