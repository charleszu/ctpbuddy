# ReqQryCombLeg

ReqQryCombLeg

组合腿信息查询，对应响应请求[OnRspQryCombLeg](../CTHOSTFTDCTRADERSPI/ONRSPQRYCOMBLEG.html)

该接口支持查询可申请组合合约的信息和套利合约信息，查询条件中单腿合约代码**（必填）**。

注：业务上只有大商所、广期所有可组合的合约（中金所的RCAMS也有，但目前未上线）。该接口若填写‘Z’（郑商所）时，也可返回郑商所的组合合约，业务上不建议这么用。

◇ 1. 函数原型

virtual int ReqQryCombLeg(CThostFtdcQryCombLegField *pQryCombLeg, int nRequestID) = 0;

◇ 2. 参数

pQryCombLeg：组合腿信息查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | LegInstrumentID | 单腿合约代码 | 是 |

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
