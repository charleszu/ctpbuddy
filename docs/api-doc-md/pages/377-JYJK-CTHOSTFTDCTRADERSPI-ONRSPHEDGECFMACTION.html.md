# OnRspHedgeCfmAction

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspHedgeCfmAction<a id="content"></a>

<a id="left_menu"></a>

  ** **

套保确认撤销回复,当执行[ReqHedgeCfmAction](pages/201-JYJK-CTHOSTFTDCTRADERAPI-REQHEDGECFMACTION.html.md)返回错误后，返回此接口
<a id="e7eb1a0e-3cea-4bdd-a3ea-4983a3ff70b6"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspHedgeCfmAction(CThostFtdcInputHedgeCfmActionField *pInputHedgeCfmAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="117ed027-600b-429f-9aae-911afb8f75ca"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputHedgeCfmAction：套保申请撤销

```
struct CThostFtdcInputHedgeCfmActionField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合同编号
    TThostFtdcOrderSysIDType    OrderSysID;
    ///报单引用
    TThostFtdcOrderRefType  OrderRef;
    ///前置编号
    TThostFtdcFrontIDType   FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///IP地址
    TThostFtdcIPAddressType IPAddress;
    ///Mac地址
    TThostFtdcMacAddressType    MacAddress;
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

<a id="ffef69f6-c4c2-41a6-bea4-d9f7b8038558"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

<a id="274fd5cd-17d7-4332-88a2-87af1f615c6a"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
