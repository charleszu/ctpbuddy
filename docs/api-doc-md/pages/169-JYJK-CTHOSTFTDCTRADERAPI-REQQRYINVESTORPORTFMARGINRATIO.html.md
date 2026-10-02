# ReqQryInvestorPortfMarginRatio

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorPortfMarginRatio<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者新型组合保证金系数查询，对应响应请求[OnRspQryInvestorPortfMarginRatio](pages/340-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPORTFMARGINRATIO.html.md)
<a id="963850c8-0d3f-4243-8053-c41b63e52ed4"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorPortfMarginRatio(CThostFtdcQryInvestorPortfMarginRatioField *pQryInvestorPortfMarginRatio, int nRequestID) = 0;

<a id="b68cfec4-ae12-4e26-b2de-17bb499b3d5f"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorPortfMarginRatio：投资者新型组合保证金系数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcProductIDType | ProductGroupID | 产品群代码 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="805a3ca7-6438-49ac-9b2b-a2b2439e8a98"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="f5344841-6b9b-4536-a71e-8c92eef5bc8b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="202729f6-450a-4307-8948-b7017649a3ea"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
