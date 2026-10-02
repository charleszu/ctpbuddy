# ReqQryProductExchRate

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryProductExchRate<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询产品报价汇率

响应: [OnRspQryProductExchRate](pages/275-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYPRODUCTEXCHRATE.html.md)
<a id="5c705034-017f-4943-8959-e1b743ca046e"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryProductExchRate(CThostFtdcQryProductExchRateField *pQryProductExchRate, int nRequestID) = 0;

<a id="09053ce9-f92a-4c59-9768-001e6abdf631"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryProductExchRate：产品报价汇率查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | ProductID | 产品代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="0cbde2fe-e38c-4b1f-ae3b-26b2bd1ed44a"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="d88a256b-e118-45ca-ab13-7b8f6a247018"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="67cea87f-0399-4a2f-b354-0ffced25e99a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
