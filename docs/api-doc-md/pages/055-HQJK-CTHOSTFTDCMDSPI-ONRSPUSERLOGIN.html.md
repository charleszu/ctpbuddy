# OnRspUserLogin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspUserLogin<a id="content"></a>

<a id="left_menu"></a>

  ** **

登录请求响应，当[ReqUserLogin](pages/039-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGIN.html.md)后，该方法被调用。

关于[行情流控](pages/403-QTYWGZ-HQLK.html.md)详见[行情流控](pages/403-QTYWGZ-HQLK.html.md)
<a id="2ee726d1-7fd8-40c9-a5c2-4830d333278b"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [OnRspUserLogin](pages/299-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERLOGIN.html.md)(CThostFtdcRspUserLoginField *pRspUserLogin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="707a28e0-8fe4-4944-9168-ae1cef7323f6"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRspUserLogin：用户登录应答

```
struct CThostFtdcRspUserLoginField
{
    ///交易日
    TThostFtdcDateType TradingDay;
    ///登录成功时间
    TThostFtdcTimeType LoginTime;
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///用户代码
    TThostFtdcUserIDType UserID;
    ///交易系统名称
    TThostFtdcSystemNameType SystemName;
    ///前置编号
    TThostFtdcFrontIDType FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///最大报单引用
    TThostFtdcOrderRefType MaxOrderRef;
    ///上期所时间
    TThostFtdcTimeType SHFETime;
    ///大商所时间
    TThostFtdcTimeType DCETime;
    ///郑商所时间
    TThostFtdcTimeType CZCETime;
    ///中金所时间
    TThostFtdcTimeType FFEXTime;
    ///能源中心时间
    TThostFtdcTimeType INETime;
    ///后台版本信息
    TThostFtdcSysVersionType    SysVersion;
    ///广期所时间
    TThostFtdcTimeType  GFEXTime;
    ///当前登录中心号
    TThostFtdcDRIdentityIDType  LoginDRIdentityID;
    ///用户所属中心号
    TThostFtdcDRIdentityIDType  UserDRIdentityID;
};

```

pRspInfo：响应信息

```
struct CThostFtdcRspInfoField
{
    ///错误代码
    TThostFtdcErrorIDType ErrorID;
    ///错误信息
    TThostFtdcErrorMsgType ErrorMsg;
};

```

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="3ca04b0b-a0f9-4cf3-a042-653696b9e6fe"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="d982b1b7-c9b7-498d-9c74-12606756d078"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
