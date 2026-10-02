# ReqQryInvestorCommoditySPMMMargin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorCommoditySPMMMargin<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者商品组SPMM记录查询，对应响应请求[OnRspQryInvestorCommoditySPMMMargin](pages/342-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORCOMMODITYSPMMMARGIN.html.md)
<a id="180879a9-b6e0-4607-b77d-e2b126e74945"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorCommoditySPMMMargin(CThostFtdcQryInvestorCommoditySPMMMarginField *pQryInvestorCommoditySPMMMargin, int nRequestID) = 0;

<a id="d7bdd47b-ad69-4cd4-8784-2c15b7f1cebf"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorCommoditySPMMMargin：投资者商品组SPMM记录查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcSPMMProductIDType | CommodityID | 商品组代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="ddec508a-eb31-444e-87a8-fc5c2d92d607"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="088876ed-2c92-45f4-b82c-35b7ddf6257d"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryInvestorCommoditySPMMMarginField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.CommodityID, "cu&cu_o");
m_pUserApi->ReqQryInvestorCommoditySPMMMargin(&a, nRequestID++);

```

<a id="ea480e3a-502d-4ace-9fcb-d7d8165783f7"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
