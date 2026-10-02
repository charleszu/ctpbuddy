# OnErrRtnHedgeCfmAction

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnErrRtnHedgeCfmAction<a id="content"></a>

<a id="left_menu"></a>

  ** **

套保确认撤销通知,当执行[ReqHedgeCfmAction](pages/201-JYJK-CTHOSTFTDCTRADERAPI-REQHEDGECFMACTION.html.md)后，如果返回错误，通过此接口返回
<a id="1d4d8c6a-7d55-4b2a-8af9-6749868cd010"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnErrRtnHedgeCfmAction(CThostFtdcHedgeCfmActionField *pHedgeCfmAction, CThostFtdcRspInfoField *pRspInfo) {};

<a id="ca0250cc-1697-43c5-919a-4ddb15e86879"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pHedgeCfmAction：套保申请撤销回报

```
struct CThostFtdcHedgeCfmActionField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///操作日期
    TThostFtdcDateType  ActionDate;
    ///操作时间
    TThostFtdcTimeType  ActionTime;
    ///交易所交易员代码
    TThostFtdcTraderIDType  TraderID;
    ///安装编号
    TThostFtdcInstallIDType InstallID;
    ///本地报单编号
    TThostFtdcOrderLocalIDType  OrderLocalID;
    ///操作本地编号
    TThostFtdcOrderLocalIDType  ActionLocalID;
    ///会员代码
    TThostFtdcParticipantIDType ParticipantID;
    ///客户代码
    TThostFtdcClientIDType  ClientID;
    ///报单操作状态
    TThostFtdcOrderActionStatusType OrderActionStatus;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合同编号
    TThostFtdcOrderSysIDType    OrderSysID;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///状态信息
    TThostFtdcErrorMsgType  StatusMsg;
    ///报单引用
    TThostFtdcOrderRefType  OrderRef;
    ///前置编号
    TThostFtdcFrontIDType   FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///IP地址
    TThostFtdcIPAddressType IPAddress;
    ///Mac地址
    TThostFtdcMacAddressType    MacAddress;
};

```

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="091d3182-43a1-4dab-af25-484a539364b1"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

<a id="434b251e-a2fc-4adf-969a-289256d58232"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
