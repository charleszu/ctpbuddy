# ReqQryCombPromotionParam

ReqQryCombPromotionParam

请求组合优惠比例，对应响应请求[OnRspQryCombPromotionParam](../CTHOSTFTDCTRADERSPI/ONRSPQRYCOMBPROMOTIONPARAM.html)

暂时只支持期权x参数查询，不支持期货。即大商所2025年12月31日推出的期货对锁、跨期、跨品种组合的新优惠中的x参数暂时不支持查询。

详见  [6.5.1版本更新说明补充说明](../../6.5.1BBGXSMBCSM.html)

◇ 1. 函数原型

virtual int ReqQryCombPromotionParam(CThostFtdcQryCombPromotionParamField *pQryCombPromotionParam, int nRequestID) = 0;

◇ 2. 参数

pQryCombPromotionParam：查询组合优惠比例

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 否 |

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
