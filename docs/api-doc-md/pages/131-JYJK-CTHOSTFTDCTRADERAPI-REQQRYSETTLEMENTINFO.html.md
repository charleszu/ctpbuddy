# ReqQrySettlementInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySettlementInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询投资者结算结果，对应响应[OnRspQrySettlementInfo](pages/282-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSETTLEMENTINFO.html.md)。可以查询当天或历史结算单，也可以查询月结算单，但是前提是CTP柜台生成了相应的日或月结算单。
<a id="5b5908b9-99d6-41bc-85e1-6ed19b930031"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySettlementInfo(CThostFtdcQrySettlementInfoField *pQrySettlementInfo, int nRequestID) = 0;

<a id="5a373b8e-ccb2-452d-894c-e43067642df8"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySettlementInfo：查询投资者结算结果

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcDateType | TradingDay | 交易日 | 是 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 否 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 否 |

TradingDay：查询某一天的结算单，填写格式为“yyyymmdd”；查询某一月的结算单，填写格式为“yyyymm”

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="2ca8f9a2-018c-416a-a79f-79f03881e7a4"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="f6502b62-cfdc-4e66-9fa8-2f79384753ba"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQrySettlementInfoField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.TradingDay,“20180101”);
m_pUserApi->ReqQrySettlementInfo(&a, nRequestID++);

```

<a id="cb56984d-7269-4ab4-b13b-a3f8fdcb29a5"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

为什么我查不到月结单？<a id="region_panel_1"></a>

| 这有可能是CTP系统里没有生成你的月结算单。CTP的日结算单是需要每天结算的时候业务人员去点击生成的，月结算单也是需要定期生成的。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
