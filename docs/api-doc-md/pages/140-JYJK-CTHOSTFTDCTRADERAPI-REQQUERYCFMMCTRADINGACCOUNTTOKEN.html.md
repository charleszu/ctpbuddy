# ReqQueryCFMMCTradingAccountToken

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQueryCFMMCTradingAccountToken<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询监控中心用户令牌，对应响应[OnRspQueryCFMMCTradingAccountToken](pages/291-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQUERYCFMMCTRADINGACCOUNTTOKEN.html.md)、[OnRtnCFMMCTradingAccountToken](pages/304-JYJK-CTHOSTFTDCTRADERSPI-ONRTNCFMMCTRADINGACCOUNTTOKEN.html.md)。
<a id="6d92dcb0-308d-4687-bc78-c787a045e059"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQueryCFMMCTradingAccountToken(CThostFtdcQueryCFMMCTradingAccountTokenField *pQueryCFMMCTradingAccountToken, int nRequestID) = 0;

<a id="089585af-319c-4c91-a27e-722c0031de72"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQueryCFMMCTradingAccountToken：查询监控中心用户令牌

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="b84b21d6-9968-4e8d-b670-c7c92fe6696e"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="5098d4e8-e1c3-44d2-9c55-def81be77c18"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="33e9c870-17f1-4cf6-a50e-8ef753777261"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
