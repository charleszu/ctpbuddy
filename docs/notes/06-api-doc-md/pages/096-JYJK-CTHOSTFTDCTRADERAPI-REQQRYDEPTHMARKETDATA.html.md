# ReqQryDepthMarketData

ReqQryDepthMarketData

请求查询行情，只能查询当前快照，不能查询历史行情。

响应: [OnRspQryDepthMarketData](../CTHOSTFTDCTRADERSPI/ONRSPQRYDEPTHMARKETDATA.html)

◇ 1. 函数原型

virtual int ReqQryDepthMarketData(CThostFtdcQryDepthMarketDataField *pQryDepthMarketData, int nRequestID) = 0;

◇ 2. 参数

pQryDepthMarketData：查询行情

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcProductClassType | ProductClass | 产品类型 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

查询某一产品的所有期货、期权合约行情，入参 InstrumentD 填入产品代码即可

查询某一大类的所有合约行情，比如返回所有期货类合约行情，入参ProductClass填入1即可。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

无

◇ 5. FAQ

无
