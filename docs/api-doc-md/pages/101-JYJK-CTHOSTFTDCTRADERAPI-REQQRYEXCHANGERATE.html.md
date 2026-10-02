# ReqQryExchangeRate

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryExchangeRate<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询汇率

响应：[OnRspQryExchangeRate](pages/252-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYEXCHANGERATE.html.md)
<a id="363b39b4-5358-46bf-9207-fa9d0c0d2bee"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryExchangeRate(CThostFtdcQryExchangeRateField *pQryExchangeRate, int nRequestID) = 0;

<a id="c8537598-15d5-4191-99f7-0a1b11e6332a"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryExchangeRate：查询汇率

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcCurrencyIDType | FromCurrencyID | 源币种 | 是 |
| TThostFtdcCurrencyIDType | ToCurrencyID | 目标币种 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="08cbb4ce-9743-45bf-8253-4bde98394df4"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="c785c3da-4329-4f97-81a7-b22a2b51c34e"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="f04a3f14-9170-472c-8dc3-df76b7c285a6"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
