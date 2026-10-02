# ReqQrySecAgentTradingAccount

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySecAgentTradingAccount<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询资金账户

响应: [OnRspQrySecAgentTradingAccount](pages/281-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSECAGENTTRADINGACCOUNT.html.md)
<a id="1d9ba819-5a59-43c3-851d-f1120f47ef01"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySecAgentTradingAccount(CThostFtdcQryTradingAccountField *pQryTradingAccount, int nRequestID) = 0;

<a id="a1356cf5-0b92-4e24-ba2d-967a7c085b78"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryTradingAccount：请求查询资金账户

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 否 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 否 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 否 |
| TThostFtdcAccountIDType | AccountID | 业务类型 | 否 |
| TThostFtdcBizTypeType | BizType | 投资者帐号 | 否 |

使用二级代理商操作员管理的投资者进行查询，字段值填写正确时，均无查询结果。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="7c5aa25a-ab0e-45d4-b95d-4599309b3f65"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="bb86d695-a736-4576-90bb-87754596174c"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="b7319d07-a469-4c24-bfc5-d632796b63fe"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
