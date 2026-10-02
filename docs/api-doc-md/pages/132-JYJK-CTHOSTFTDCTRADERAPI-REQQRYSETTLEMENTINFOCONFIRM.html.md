# ReqQrySettlementInfoConfirm

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySettlementInfoConfirm<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询结算信息确认

响应: [OnRspQrySettlementInfoConfirm](pages/283-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSETTLEMENTINFOCONFIRM.html.md)
<a id="8f483b5c-ee7f-4d62-9202-0c8a4207f1be"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySettlementInfoConfirm(CThostFtdcQrySettlementInfoConfirmField *pQrySettlementInfoConfirm, int nRequestID) = 0;

<a id="3991f31c-c9d7-46f1-85f9-cf5b9ebfe7c9"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySettlementInfoConfirm：查询结算信息确认域

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 否 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="554f8963-9361-4fed-bc20-58a5a82e44dc"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="67af3c92-34b1-474d-8549-866fa2db1993"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="d9f7f451-560a-4106-b492-21daad4e179d"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
