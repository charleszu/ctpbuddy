# ReqQrySPBMFutureParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySPBMFutureParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求SPBM期货合约参数查询，对应响应请求[OnRspQrySPBMFutureParameter](pages/334-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMFUTUREPARAMETER.html.md)
<a id="9889c7b8-0a56-4a55-ba94-5289fb1b7c64"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySPBMFutureParameter(CThostFtdcQrySPBMFutureParameterField *pQrySPBMFutureParameter, int nRequestID) = 0;

<a id="cd21bf7b-5dd5-4f3f-930e-7a1256cc8fba"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySPBMFutureParameter：SPBM期货合约保证金参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcInstrumentIDType | ProdFamilyCode | 品种代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="7275872d-91f0-451c-a5d7-3e2d9fe46ecf"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="f8a151e2-dab0-41c7-b5d7-2fce8763b405"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="b339940f-7481-46ab-88b1-6dcd3e068487"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
