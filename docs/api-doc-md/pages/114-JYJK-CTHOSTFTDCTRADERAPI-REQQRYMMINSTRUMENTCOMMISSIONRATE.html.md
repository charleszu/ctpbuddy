# ReqQryMMInstrumentCommissionRate

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryMMInstrumentCommissionRate<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询做市商合约手续费率

响应: [OnRspQryMMInstrumentCommissionRate](pages/265-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYMMINSTRUMENTCOMMISSIONRATE.html.md)
<a id="19eaaa9f-e883-43da-8618-d7101229d8f1"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryMMInstrumentCommissionRate(CThostFtdcQryMMInstrumentCommissionRateField *pQryMMInstrumentCommissionRate, int nRequestID) = 0;

<a id="8443cb7d-77fc-4b10-958b-6a452ff8804f"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryMMInstrumentCommissionRate：查询做市商合约手续费率

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="77e0c577-0c02-4e67-9bb7-1756b80d363c"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="1eaa8ab3-dbb3-49a7-8032-e32827f72324"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="11ab8299-f743-4fdb-9d99-fd7f6a3abdc4"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
