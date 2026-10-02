# ReqQryRULEInterParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRULEInterParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RULE跨品种抵扣参数查询，对应响应请求[OnRspQryRULEInterParameter](pages/356-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRULEINTERPARAMETER.html.md)
<a id="72694fdc-89d2-4415-af12-400518fa7a38"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRULEInterParameter(CThostFtdcQryRULEInterParameterField *pQryRULEInterParameter, int nRequestID) = 0;

<a id="df8a0fff-679c-4190-a632-9515889bbb41"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRULEInterParameter：RULE跨品种抵扣参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcInstrumentIDType | Leg1ProdFamilyCode | 第一腿构成品种 | 是 |
| TThostFtdcInstrumentIDType | Leg1ProdFamilyCode | 第二腿构成品种 | 是 |
| TThostFtdcCommodityGroupIDType | CommodityGroupID | 商品群号 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="62715671-f210-4dac-bea2-8dfb18c37d73"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="cd3ae85e-dde6-4036-a45c-7531707e560b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="07255521-3139-4ce1-bf6f-2fb122c2b8cc"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
