# ReqQrySPBMAddOnInterParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySPBMAddOnInterParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求SPBM附加跨品种抵扣参数查询，对应响应请求[OnRspQrySPBMAddOnInterParameter](pages/346-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMADDONINTERPARAMETER.html.md)
<a id="7d357d7b-c41a-4c52-af3a-eca0da8c015b"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySPBMAddOnInterParameter(CThostFtdcQrySPBMAddOnInterParameterField *pQrySPBMAddOnInterParameter, int nRequestID) = 0;

<a id="aef76724-3063-42c6-8d0e-5614584b25bf"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySPBMAddOnInterParameter：SPBM附加跨品种抵扣参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcInstrumentIDType | Leg1ProdFamilyCode | 第一腿构成品种 | 是 |
| TThostFtdcInstrumentIDType | Leg2ProdFamilyCode | 第二腿构成品种 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="553d7576-43c6-4a6a-8b08-fdfb7f8cdbe1"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="95640473-5d44-4817-b661-ccd9c4ac56b9"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="095d7941-7fd8-4286-8fdf-13e9a0517498"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
