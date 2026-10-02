# ReqQrySPBMInterParameter

ReqQrySPBMInterParameter

请求SPBM跨品种抵扣参数查询，对应响应请求[OnRspQrySPBMInterParameter](../CTHOSTFTDCTRADERSPI/ONRSPQRYSPBMINTERPARAMETER.html)

◇ 1. 函数原型

virtual int ReqQrySPBMInterParameter(CThostFtdcQrySPBMInterParameterField *pQrySPBMInterParameter, int nRequestID) = 0;

◇ 2. 参数

pQrySPBMInterParameter：SPBM跨品种抵扣参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcInstrumentIDType | Leg1ProdFamilyCode | 第一腿构成品种 | 是 |
| TThostFtdcInstrumentIDType | Leg2ProdFamilyCode | 第二腿构成品种 | 是 |

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
