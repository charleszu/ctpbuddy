# ReqQryOptionInstrTradeCost

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryOptionInstrTradeCost<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询期权交易成本，该函数用于查期权保证金，对应响应[OnRspQryOptionInstrTradeCost](pages/269-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYOPTIONINSTRTRADECOST.html.md)。

保证金=max(权利金+FixedMargin,MiniMargin)，用户可根据此公式计算实时保证金。

<a id="3f14ce14-8668-4acd-a873-d858e5c3e364"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryOptionInstrTradeCost(CThostFtdcQryOptionInstrTradeCostField *pQryOptionInstrTradeCost, int nRequestID) = 0;

<a id="6e0616c3-9a87-424b-9433-0e24758106e5"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryOptionInstrTradeCost：期权交易成本查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcHedgeFlagType | HedgeFlag | 投机套保标志 | 是 |
| TThostFtdcPriceType | InputPrice | 期权合约报价 | 是 |
| TThostFtdcPriceType | UnderlyingPrice | 标的价格,填0则用昨结算价 | 是 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="67837026-d427-4ce9-93ba-7635a075b5a1"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="df60d3d9-e166-414e-b625-4a66da29c412"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryOptionInstrTradeCostField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "ad2503C13800");
a.HedgeFlag = THOST_FTDC_HF_Speculation;
a.InputPrice = 300;
m_pUserApi->ReqQryOptionInstrTradeCost(&a, nRequestID++);

```

<a id="36801fe5-4c32-4d19-b52f-abebb7eac580"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

为什么我用这个接口算出来的保证金跟资金查询得到的保证金占用不一致？<a id="region_panel_1"></a>

| ReqQryOptionInstrTradeCost计算出的期权保证金跟资金查询里的期权保证金的计算方式不一样。
ReqQryOptionInstrTradeCost只是估计计算，因为其使用的公式（保证金=max(权利金+FixedMargin,MiniMargin)）中的权利金部分在计算时使用的期权价格是InputPrice。
而资金查询里的期权保证金计算公式中的期权价格是使用max算法（max(昨结算，最新价)）得到的。
所以两者存在差别。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
