# ReqQryRULEIntraParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRULEIntraParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RULE品种内对锁仓折扣参数查询，对应响应请求[OnRspQryRULEIntraParameter](pages/355-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRULEINTRAPARAMETER.html.md)
<a id="fbfc7e30-ebdc-410a-8f2d-b51bebad392b"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRULEIntraParameter(CThostFtdcQryRULEIntraParameterField *pQryRULEIntraParameter, int nRequestID) = 0;

<a id="df8d7f54-5844-488a-8a6b-ae647c0ce387"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRULEIntraParameter：RULE品种内对锁仓折扣参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcInstrumentIDType | ProdFamilyCode | 品种代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="bdff5bfe-31c1-471a-a03f-de51a6f68ef8"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="0de5adee-05c1-437b-8ea1-694f0e6c60ee"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="c7ceec82-90dc-4f7b-950f-aede1831e0ad"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
