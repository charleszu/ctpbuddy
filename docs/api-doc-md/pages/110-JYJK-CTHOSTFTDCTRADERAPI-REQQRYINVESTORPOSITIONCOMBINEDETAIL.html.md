# ReqQryInvestorPositionCombineDetail

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorPositionCombineDetail<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询投资者持仓明细

响应: [OnRspQryInvestorPositionCombineDetail](pages/261-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPOSITIONCOMBINEDETAIL.html.md)
<a id="aa477e4f-9d68-4e7d-bd2b-962f6e260789"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorPositionCombineDetail(CThostFtdcQryInvestorPositionCombineDetailField *pQryInvestorPositionCombineDetail, int nRequestID) = 0;

<a id="2b9e50d2-7f88-427f-8961-b6da3e2634d5"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorPositionCombineDetail：查询组合持仓明细

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | CombInstrumentID | 组合持仓合约编码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="2f854abf-2389-40b3-abf2-76a2701a69a9"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="561e083f-affc-4713-8c55-4754628466a8"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="09cf4e0e-8a16-487e-ac04-75a97fa6f93b"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="anchor-id-01"></a>

<a id="region_header_1"></a>

大商所下套利单成交后查询接口没有找到持仓，是什么原因呢？<a id="region_panel_1"></a>

| 因为柜台开启了大商所rule新型组合保证金算法，就没有套利持仓了。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
