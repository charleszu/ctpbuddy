# ReqQryTraderOffer

ReqQryTraderOffer

请求查询交易员报盘机，对应响应请求[OnRspQryTraderOffer](../CTHOSTFTDCTRADERSPI/ONRSPQRYTRADEROFFER.html)

◇ 1. 函数原型

virtual int ReqQryTraderOffer(CThostFtdcQryTraderOfferField *pQryTraderOffer, int nRequestID) = 0;

◇ 2. 参数

pQryTraderOffer：查询交易员报盘机

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcParticipantIDType | ParticipantID | 会员代码 | 否 |
| TThostFtdcTraderIDType | TraderID | 交易所交易员代码 | 否 |

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
