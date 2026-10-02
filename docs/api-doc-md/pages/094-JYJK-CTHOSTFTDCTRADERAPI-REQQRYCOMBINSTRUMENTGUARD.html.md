# ReqQryCombInstrumentGuard

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryCombInstrumentGuard<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询组合合约安全系数

响应：[OnRspQryCombInstrumentGuard](pages/245-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCOMBINSTRUMENTGUARD.html.md)
<a id="76a862a0-4be1-43a6-a74e-d4a6385958dc"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryCombInstrumentGuard(CThostFtdcQryCombInstrumentGuardField *pQryCombInstrumentGuard, int nRequestID) = 0;

<a id="3a79ba78-0787-4560-bb81-70b5bb47fd15"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryCombInstrumentGuard：组合合约安全系数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 否 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 否 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="27479328-a631-4df5-913a-ca8763891e4f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="3045cd60-0b8b-4d40-8147-c975ff698769"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="96ddad19-0f69-4568-a052-abf52d28b50a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
