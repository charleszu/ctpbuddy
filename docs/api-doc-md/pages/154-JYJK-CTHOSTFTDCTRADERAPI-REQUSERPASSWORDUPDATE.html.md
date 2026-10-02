# ReqUserPasswordUpdate

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqUserPasswordUpdate<a id="content"></a>

<a id="left_menu"></a>

  ** **

用户口令更新请求，对应响应[OnRspUserPasswordUpdate](pages/301-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERPASSWORDUPDATE.html.md)。
<a id="a580e4eb-ca31-4149-9574-6d030899a203"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqUserPasswordUpdate(CThostFtdcUserPasswordUpdateField *pUserPasswordUpdate, int nRequestID) = 0;

<a id="da4e18e6-0690-45a8-a114-74855c6c8971"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pUserPasswordUpdate：用户口令变更

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcPasswordType | OldPassword | 原来的口令 | 必填 |
| TThostFtdcPasswordType | NewPassword | 新的口令 | 必填 |

NewPassword：新密码需要符合复杂度要求，不然修改失败。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="dc8f4958-b3d3-45ce-ba42-94a8d224429c"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="f82e0805-2464-460d-9fe3-058d059f1db3"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcUserPasswordUpdateField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.UserID, "1000001");
strcpy_s(a.OldPassword, "123456");
strcpy_s(a.NewPassword, "666666");
m_pUserApi->ReqUserPasswordUpdate(&a, nRequestID++);

```

<a id="36f082d3-e1fb-450d-994c-5c2f64310dcb"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
