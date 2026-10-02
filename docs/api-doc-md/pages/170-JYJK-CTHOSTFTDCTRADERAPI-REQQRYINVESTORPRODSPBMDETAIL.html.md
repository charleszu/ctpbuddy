# ReqQryInvestorProdSPBMDetail

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorProdSPBMDetail<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者产品SPBM明细查询，对应响应请求[OnRspQryInvestorProdSPBMDetail](pages/341-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPRODSPBMDETAIL.html.md)
<a id="c168571d-cb68-4e52-910c-492f7b81fb35"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorProdSPBMDetail(CThostFtdcQryInvestorProdSPBMDetailField *pQryInvestorProdSPBMDetail, int nRequestID) = 0;

<a id="d2c56996-d3e7-48fa-9ede-70bf0dfd6f97"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorProdSPBMDetail：投资者产品SPBM明细查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | ProdFamilyCode | 品种代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="2ffae1b1-8c6d-4600-bd6e-494c7457e8d3"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="8154dd84-72db-4dee-87ff-15b29ada8138"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="f82c27f5-ea7f-430c-963b-a264122d3866"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
