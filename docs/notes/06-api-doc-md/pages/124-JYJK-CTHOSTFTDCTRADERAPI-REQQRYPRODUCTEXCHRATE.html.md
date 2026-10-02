# ReqQryProductExchRate

ReqQryProductExchRate

请求查询产品报价汇率

响应: [OnRspQryProductExchRate](../CTHOSTFTDCTRADERSPI/ONRSPQRYPRODUCTEXCHRATE.html)

◇ 1. 函数原型

virtual int ReqQryProductExchRate(CThostFtdcQryProductExchRateField *pQryProductExchRate, int nRequestID) = 0;

◇ 2. 参数

pQryProductExchRate：产品报价汇率查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | ProductID | 产品代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
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

无
