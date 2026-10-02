# ReqQryHedgeCfm

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryHedgeCfm<a id="content"></a>

<a id="left_menu"></a>

  ** **

套保确认查询请求,查询返回为[OnRspQryHedgeCfm](pages/378-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYHEDGECFM.html.md)
<a id="3c61a25b-e105-4e69-bb7b-40f68759ab05"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryHedgeCfm(CThostFtdcQryHedgeCfmField *pQryHedgeCfm, int nRequestID) = 0;

<a id="91e2f27f-0787-4d99-896e-7bda206f7bfa"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryHedgeCfm：套利套保申请查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcOrderSysIDType | OrderSysID | 报单编号 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="e224c9e6-0cc8-4778-8fe5-96503d5b62c8"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="40f01b4f-8ad1-4fee-8ef6-e7f71b6cf5c0"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="1dee2b44-78e6-4d40-920c-b88caa9b8adc"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
