# ReqTradingAccountPasswordUpdate

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqTradingAccountPasswordUpdate<a id="content"></a>

<a id="left_menu"></a>

  ** **

资金账户口令更新请求，对应响应[OnRspTradingAccountPasswordUpdate](pages/297-JYJK-CTHOSTFTDCTRADERSPI-ONRSPTRADINGACCOUNTPASSWORDUPDATE.html.md)。
<a id="38f49675-6872-42af-b335-f9c7a296830c"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqTradingAccountPasswordUpdate(CThostFtdcTradingAccountPasswordUpdateField *pTradingAccountPasswordUpdate, int nRequestID) = 0;

<a id="0caa8c2a-4a33-4288-830f-518af9e4b86c"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pTradingAccountPasswordUpdate：资金账户口令变更域

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 必填 |
| TThostFtdcPasswordType | OldPassword | 原来的口令 | 必填 |
| TThostFtdcPasswordType | NewPassword | 新的口令 | 必填 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 必填 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="47160ae7-64ed-4adb-839c-01e06f70bc32"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="d54b34b7-663d-4bdf-bb2f-3cc073166000"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcTradingAccountPasswordUpdateField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.AccountID, "1000001");
strcpy_s(a.OldPassword, "123456");
strcpy_s(a.NewPassword, "666666");
strcpy_s(a.CurrencyID, "CNY");
m_pUserApi->ReqTradingAccountPasswordUpdate(&a, nRequestID++);

```

<a id="3f737448-2fc8-48e0-aff0-9357f3ef2598"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
