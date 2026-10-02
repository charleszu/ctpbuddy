# ReqRemoveParkedOrder

ReqRemoveParkedOrder

请求删除预埋单，对应响应[OnRspRemoveParkedOrder](../CTHOSTFTDCTRADERSPI/ONRSPREMOVEPARKEDORDER.html)。该函数用于删除已经报入但未触发的某笔预埋报单，注意跟[ReqRemoveParkedOrderAction](REQREMOVEPARKEDORDERACTION.html)的区别。

◇ 1. 函数原型

virtual int ReqRemoveParkedOrder(CThostFtdcRemoveParkedOrderField *pRemoveParkedOrder, int nRequestID) = 0;

◇ 2. 参数

pRemoveParkedOrder：删除预埋单

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcParkedOrderIDType | ParkedOrderID | 预埋报单编号 | 必填 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |

ParkedOrderID：对应的预埋报单编号，指定要删除的预埋报单。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4.示例调用

```
CThostFtdcRemoveParkedOrderField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.ParkedOrderID, "           5");
m_pUserApi->ReqRemoveParkedOrder(&a, nRequestID++);

```

◇ 5. FAQ

无
