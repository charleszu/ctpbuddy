# OnRspGenSMSCode

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspGenSMSCode<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求申请短信验证码响应,当执行[ReqGenSMSCode](pages/196-JYJK-CTHOSTFTDCTRADERAPI-REQGENSMSCODE.html.md)后，该方法被调用。
<a id="65bf6501-da75-4c8e-8464-d0389c0c7fc0"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspGenSMSCode(CThostFtdcRspGenSMSCodeField *pRspGenSMSCode, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="a767ed83-66f4-4f24-881e-716d50fc888b"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRspGenSMSCode：申请短信验证码响应

```
struct CThostFtdcRspGenSMSCodeField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///生成时间
    TThostFtdcTimeType  GenTime;
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

<a id="8f5e6e80-6682-4579-a7d6-80d2eff59ac3"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

<a id="aa38eef5-f5c1-40f7-9994-2cdfe1271e74"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
