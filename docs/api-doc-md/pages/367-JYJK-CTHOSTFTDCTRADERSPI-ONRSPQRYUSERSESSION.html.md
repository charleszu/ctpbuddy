# OnRspQryUserSession

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryUserSession<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询用户会话响应，当执行[ReqQryUserSession](pages/195-JYJK-CTHOSTFTDCTRADERAPI-REQQRYUSERSESSION.html.md)后，该方法被调用。
<a id="9e1105ff-c6df-4c59-9565-659cd7eb75cd"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryUserSession(CThostFtdcUserSessionField *pUserSession, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="663a04bd-6735-40f5-accc-a1eb0b2fc5b8"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pUserSession：用户会话

```
struct CThostFtdcUserSessionField
{
    ///前置编号
    TThostFtdcFrontIDType   FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///登录日期
    TThostFtdcDateType  LoginDate;
    ///登录时间
    TThostFtdcTimeType  LoginTime;
    ///保留的无效字段
    TThostFtdcOldIPAddressType  reserve1;
    ///用户端产品信息
    TThostFtdcProductInfoType   UserProductInfo;
    ///接口端产品信息
    TThostFtdcProductInfoType   InterfaceProductInfo;
    ///协议信息
    TThostFtdcProtocolInfoType  ProtocolInfo;
    ///Mac地址
    TThostFtdcMacAddressType    MacAddress;
    ///登录备注
    TThostFtdcLoginRemarkType   LoginRemark;
    ///IP地址
    TThostFtdcIPAddressType IPAddress;
};

```

pRspInfo：响应信息

```
struct CThostFtdcRspInfoField
{
    ///错误代码
    TThostFtdcErrorIDType   ErrorID;
    ///错误信息
    TThostFtdcErrorMsgType  ErrorMsg;
};

```

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="319057e3-c4e5-403c-a6a7-9bf537ad1c65"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="94fed83f-15cf-4e02-a869-8b7f6c95d4b2"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
