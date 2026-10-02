# ReqQryTraderOffer

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryTraderOffer<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询交易员报盘机，对应响应请求[OnRspQryTraderOffer](pages/333-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRADEROFFER.html.md)
<a id="9bdf2d1e-356d-4559-a32f-e0a115c8e4cc"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryTraderOffer(CThostFtdcQryTraderOfferField *pQryTraderOffer, int nRequestID) = 0;

<a id="a3cebdbb-4788-407e-8d88-ab030d2f2fed"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryTraderOffer：查询交易员报盘机

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcParticipantIDType | ParticipantID | 会员代码 | 否 |
| TThostFtdcTraderIDType | TraderID | 交易所交易员代码 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="3f98278f-4a02-43a1-8721-4e60c4991eec"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="a1572b0f-cb4d-4fe1-bc4e-ffb8fcf6e940"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="578eb6a0-c77d-47c9-9d1c-af32b6d2f9c1"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
