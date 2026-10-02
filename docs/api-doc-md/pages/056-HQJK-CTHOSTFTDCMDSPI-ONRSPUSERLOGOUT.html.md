# OnRspUserLogout

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspUserLogout<a id="content"></a>

<a id="left_menu"></a>

  ** **

登出请求响应，当[ReqUserLogout](pages/040-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGOUT.html.md)后，该方法被调用。
<a id="628e355f-24e2-4128-bea7-00118966958a"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [OnRspUserLogout](pages/300-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERLOGOUT.html.md)(CThostFtdcUserLogoutField *pUserLogout, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="e51569e2-1085-45c2-a30f-8be38f5d9e0f"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pUserLogout：用户登出请求

```
struct CThostFtdcUserLogoutField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///用户代码
    TThostFtdcUserIDType UserID;
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

nRequestID：返回用户操作请求的ID，该ID由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="359160ab-3a04-43fc-b25a-a313feaa9bc5"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="9b865c9a-64f3-4b63-b987-6b797bc5649c"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
