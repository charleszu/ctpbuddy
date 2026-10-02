# ReqQryInstrument

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInstrument<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询合约，填空可以查询到所有合约。

响应:[OnRspQryInstrument](pages/255-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINSTRUMENT.html.md)
<a id="2c9b14df-08b6-4164-aad6-4868249c118a"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInstrument(CThostFtdcQryInstrumentField *pQryInstrument, int nRequestID) = 0;

<a id="c1afed2b-1985-4baa-b567-f3ea74e028de"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInstrument：查询合约

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcExchangeInstIDType | ExchangeInstID | 合约在交易所的代码 | 是 |
| TThostFtdcInstrumentIDType | ProductID | 产品代码 | 是 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldExchangeInstIDType | reserve2 | 保留的无效字段 | 否 |
| TThostFtdcOldInstrumentIDType | reserve3 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="d01d888f-dd9d-494f-8254-79189ae25f4b"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="2c1ff269-94c7-400a-a11f-f4abe722bc2f"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryInstrumentField a = { 0 };
strcpy_s(a.InstrumentID, "rb1809");
strcpy_s(a.ExchangeID, "SHFE");
m_pUserApi->ReqQryInstrument(&a, nRequestID++);

```

<a id="3098bcfe-ba16-4462-9394-8bb21fdcc4fe"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

为何查询不到郑商所跨式和宽跨式套利合约？<a id="region_panel_1"></a>

| 郑商所没有推跨式和宽跨式套利合约，所以查询不到此类合约。 |
|---|

<a id="region_tail_1"></a>

<a id="region_header_2"></a>

为何查询中金所的EFP（期转现）合约，InstrumentName为空？<a id="region_panel_2"></a>

| 交易所就没有推送instrumentname。 |
|---|

<a id="region_tail_2"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
