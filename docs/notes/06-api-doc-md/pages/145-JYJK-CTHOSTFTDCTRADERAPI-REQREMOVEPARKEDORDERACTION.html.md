# ReqRemoveParkedOrderAction

ReqRemoveParkedOrderAction

请求删除预埋撤单，对应响应[OnRspRemoveParkedOrderAction](../CTHOSTFTDCTRADERSPI/ONRSPREMOVEPARKEDORDERACTION.html)。该函数用于删除已经报入但未触发的某笔预埋撤单，注意是预埋撤单，而不是预埋报单。删除预埋报单使用[ReqRemoveParkedOrder](REQREMOVEPARKEDORDER.html)。

◇ 1. 函数原型

virtual int ReqRemoveParkedOrderAction(CThostFtdcRemoveParkedOrderActionField *pRemoveParkedOrderAction, int nRequestID) = 0;

◇ 2. 参数

pRemoveParkedOrderAction：删除预埋撤单

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcParkedOrderActionIDType | ParkedOrderActionID | 预埋撤单单编号 | 必填 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |

ParkedOrderActionID：对应的要删除的预埋撤单编号

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcRemoveParkedOrderActionField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.ParkedOrderActionID, "1");
m_pUserApi->ReqRemoveParkedOrderAction(&a, nRequestID++);

```

◇ 5. FAQ

无
