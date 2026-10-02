# ReqQryRCAMSShortOptAdjustParam

ReqQryRCAMSShortOptAdjustParam

请求RCAMS空头期权风险调整参数查询，对应响应请求[OnRspQryRCAMSShortOptAdjustParam](../CTHOSTFTDCTRADERSPI/ONRSPQRYRCAMSSHORTOPTADJUSTPARAM.html)

◇ 1. 函数原型

virtual int ReqQryRCAMSShortOptAdjustParam(CThostFtdcQryRCAMSShortOptAdjustParamField *pQryRCAMSShortOptAdjustParam, int nRequestID) = 0;

◇ 2. 参数

pQryRCAMSShortOptAdjustParam：RCAMS空头期权风险调整参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcProductIDType | CombProductID | 产品组合代码 | 是 |

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
