# ReqQryInvestorProdRULEMargin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorProdRULEMargin<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者产品RULE保证金查询，对应响应请求[OnRspQryInvestorProdRULEMargin](pages/357-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPRODRULEMARGIN.html.md)
<a id="79cf8feb-24da-4ac3-a2cc-50c6cc92cdc9"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorProdRULEMargin(CThostFtdcQryInvestorProdRULEMarginField *pQryInvestorProdRULEMargin, int nRequestID) = 0;

<a id="ebfdb2c8-eadf-49b1-b58a-0ad48823a8b3"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorProdRULEMargin：投资者产品RULE保证金查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | ProdFamilyCode | 品种代码 | 是 |
| TThostFtdcCommodityGroupIDType | CommodityGroupID | 商品群号 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="657e4b70-f069-45cb-9c3d-c534d0f15f35"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="59d724ea-71e4-4627-ba97-ff85fcf543d9"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="d9c409e3-8185-4662-93c9-ec3e3124dff9"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
