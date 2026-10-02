# ReqQryContractBank

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryContractBank<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询签约银行

响应: [OnRspQryContractBank](pages/246-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCONTRACTBANK.html.md)
<a id="7bb2a8d4-2585-4643-a1bd-237f6890360d"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryContractBank(CThostFtdcQryContractBankField *pQryContractBank, int nRequestID) = 0;

<a id="cf35fd7b-6cf1-4055-b086-24cfe8d652a9"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryContractBank：查询签约银行请求

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcBankIDType | BankID | 银行代码 | 是 |
| TThostFtdcBankBrchIDType | BankBrchID | 银行分中心代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="78c90fd2-c0ce-46e6-b736-b5cda562e59f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="846336bd-0c8c-43f6-8eb2-9ff03ce6894d"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryContractBankField a = { 0 };
strcpy_s(a.BrokerID, "9999");
m_pUserApi->ReqQryContractBank(&a, nRequestID++);

```

<a id="00b1763d-2a59-4976-a8cf-0766c23713f8"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
