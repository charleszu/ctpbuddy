# ReqQryInvestorProdRCAMSMargin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorProdRCAMSMargin<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者品种RCAMS保证金查询，对应响应请求[OnRspQryInvestorProdRCAMSMargin](pages/353-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPRODRCAMSMARGIN.html.md)
<a id="a0804fda-abcd-4073-8397-cd0eb0526a2a"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorProdRCAMSMargin(CThostFtdcQryInvestorProdRCAMSMarginField *pQryInvestorProdRCAMSMargin, int nRequestID) = 0;

<a id="0f3f9f11-fad3-4f5e-bf4e-449d18cb5ed7"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorProdRCAMSMargin：投资者品种RCAMS保证金查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcProductIDType | CombProductID | 产品组合代码 | 是 |
| TThostFtdcProductIDType | ProductGroupID | 商品群代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="53155ca0-df43-4dc5-b3cd-8fc643a72890"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="a81ee987-9e8e-4262-afe8-6133f61f8318"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="b364fec2-5812-4961-a79e-b0d9e806bd32"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
