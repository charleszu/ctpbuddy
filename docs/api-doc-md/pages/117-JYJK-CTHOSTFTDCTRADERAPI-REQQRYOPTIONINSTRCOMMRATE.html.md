# ReqQryOptionInstrCommRate

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryOptionInstrCommRate<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询期权合约手续费

响应: [OnRspQryOptionInstrCommRate](pages/268-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYOPTIONINSTRCOMMRATE.html.md)
<a id="80490358-be5e-40fe-8573-8c8c353f8b91"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryOptionInstrCommRate(CThostFtdcQryOptionInstrCommRateField *pQryOptionInstrCommRate, int nRequestID) = 0;

<a id="93e7373b-2d68-48f8-ac67-1c100c3a4aed"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryOptionInstrCommRate：期权手续费率查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="6a141a56-7aa5-4844-bcbf-d52767742a02"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="ef548ec5-25da-446b-a7c3-33cea85ba93e"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

<a id="anchor-id-02"></a>

```
CThostFtdcQryOptionInstrCommRateField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "ag2311C1200");
m_pUserApi->ReqQryOptionInstrCommRate(&a, nRequestID++);

```

<a id="27adc8c3-2e81-4516-83b1-d69622294383"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="anchor-id-01"></a>

<a id="region_header_1"></a>

合约填空，是否能查询所有期权合约手续费率？<a id="region_panel_1"></a>

| 不能，填空只能返回持仓的期权合约手续费率。查询所有期权合约需要一个个请求查询。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
