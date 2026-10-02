# ReqUserLogin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqUserLogin<a id="content"></a>

<a id="left_menu"></a>

  ** **

用户登录请求，对应响应[OnRspUserLogin](pages/299-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERLOGIN.html.md)。目前行情登陆不校验账号密码。
<a id="66597bee-1f51-4d40-aad3-9f7757f80926"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1.特别说明
<a id="panel1"></a>

自CTP交易系统升级6.6.2版本后，后台支持对用户登录行情前置进行身份校验。

若启用该功能后，登录行情前置时要求当前交易日该IP已成功登录过交易系统，且发起登录行情的请求中必须正确填写BrokerID和UserID，与登录交易的信息保持一致。

不填、填错或该IP未成功登录过交易系统，则校验不通过，会提示“CTP:不合法登录”；

若不启用，则无需验证，可直接发起登录。

关于[行情流控](pages/403-QTYWGZ-HQLK.html.md)详见[行情流控](pages/403-QTYWGZ-HQLK.html.md)

<a id="c73fe118-f925-4595-b8ab-cfd68ab73015"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 函数原型
<a id="panel2"></a>

virtual int ReqUserLogin(CThostFtdcReqUserLoginField *pReqUserLoginField, int nRequestID) = 0;

<a id="f242ea28-4a1f-4bd8-8d4e-f18ed8a16b2b"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 参数
<a id="panel3"></a>

pReqUserLoginField：用户登录请求

```
struct CThostFtdcReqUserLoginField
{
    ///交易日
    TThostFtdcDateType TradingDay;
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///用户代码
    TThostFtdcUserIDType UserID;
    ///密码
    TThostFtdcPasswordType Password;
    ///用户端产品信息
    TThostFtdcProductInfoType UserProductInfo;
    ///接口端产品信息
    TThostFtdcProductInfoType InterfaceProductInfo;
    ///协议信息
    TThostFtdcProtocolInfoType ProtocolInfo;
    ///Mac地址
    TThostFtdcMacAddressType MacAddress;
    ///动态密码
    TThostFtdcPasswordType OneTimePassword;
    ///终端IP地址
    TThostFtdcIPAddressType ClientIPAddress;
    ///登录备注
    TThostFtdcLoginRemarkType LoginRemark;
};

```

BrokerID:开启行情身份校验功能后，该字段必需正确填写

UserID：操作员代码，后续请求中的investorid需要属于该操作员的组织架构下；开启行情身份校验功能后，该字段必需正确填写

UserProductInfo：客户端的产品信息，如软件开发商、版本号等，

CTP后台用户事件中的用户登录事件所显示的用户端产品信息取自ReqAuthentication接口里的UserProductInfo，而非ReqUserLogin里的。

LoginRemark：可以写登录备注，能够被交易系统的日志查询到。

IPAddress：系统自动获取，填写无效。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="f3d9c740-1f16-4931-9a74-b3d71f1e8c3f"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 返回
<a id="panel4"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="21f13bf4-d7c4-4db1-8e38-1d3961d761f9"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. 调用示例
<a id="panel5"></a>

```
CThostFtdcReqUserLoginField reqUserLogin = {0};
m_pUserMdApi->ReqUserLogin(&reqUserLogin, nRequestID++);

```

<a id="297da93a-5827-4111-95d7-6a4c08b8d383"></a><a id="title6"></a>

<a id="header_span6"></a>◇ 6. FAQ
<a id="panel6"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
