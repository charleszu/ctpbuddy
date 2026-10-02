# ReqQryRiskSettleInvstPosition

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRiskSettleInvstPosition<a id="content"></a>

<a id="left_menu"></a>

  ** **

投资者风险结算持仓查询，对应响应请求[OnRspQryRiskSettleInvstPosition](pages/331-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRISKSETTLEINVSTPOSITION.html.md)

详见  [6.6.1P1版本更新说明](pages/007-6.6.1P1BBGXSM.html.md)
<a id="f8a068bf-17ae-4890-8731-48dc28742723"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRiskSettleInvstPosition(CThostFtdcQryRiskSettleInvstPositionField *pQryRiskSettleInvstPosition, int nRequestID) = 0;

<a id="bf072137-b80d-4dc3-912a-419d67f37426"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRiskSettleInvstPosition：投资者风险结算持仓查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 否 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 否 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="a78e2666-83ef-4a72-b8c7-a66bd9f36f23"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="796d78d3-c421-4a21-9724-5026ae205c58"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="64a2b681-f2bb-45bd-b74d-659c86930d88"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
