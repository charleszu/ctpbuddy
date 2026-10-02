# ReqQryClassifiedInstrument

ReqQryClassifiedInstrument

请求查询分类合约，对应响应请求[OnRspQryClassifiedInstrument](../CTHOSTFTDCTRADERSPI/ONRSPQRYCLASSIFIEDINSTRUMENT.html)

详见  [6.5.1版本更新说明补充说明](../../6.5.1BBGXSMBCSM.html)

◇ 1. 函数原型

virtual int ReqQryClassifiedInstrument(CThostFtdcQryClassifiedInstrumentField *pQryClassifiedInstrument, int nRequestID) = 0;

◇ 2. 参数

pQryClassifiedInstrument：查询分类合约

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 否 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcExchangeInstIDType | ExchangeInstID | 合约在交易所的代码 | 否 |
| TThostFtdcInstrumentIDType | ProductID | 产品代码 | 否 |
| TThostFtdcTradingTypeType | TradingType | 合约交易状态 | 是 |
| TThostFtdcClassTypeType | ClassType | 合约分类类型 | 是 |

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
