# ReqQryInstrumentCommissionRate

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInstrumentCommissionRate<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询合约手续费率，对应响应[OnRspQryInstrumentCommissionRate](pages/256-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINSTRUMENTCOMMISSIONRATE.html.md)。如果InstrumentID填空，则返回持仓对应的合约手续费率。

目前无法通过一次查询得到所有合约手续费率，如果要查询所有，则需要通过多次查询得到。

<a id="98a73fe7-5300-4504-b4a6-2720f22fc1be"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInstrumentCommissionRate(CThostFtdcQryInstrumentCommissionRateField *pQryInstrumentCommissionRate, int nRequestID) = 0;

<a id="d5752b96-aa9a-4325-8fb5-60246dbd6846"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInstrumentCommissionRate：查询手续费率

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

InstrumentID：返回手续费率对应的合约。

但是如果在柜台没有设置具体合约的手续费率，则默认会返回产品的手续费率，InstrumentID就为对应产品ID。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="859b1f5e-2747-4222-a2fa-0dd7d7cce01d"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="5197d5cb-8a60-46dc-be46-1c79a7a0e0db"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryInstrumentCommissionRateField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
m_pUserApi->ReqQryInstrumentCommissionRate(&a, nRequestID++);

```

<a id="b21c2b35-76b4-430d-8751-d61238d0b35e"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="anchor-id-01"></a>

<a id="region_header_1"></a>

查询返回结果是交易所手续费率还是投资者手续费率？<a id="region_panel_1"></a>

| 返回的是投资者手续费率。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
