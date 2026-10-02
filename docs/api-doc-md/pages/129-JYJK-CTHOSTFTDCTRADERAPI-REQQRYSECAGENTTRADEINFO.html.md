# ReqQrySecAgentTradeInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySecAgentTradeInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询二级代理商信息

响应: [OnRspQrySecAgentTradeInfo](pages/280-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSECAGENTTRADEINFO.html.md)
<a id="5d9f516c-e0b9-4bca-8b66-01b9043e314a"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySecAgentTradeInfo(CThostFtdcQrySecAgentTradeInfoField *pQrySecAgentTradeInfo, int nRequestID) = 0;

<a id="2151e220-c0f7-40bf-b4b8-e7f78a59ae03"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySecAgentTradeInfo：查询二级代理商信息

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcAccountIDType | BrokerSecAgentID | 境外中介机构资金帐号 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="5e4e0e1e-56a2-4a14-b19f-1908240af7f0"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="24255d06-9ae2-45f8-9825-d1d576b1e7ad"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="8402b795-4a22-4772-a939-75f6de5d2575"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
