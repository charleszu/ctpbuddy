# ReqRemoveParkedOrder

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqRemoveParkedOrder<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求删除预埋单，对应响应[OnRspRemoveParkedOrder](pages/294-JYJK-CTHOSTFTDCTRADERSPI-ONRSPREMOVEPARKEDORDER.html.md)。该函数用于删除已经报入但未触发的某笔预埋报单，注意跟[ReqRemoveParkedOrderAction](pages/145-JYJK-CTHOSTFTDCTRADERAPI-REQREMOVEPARKEDORDERACTION.html.md)的区别。
<a id="d2b4d729-e341-4164-b607-b765c44a51e1"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqRemoveParkedOrder(CThostFtdcRemoveParkedOrderField *pRemoveParkedOrder, int nRequestID) = 0;

<a id="60b43a5e-e54f-4b82-97a3-51907f8c028a"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRemoveParkedOrder：删除预埋单

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcParkedOrderIDType | ParkedOrderID | 预埋报单编号 | 必填 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |

ParkedOrderID：对应的预埋报单编号，指定要删除的预埋报单。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="d6de7ca9-e990-4c18-a493-a163801aff72"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="5dc1cc48-ee4f-4767-a2f5-47957998ede0"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4.示例调用
<a id="panel4"></a>

```
CThostFtdcRemoveParkedOrderField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.ParkedOrderID, "           5");
m_pUserApi->ReqRemoveParkedOrder(&a, nRequestID++);

```

<a id="833dcc47-69e3-47df-9c89-37d32644cd99"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
