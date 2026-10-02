# ReqRemoveParkedOrderAction

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqRemoveParkedOrderAction<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求删除预埋撤单，对应响应[OnRspRemoveParkedOrderAction](pages/295-JYJK-CTHOSTFTDCTRADERSPI-ONRSPREMOVEPARKEDORDERACTION.html.md)。该函数用于删除已经报入但未触发的某笔预埋撤单，注意是预埋撤单，而不是预埋报单。删除预埋报单使用[ReqRemoveParkedOrder](pages/144-JYJK-CTHOSTFTDCTRADERAPI-REQREMOVEPARKEDORDER.html.md)。
<a id="4150f110-abb5-4b15-ab20-d46647e1cf48"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqRemoveParkedOrderAction(CThostFtdcRemoveParkedOrderActionField *pRemoveParkedOrderAction, int nRequestID) = 0;

<a id="558ea525-a102-4c2b-8b55-8101611a4743"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRemoveParkedOrderAction：删除预埋撤单

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcParkedOrderActionIDType | ParkedOrderActionID | 预埋撤单单编号 | 必填 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |

ParkedOrderActionID：对应的要删除的预埋撤单编号

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="394f3994-7797-47ca-9c5a-d813f9df309c"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="74886fe3-e72d-478c-b4e8-c0b201983910"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcRemoveParkedOrderActionField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.ParkedOrderActionID, "1");
m_pUserApi->ReqRemoveParkedOrderAction(&a, nRequestID++);

```

<a id="fbada294-61ba-4b77-bdf2-41664ba7f1bc"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
