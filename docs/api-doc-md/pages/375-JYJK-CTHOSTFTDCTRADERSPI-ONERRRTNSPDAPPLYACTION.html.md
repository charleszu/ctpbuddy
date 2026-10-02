# OnErrRtnSpdApplyAction

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnErrRtnSpdApplyAction<a id="content"></a>

<a id="left_menu"></a>

  ** **

套利确认撤销通知,当执行[ReqSpdApplyAction](pages/198-JYJK-CTHOSTFTDCTRADERAPI-REQSPDAPPLYACTION.html.md)报错时返回此接口
<a id="42295d63-599d-43c4-9eb9-abca78610ac1"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnErrRtnSpdApplyAction(CThostFtdcSpdApplyActionField *pSpdApplyAction, CThostFtdcRspInfoField *pRspInfo) {};

<a id="6beb23a7-4516-4e5c-9212-847ef41ef017"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pSpdApplyAction：套利申请撤销回报

```
struct CThostFtdcSpdApplyActionField
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

<a id="4214f8a1-db04-432f-b032-dbfe3c7468f5"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

<a id="74ce5988-2b90-4a7e-9f4d-d39698e14d0b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
