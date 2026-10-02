# ReqSpdApplyAction

ReqSpdApplyAction

套利确认撤销请求

若CTP校验通过后返回[OnRtnSpdApply](../CTHOSTFTDCTRADERSPI/ONRTNSPDAPPLY.html)撤单状态通知

如果CTP校验未通过返回[OnRspSpdApplyAction](../CTHOSTFTDCTRADERSPI/ONRSPSPDAPPLYACTION.html)响应和[OnErrRtnSpdApplyAction](../CTHOSTFTDCTRADERSPI/ONERRRTNSPDAPPLYACTION.html)错误回报

从交易所回来收到回报后给[OnRtnSpdApply](../CTHOSTFTDCTRADERSPI/ONRTNSPDAPPLY.html)撤单状态通知（若交易所校验未通过会有[OnErrRtnSpdApplyAction](../CTHOSTFTDCTRADERSPI/ONERRRTNSPDAPPLYACTION.html)的）。

◇ 1. 函数原型

virtual int ReqSpdApplyAction(CThostFtdcInputSpdApplyActionField *pInputSpdApplyAction, int nRequestID) = 0;

◇ 2. 参数

pInputSpdApplyAction：套利申请撤销

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填*2 |
| TThostFtdcOrderSysIDType | OrderSysID | 合同编号 | 必填*2 |
| TThostFtdcOrderRefType | OrderRef | 报单引用 | 必填*1 |
| TThostFtdcFrontIDType | FrontID | 前置编号 | 必填*1 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 必填*1 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 否 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 否 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 否 |

**必填*1、必填*2**：两组选一组必填，能对应要撤的报单。

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
