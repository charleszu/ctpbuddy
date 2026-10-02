# ReqQryInvestorPositionDetail

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorPositionDetail<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询投资者持仓明细，对应响应[OnRspQryInvestorPositionDetail](pages/262-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPOSITIONDETAIL.html.md)。

CTP系统根据来自交易所的成交记录生成持仓明细记录，一笔成交记录对应一条持仓明细记录。
<a id="d2f0d09c-1ee0-4eae-92ea-2b041bebe235"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorPositionDetail(CThostFtdcQryInvestorPositionDetailField *pQryInvestorPositionDetail, int nRequestID) = 0;

<a id="067ae213-9240-45bd-86dc-fdd1d2c989ca"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorPositionDetail：查询投资者持仓明细

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="763ed001-05ad-420e-812e-c1112879230d"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="5f152042-4a6c-4e4d-b713-1c3ecd4d0ce9"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="5ace4d74-feb7-4119-9d89-fe264eab3e7a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
