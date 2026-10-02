# ReqQrySecAgentTradeInfo

ReqQrySecAgentTradeInfo

请求查询二级代理商信息

响应: [OnRspQrySecAgentTradeInfo](../CTHOSTFTDCTRADERSPI/ONRSPQRYSECAGENTTRADEINFO.html)

◇ 1. 函数原型

virtual int ReqQrySecAgentTradeInfo(CThostFtdcQrySecAgentTradeInfoField *pQrySecAgentTradeInfo, int nRequestID) = 0;

◇ 2. 参数

pQrySecAgentTradeInfo：查询二级代理商信息

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcAccountIDType | BrokerSecAgentID | 境外中介机构资金帐号 | 是 |

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
