# ReqQryInvestorProductGroupMargin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorProductGroupMargin<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询投资者品种/跨品种保证金

响应: [OnRspQryInvestorProductGroupMargin](pages/263-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPRODUCTGROUPMARGIN.html.md)
<a id="22b85743-8a2e-46f8-87d5-437ed78da7d5"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorProductGroupMargin(CThostFtdcQryInvestorProductGroupMarginField *pQryInvestorProductGroupMargin, int nRequestID) = 0;

<a id="993359d9-176e-4b46-9c53-8d2b491174b5"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorProductGroupMargin：查询投资者品种/跨品种保证金

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | ProductGroupID | 品种 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcHedgeFlagType | HedgeFlag | 投机套保标志 | 不需要填写 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="adcabada-308f-4540-95b3-cf467cc56b6c"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="584b0b0f-24b4-4e8a-ac1a-d679c230af3b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="7388b3ea-4157-4f9b-96f5-c7f510c84d65"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
