# ReqQryRULEInstrParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRULEInstrParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RULE合约保证金参数查询，对应响应请求[OnRspQryRULEInstrParameter](pages/354-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRULEINSTRPARAMETER.html.md)
<a id="9a141da9-9249-46d5-bc5e-9e41fafc5119"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRULEInstrParameter(CThostFtdcQryRULEInstrParameterField *pQryRULEInstrParameter, int nRequestID) = 0;

<a id="440814cf-ccbf-4edd-bcb1-e26371d52ccf"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRULEInstrParameter：RULE合约保证金参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="ef73e466-512e-4708-8b32-5ef9f6b4868b"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="7089ebdd-37fd-473a-9a21-49ad04c78c1b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="8ea930a5-867d-4b2f-9a7a-2c0cd7954b3c"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
