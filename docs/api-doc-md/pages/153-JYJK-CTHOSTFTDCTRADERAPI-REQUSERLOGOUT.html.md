# ReqUserLogout

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqUserLogout<a id="content"></a>

<a id="left_menu"></a>

  ** **

登出请求，对应响应[OnRspUserLogout](pages/300-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERLOGOUT.html.md)。
<a id="93f35e6c-6285-48e7-8f97-43b25970ab54"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int [ReqUserLogout](pages/040-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGOUT.html.md)(CThostFtdcUserLogoutField *pUserLogout, int nRequestID) = 0;

<a id="cab26829-0ed6-426e-961d-c122ab47c6c3"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pUserLogout：用户登出请求

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="19ab9cde-955b-47d4-9641-7695e1b73b47"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="7a1d0b3b-c487-4a69-9960-7a99192b1764"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcUserLogoutField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.UserID, "1000001");
m_pUserApi->ReqUserLogout(&a, nRequestID++);

```

<a id="e50f748a-acda-4a08-ba9f-ca1cc68fd49f"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

用户调用[ReqUserLogout](pages/040-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGOUT.html.md)后是否会自动重连？<a id="region_panel_1"></a>

| 会，Logout后，触发OnFrontDisconnected，能自动OnFrontConnected。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
