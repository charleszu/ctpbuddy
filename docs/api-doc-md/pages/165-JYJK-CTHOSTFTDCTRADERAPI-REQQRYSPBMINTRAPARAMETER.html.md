# ReqQrySPBMIntraParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySPBMIntraParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求SPBM品种内对锁仓折扣参数查询，对应响应请求[OnRspQrySPBMIntraParameter](pages/336-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMINTRAPARAMETER.html.md)
<a id="6bbaddeb-68a8-49df-863c-60bd2b420b02"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySPBMIntraParameter(CThostFtdcQrySPBMIntraParameterField *pQrySPBMIntraParameter, int nRequestID) = 0;

<a id="6c1b689c-fea8-4f56-9227-e27a8ee963af"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySPBMIntraParameter：SPBM品种内对锁仓折扣参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcInstrumentIDType | ProdFamilyCode | 品种代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="2bd79927-0aac-4b98-b4b9-b08ee98bf370"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="171898fd-a915-4e18-8f5a-aa279eee3007"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="efd91e61-415d-4a73-954d-35ee9bf97ef3"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
