# ReqQryTradingAccount

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryTradingAccount<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询资金账户，

响应: [OnRspQryTradingAccount](pages/285-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRADINGACCOUNT.html.md)
<a id="cd9d8175-3cec-4829-8e88-f2c45b1c0560"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryTradingAccount(CThostFtdcQryTradingAccountField *pQryTradingAccount, int nRequestID) = 0;

<a id="6a396c18-e479-47c8-a801-7ad768eb5e83"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryTradingAccount：查询资金账户

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 是 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 否 |
| TThostFtdcBizTypeType | BizType | 业务类型 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="956a3444-949b-4fdd-8074-3f4bbb34702a"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="bba14a93-663d-4939-9b3e-c56715abe741"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryTradingAccountField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.CurrencyID, "CNY");
m_pUserApi->ReqQryTradingAccount(&a, nRequestID++);

```

<a id="c793d492-59a6-4782-a079-c3402fa98b60"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
