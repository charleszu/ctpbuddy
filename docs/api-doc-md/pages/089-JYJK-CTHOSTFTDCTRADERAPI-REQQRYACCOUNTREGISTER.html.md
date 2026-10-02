# ReqQryAccountregister

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryAccountregister<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询银期签约关系

响应: [OnRspQryAccountregister](pages/240-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYACCOUNTREGISTER.html.md)
<a id="5ab9d260-337c-43dd-958c-f81dfd6a94b8"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryAccountregister(CThostFtdcQryAccountregisterField *pQryAccountregister, int nRequestID) = 0;

<a id="510315ab-eb19-4e9a-9e26-c800c05af3a5"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryAccountregister：请求查询银期签约关系

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 否 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 否 |
| TThostFtdcBankIDType | BankID | 银行代码 | 否 |
| TThostFtdcBankBrchIDType | BankBranchID | 银行分支机构代码 | 否 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="e084fd25-4dd1-4267-bfde-9c8212d6dad4"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="69bd015b-c21c-4145-9e75-68036b19f3db"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryAccountregisterField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.AccountID, "1000001");
strcpy_s(a.BankID, "1");
strcpy_s(a.CurrencyID, "CNY");
m_pUserApi->ReqQryAccountregister(&a, 1);

```

<a id="99e0966c-135f-4492-8b48-0c49ec5ad59b"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

查询无结果是什么原因？<a id="region_panel_1"></a>

| 只有和银行签约成功后，并且银行的签约信息返回CTP柜台后才能查询到信息。对应柜台菜单【银期转账账户信息查询】 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
