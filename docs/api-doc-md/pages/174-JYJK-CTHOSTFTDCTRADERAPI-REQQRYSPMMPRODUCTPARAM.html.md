# ReqQrySPMMProductParam

ReqQrySPMMProductParam

请求SPMM产品参数查询，对应响应请求[OnRspQrySPMMProductParam](../CTHOSTFTDCTRADERSPI/ONRSPQRYSPMMPRODUCTPARAM.html)

◇ 1. 函数原型

virtual int ReqQrySPMMProductParam(CThostFtdcQrySPMMProductParamField *pQrySPMMProductParam, int nRequestID) = 0;

◇ 2. 参数

pQrySPMMProductParam：SPMM产品参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcSPMMProductIDType | ProductID | 产品代码 | 是 |

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
