# ReqQryInvestorCommodityGroupSPMMMargin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorCommodityGroupSPMMMargin<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者商品群SPMM记录查询，对应响应请求[OnRspQryInvestorCommoditySPMMMargin](pages/342-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORCOMMODITYSPMMMARGIN.html.md)
<a id="48fa7fea-c9c2-460f-81ee-16005bfe903c"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorCommodityGroupSPMMMargin(CThostFtdcQryInvestorCommodityGroupSPMMMarginField *pQryInvestorCommodityGroupSPMMMargin, int nRequestID) = 0;

<a id="9ddd5e9e-ea34-4d37-9ad0-6f40154dbe3e"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorCommodityGroupSPMMMargin:投资者商品群SPMM记录查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcSPMMProductIDType | CommodityGroupID | 商品群代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="d31c77ff-980d-438c-906b-613a67d08be0"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="6de4f571-aecc-4439-9e10-66c217c541ab"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryInvestorCommodityGroupSPMMMarginField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.CommodityGroupID, "01");
m_pUserApi->ReqQryInvestorCommodityGroupSPMMMargin(&a, nRequestID++);

```

<a id="24941afd-a4a1-4534-b5ef-c12481b3fb56"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
