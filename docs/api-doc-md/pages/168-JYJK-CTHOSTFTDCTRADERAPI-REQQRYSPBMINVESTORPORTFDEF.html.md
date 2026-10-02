# ReqQrySPBMInvestorPortfDef

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySPBMInvestorPortfDef<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者SPBM套餐选择查询，对应响应请求[OnRspQrySPBMInvestorPortfDef](pages/339-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMINVESTORPORTFDEF.html.md)
<a id="ac8f896f-e2a7-4fe5-b7ab-bd19bff52e47"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySPBMInvestorPortfDef(CThostFtdcQrySPBMInvestorPortfDefField *pQrySPBMInvestorPortfDef, int nRequestID) = 0;

<a id="e7a599d6-5f43-4d68-9151-70b20cddbe2e"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySPBMInvestorPortfDef：投资者套餐选择查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="4be0306c-8c8f-47e8-968a-ece6336e655b"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="e70e0495-9405-4033-bf5e-1e3e11e778f7"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="ed88a3e7-5d92-47f9-83a3-bc5add3d2588"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
