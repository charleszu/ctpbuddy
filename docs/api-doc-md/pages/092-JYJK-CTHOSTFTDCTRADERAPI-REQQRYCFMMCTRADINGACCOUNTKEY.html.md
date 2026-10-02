# ReqQryCFMMCTradingAccountKey

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryCFMMCTradingAccountKey<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询保证金监管系统经纪公司资金账户密钥，**此接口已弃用**，请使用[ReqQueryCFMMCTradingAccountToken](pages/140-JYJK-CTHOSTFTDCTRADERAPI-REQQUERYCFMMCTRADINGACCOUNTTOKEN.html.md)查询。

响应: [OnRspQryCFMMCTradingAccountKey](pages/243-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCFMMCTRADINGACCOUNTKEY.html.md)
<a id="e457a756-4eb3-4e36-acd1-f462623b71a3"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryCFMMCTradingAccountKey(CThostFtdcQryCFMMCTradingAccountKeyField *pQryCFMMCTradingAccountKey, int nRequestID) = 0;

<a id="65e75c7e-f819-4307-a24c-cb2975834140"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryCFMMCTradingAccountKey：请求查询保证金监管系统经纪公司资金账户密钥

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 否 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 否 |

接口弃用

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="24f40e8c-9af3-420c-b44a-398c1526d3fa"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="fe303045-7317-437a-b825-63e567b638b4"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryCFMMCTradingAccountKeyField a = { 0 };
strcpy(a.BrokerID, "9999");
strcpy(a.InvestorID, "1000001");
m_pUserApi->ReqQryCFMMCTradingAccountKey(&a, nRequestID++);

```

<a id="c6bb1d68-a888-4050-af17-6d10bc73fda7"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
