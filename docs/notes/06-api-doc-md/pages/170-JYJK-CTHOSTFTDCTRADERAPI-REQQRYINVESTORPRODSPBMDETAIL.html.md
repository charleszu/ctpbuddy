# ReqQryInvestorProdSPBMDetail

ReqQryInvestorProdSPBMDetail

请求投资者产品SPBM明细查询，对应响应请求[OnRspQryInvestorProdSPBMDetail](../CTHOSTFTDCTRADERSPI/ONRSPQRYINVESTORPRODSPBMDETAIL.html)

◇ 1. 函数原型

virtual int ReqQryInvestorProdSPBMDetail(CThostFtdcQryInvestorProdSPBMDetailField *pQryInvestorProdSPBMDetail, int nRequestID) = 0;

◇ 2. 参数

pQryInvestorProdSPBMDetail：投资者产品SPBM明细查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | ProdFamilyCode | 品种代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

无

◇ 5. FAQ

无
