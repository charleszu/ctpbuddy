# ReqQryExchange

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryExchange<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询交易所

响应: [OnRspQryExchange](pages/249-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYEXCHANGE.html.md)
<a id="96e7aa40-34d0-4763-9201-811530695035"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryExchange(CThostFtdcQryExchangeField *pQryExchange, int nRequestID) = 0;

<a id="f0b6f5c6-8c80-4722-82b1-918781af1304"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryExchange：查询交易所

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="42406abf-d0f1-4dd7-820a-86f9a18dc410"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="b4aeadb3-a3df-483b-958f-05039148edb6"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="9a40b2ac-b64b-4d65-aa50-36bcf83b22da"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
