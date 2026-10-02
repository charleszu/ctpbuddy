# ReqAuthenticate

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqAuthenticate<a id="content"></a>

<a id="left_menu"></a>

  ** **

客户端认证请求，对应响应[OnRspAuthenticate](pages/223-JYJK-CTHOSTFTDCTRADERSPI-ONRSPAUTHENTICATE.html.md)。如果交易系统开启了强制终端认证，则必须认证通过后才能发起登陆；如果未开启，则不需要认证即可登陆，此时如果主动去认证，不管成功或失败，也不影响后续登陆。
<a id="11f16d3a-043b-458f-94f6-2edae569cac9"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqAuthenticate(CThostFtdcReqAuthenticateField *pReqAuthenticateField, int nRequestID) = 0;

<a id="87a7b8ed-1bf8-427c-9241-2e0869f2d401"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pReqAuthenticateField：客户端认证请求

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcProductInfoType | UserProductInfo | 用户端产品信息 | 无 |
| TThostFtdcAuthCodeType | AuthCode | 认证码 | 必填 |
| TThostFtdcAppIDType | AppID | App代码 | 必填 |

UserProductInfo：客户端的产品信息，如软件开发商、版本号等。

CTP后台用户事件中的用户登录事件所显示的用户端产品信息取自ReqAuthentication接口里的UserProductInfo，而非[ReqUserLogin](pages/039-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGIN.html.md)里的。

AuthCode：认证码需要向期货公司申请得到

AppID：**必填项**，不然认证失败，如果没有则要向期货公司申请，申请的AppID必须遵循监控中心规范格式

UserID：**必填项**，用户代码。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="408a3c11-42b6-4734-8fb8-4a5497dc148a"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="d5e6eb38-9089-4561-8935-47a31bd0ea35"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcReqAuthenticateField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.UserID, "1000001");
strcpy_s(a.UserProductInfo, "mytest");
strcpy_s(a.AuthCode, "MLX0LEA4L4UPUCBF");
strcpy_s(a.AppID, "mytest");
m_pUserApi->ReqAuthenticate(&a, nRequestID++);

```

<a id="d145acac-fbe4-493b-9a1b-04523adf3e8e"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

每次重连都需要做一遍客户端认证吗？<a id="region_panel_1"></a>

| 重新连接都需要重新认证一下。 |
|---|

<a id="region_tail_1"></a>

<a id="region_header_2"></a>

非穿透式版本，开通了强制认证，需要申请一个认证码的。穿透式版本，也需要一个穿透式的认证码。如果想同时开通非穿透式强制认证和穿透式认证，这样就要两个认证码；但API里是只需要一个认证码，那请问这种情况怎么解决？<a id="region_panel_2"></a>

| 这种场景是不存在的，因为穿透式和非穿透式版本的api互不兼容，所以不存在用一个api去同时做新旧前置的认证。 |
|---|

<a id="region_tail_2"></a>

<a id="region_header_3"></a>

在使用“看穿式监管信息采集评测工具”时候，发现获取到的记录数和数据库中信息的条目数不匹配。这是为什么？<a id="region_panel_3"></a>

| 数据库中的记录为乱码的话就不显示。 |
|---|

<a id="region_tail_3"></a>

<a id="region_header_4"></a>

采集信息库是否是线程安全的？能多个线程同时调用吗？<a id="region_panel_4"></a>

| 不是线程安全的，不能同时调用。 |
|---|

<a id="region_tail_4"></a>

<a id="region_header_5"></a>

ErrorID=4043, ErrorMsg-CTP:用户与客户端授权验证失败 请问这个报错的原因是什么？<a id="region_panel_5"></a>

| 认证使用的appid没有授权给当前投资者账号使用。需要柜台重新指定下。 |
|---|

<a id="region_tail_5"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
