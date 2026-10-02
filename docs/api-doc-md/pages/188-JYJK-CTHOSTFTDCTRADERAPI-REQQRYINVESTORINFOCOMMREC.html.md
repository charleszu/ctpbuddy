# ReqQryInvestorInfoCommRec

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorInfoCommRec<a id="content"></a>

<a id="left_menu"></a>

  ** **

投资者申报费阶梯收取记录查询，对应响应请求[OnRspQryInvestorInfoCommRec](pages/359-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORINFOCOMMREC.html.md)

请求查询时3个入参都支持为空，查询期权系列的申报费时通过填写标的的商品代码查询。同一个投资者、同一个商品代码可能会返回两条记录：

一条记录为期货合约的申报费收取记录，IsOptSeries字段返回为0；

一条记录为以此期货合约为标的的系列期权的申报费收取记录，IsOptSeries字段返回为1。
<a id="ecf1accc-057b-4d5f-98eb-cd032ae34ff3"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorInfoCommRec(CThostFtdcQryInvestorInfoCommRecField *pQryInvestorInfoCommRec, int nRequestID) = 0;

<a id="36f669d0-3c6b-47e3-8b72-d3760862bc0d"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorInfoCommRec：投资者申报费阶梯收取记录查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | InstrumentID | 交易所代码 | 是 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="bacb56aa-a5ae-40f0-b826-3d439cff571b"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="18a7a58f-e236-4df8-b960-af1cf4db6986"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="cc33b78a-b3c1-4eae-8f4b-3fb0d7bb0c15"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
