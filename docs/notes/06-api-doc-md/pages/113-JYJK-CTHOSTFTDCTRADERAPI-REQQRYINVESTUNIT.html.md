# ReqQryInvestUnit

ReqQryInvestUnit

请求查询投资单元，暂不支持此功能

响应: [OnRspQryInvestUnit](../CTHOSTFTDCTRADERSPI/ONRSPQRYINVESTUNIT.html)

◇ 1. 函数原型

virtual int ReqQryInvestUnit(CThostFtdcQryInvestUnitField *pQryInvestUnit, int nRequestID) = 0;

◇ 2. 参数

pQryInvestUnit：查询投资单元

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 否 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |

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
