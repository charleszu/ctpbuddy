# ReqQrySPBMOptionParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySPBMOptionParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求SPBM期权合约参数查询，对应响应请求[OnRspQrySPBMOptionParameter](pages/335-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMOPTIONPARAMETER.html.md)
<a id="3fdc7735-3eeb-4716-8907-707f299d3d04"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySPBMOptionParameter(CThostFtdcQrySPBMOptionParameterField *pQrySPBMOptionParameter, int nRequestID) = 0;

<a id="6f9b7315-de0f-432c-90fa-31ffc36e1596"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySPBMOptionParameter：SPBM期权合约保证金参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcInstrumentIDType | ProdFamilyCode | 品种代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="a6b7ba8b-160f-4233-9468-9af3ec27b2cf"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="d62edda7-d897-49da-bfe2-2846fce7c93b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="f0ebeb9d-28d7-492f-a89b-722f640a78bc"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
