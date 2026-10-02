# ReqQryTradingCode

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryTradingCode<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询交易编码

响应: [OnRspQryTradingCode](pages/286-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRADINGCODE.html.md)
<a id="07246a36-02c7-4443-a0bc-f28705c4937a"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryTradingCode(CThostFtdcQryTradingCodeField *pQryTradingCode, int nRequestID) = 0;

<a id="5e272574-4138-44fe-a86b-7775a62dc87a"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryTradingCode：查询交易编码

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcClientIDType | ClientID | 客户代码 | 是 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcClientIDTypeType | ClientIDType | 交易编码类型 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="cd1889a5-4c7c-4afa-958d-a6b164781f52"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="2c04dc01-2413-4a7c-9bf9-e3437156cc68"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="f9080095-c257-4664-bb26-30e44d3c2bd0"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
