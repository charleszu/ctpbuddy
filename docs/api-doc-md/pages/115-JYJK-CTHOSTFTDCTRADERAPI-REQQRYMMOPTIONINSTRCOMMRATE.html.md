# ReqQryMMOptionInstrCommRate

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryMMOptionInstrCommRate<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询做市商期权合约手续费

响应: [OnRspQryMMOptionInstrCommRate](pages/266-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYMMOPTIONINSTRCOMMRATE.html.md)
<a id="60c14aed-5b0c-4aec-8e8e-61d9f01ea1c3"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryMMOptionInstrCommRate(CThostFtdcQryMMOptionInstrCommRateField *pQryMMOptionInstrCommRate, int nRequestID) = 0;

<a id="169de7bc-e4ae-47ed-ae6c-f777255c9f9c"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryMMOptionInstrCommRate：做市商期权手续费率查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="eb0f2fa3-8130-4a32-9531-4a793c49b76d"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="411bf80d-ed4e-483a-b27e-3eca34ad9f22"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="e2974e6c-be33-439d-9ec9-f275261ed5f0"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
