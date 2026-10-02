# ReqQrySecAgentCheckMode

ReqQrySecAgentCheckMode

请求查询二级代理商资金校验模式

响应: [OnRspQrySecAgentCheckMode](../CTHOSTFTDCTRADERSPI/ONRSPQRYSECAGENTCHECKMODE.html)

◇ 1. 函数原型

virtual int ReqQrySecAgentCheckMode(CThostFtdcQrySecAgentCheckModeField *pQrySecAgentCheckMode, int nRequestID) = 0;

◇ 2. 参数

pQrySecAgentCheckMode：查询二级代理商资金校验模式

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |

使用二级代理商操作员管理的投资者进行查询，字段值填写正确时，有查询结果。

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
