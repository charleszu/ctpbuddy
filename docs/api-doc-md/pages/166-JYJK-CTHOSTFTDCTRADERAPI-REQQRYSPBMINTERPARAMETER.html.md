# ReqQrySPBMInterParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySPBMInterParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求SPBM跨品种抵扣参数查询，对应响应请求[OnRspQrySPBMInterParameter](pages/337-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMINTERPARAMETER.html.md)
<a id="a1748866-9e09-4151-9fe7-059d4e72c881"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySPBMInterParameter(CThostFtdcQrySPBMInterParameterField *pQrySPBMInterParameter, int nRequestID) = 0;

<a id="f771a8a5-fb98-4003-a35f-b2f4e9c889fd"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySPBMInterParameter：SPBM跨品种抵扣参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcInstrumentIDType | Leg1ProdFamilyCode | 第一腿构成品种 | 是 |
| TThostFtdcInstrumentIDType | Leg2ProdFamilyCode | 第二腿构成品种 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="148b2c69-209b-463d-9482-dad749664e9c"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="63b6c4cc-2178-4fb7-8bba-406669131172"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="1cad764e-dafb-450a-ae67-39fb26193214"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
