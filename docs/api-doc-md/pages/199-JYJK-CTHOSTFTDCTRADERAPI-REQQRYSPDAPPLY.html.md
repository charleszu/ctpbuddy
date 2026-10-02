# ReqQrySpdApply

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySpdApply<a id="content"></a>

<a id="left_menu"></a>

  ** **

套利确认查询请求,对应回报[OnRspQrySpdApply](pages/372-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPDAPPLY.html.md)
<a id="bac5ba40-140c-4336-85c6-3b069bbf8489"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySpdApply(CThostFtdcQrySpdApplyField *pQrySpdApply, int nRequestID) = 0;

<a id="c6cfd2e5-7366-424d-9795-36efdeffb753"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySpdApply：套利套保申请查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcOrderSysIDType | OrderSysID | 报单编号 | 是 |
| TThostFtdcExchangeInstIDType | FirstLegInstrumentID | 第一腿合约编码 | 是 |
| TThostFtdcExchangeInstIDType | SecondLegInstrumentID | 第二腿合约编码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="8b321284-ba5d-4f77-b020-f1104978df28"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="b2d2e24f-d98c-4d09-ac8b-da9781b9e05c"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="add69b87-e600-428b-8c94-e284c9b584d1"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
