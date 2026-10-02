# ReqQryInvestorPositionCombineDetail

ReqQryInvestorPositionCombineDetail

请求查询投资者持仓明细

响应: [OnRspQryInvestorPositionCombineDetail](../CTHOSTFTDCTRADERSPI/ONRSPQRYINVESTORPOSITIONCOMBINEDETAIL.html)

◇ 1. 函数原型

virtual int ReqQryInvestorPositionCombineDetail(CThostFtdcQryInvestorPositionCombineDetailField *pQryInvestorPositionCombineDetail, int nRequestID) = 0;

◇ 2. 参数

pQryInvestorPositionCombineDetail：查询组合持仓明细

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | CombInstrumentID | 组合持仓合约编码 | 是 |
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

无

◇ 5. FAQ

大商所下套利单成交后查询接口没有找到持仓，是什么原因呢？

| 因为柜台开启了大商所rule新型组合保证金算法，就没有套利持仓了。 |
|---|
