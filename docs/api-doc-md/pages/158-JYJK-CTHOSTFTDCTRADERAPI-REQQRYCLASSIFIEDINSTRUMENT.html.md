# ReqQryClassifiedInstrument

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryClassifiedInstrument<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询分类合约，对应响应请求[OnRspQryClassifiedInstrument](pages/329-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCLASSIFIEDINSTRUMENT.html.md)

详见  [6.5.1版本更新说明补充说明](pages/006-6.5.1BBGXSMBCSM.html.md)
<a id="c9279df1-bbd1-43a1-9c59-a0991481d2af"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryClassifiedInstrument(CThostFtdcQryClassifiedInstrumentField *pQryClassifiedInstrument, int nRequestID) = 0;

<a id="feb3a272-e80c-464e-a979-aef36b585f2e"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryClassifiedInstrument：查询分类合约

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 否 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcExchangeInstIDType | ExchangeInstID | 合约在交易所的代码 | 否 |
| TThostFtdcInstrumentIDType | ProductID | 产品代码 | 否 |
| TThostFtdcTradingTypeType | TradingType | 合约交易状态 | 是 |
| TThostFtdcClassTypeType | ClassType | 合约分类类型 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="f8242616-8f2f-472b-808a-19d67acd1dbb"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="c2f6562f-0bf5-47a5-8db8-3e9e6eef7670"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="78ccf0b2-dcf9-4ffc-b74d-789cb0bf31a4"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
