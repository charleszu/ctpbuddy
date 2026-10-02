# ReqForQuoteInsert

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqForQuoteInsert<a id="content"></a>

<a id="left_menu"></a>

  ** **

询价录入请求

错误响应: [OnErrRtnForQuoteInsert](pages/209-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNFORQUOTEINSERT.html.md)，[OnRspForQuoteInsert](pages/229-JYJK-CTHOSTFTDCTRADERSPI-ONRSPFORQUOTEINSERT.html.md)

正确响应: [OnRtnForQuoteRsp](pages/309-JYJK-CTHOSTFTDCTRADERSPI-ONRTNFORQUOTERSP.html.md)

详见[做市商询价和报价](pages/388-QTYWGZ-BJHXJ.html.md)

关于接口中的重要序号说明详见[接口中一些重要序号说明](pages/401-QTYWGZ-JKZYXZYXHSM.html.md)
<a id="68142fed-89f5-48d1-b745-fe2220bb4b19"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqForQuoteInsert(CThostFtdcInputForQuoteField *pInputForQuote, int nRequestID) = 0;

<a id="21600d23-4107-46e9-b02a-3923f3c8a509"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputForQuote：输入的询价

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 期权合约名称 |
| TThostFtdcOrderRefType | ForQuoteRef | 询价引用 | 选填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 无 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

ForQuoteRef：需要纯数字递增，不填则ctp自动填写

IPAddress：中继需填写客户IP地址；非中继填写无效，直接取登录成功会话中的IP。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

MacAddress：中继需填写客户MAC地址；非中继填写无效，直接取登录成功会话中的MAC。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="68ff96bd-1b99-47ff-a808-eba020e6a171"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="d3b77dd8-2436-4853-8925-24b3ed638ff0"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcInputForQuoteField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
strcpy_s(a.UserID, "1000001");
strcpy_s(a.ExchangeID, "SHFE");
m_pUserApi->ReqForQuoteInsert(&a, nRequestID++);

```

<a id="b2a5eea1-a215-483d-a32a-213b95b810d9"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

询价时报：“没有该合约的做市商”？<a id="region_panel_1"></a>

| 询价合约不对。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
