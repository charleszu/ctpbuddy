# ReqGenSMSCode

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqGenSMSCode<a id="content"></a>

<a id="left_menu"></a>

  ** **

申请短信验证码请求，响应[OnRspGenSMSCode](pages/369-JYJK-CTHOSTFTDCTRADERSPI-ONRSPGENSMSCODE.html.md)
<a id="5086a2b0-00b3-4616-9657-71a30e64571c"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqGenSMSCode(CThostFtdcReqGenSMSCodeField *pReqGenSMSCode, int nRequestID) = 0;

<a id="89fa059e-7fe4-4568-9239-5f1b4123edf8"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pReqGenSMSCode：申请短信验证码请求

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcUserIDType | UserID | 用户代码 | 是 |
| TThostFtdcSMSPhoneType | Mobile | 手机号 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="08d9437d-2138-4cff-96cf-aa612352e644"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="10c4f957-6318-4bb0-8fde-faa5f28f5581"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="199c33c6-1c0c-48e2-aad9-a74f8adeea7c"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
