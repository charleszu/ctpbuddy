# OnErrRtnOrderAction

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnErrRtnOrderAction<a id="content"></a>

<a id="left_menu"></a>

  ** **

报单操作错误回报，当执行[ReqOrderAction](pages/085-JYJK-CTHOSTFTDCTRADERAPI-REQORDERACTION.html.md)后有字段填写不对之类的CTP报错则通过此接口返回
<a id="b72f6878-c2b1-411f-ac36-181f91fa74a3"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnErrRtnOrderAction(CThostFtdcOrderActionField *pOrderAction, CThostFtdcRspInfoField *pRspInfo) {};

<a id="d08a0d5b-50eb-4d02-93fe-b673b680bb87"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pOrderAction：报单操作

```
struct CThostFtdcOrderActionField
{
///经纪公司代码
TThostFtdcBrokerIDType  BrokerID;
///投资者代码
TThostFtdcInvestorIDType    InvestorID;
///报单操作引用
TThostFtdcOrderActionRefType    OrderActionRef;
///报单引用
TThostFtdcOrderRefType  OrderRef;
///请求编号
TThostFtdcRequestIDType RequestID;
///前置编号
TThostFtdcFrontIDType   FrontID;
///会话编号
TThostFtdcSessionIDType SessionID;
///交易所代码
TThostFtdcExchangeIDType    ExchangeID;
///报单编号
TThostFtdcOrderSysIDType    OrderSysID;
///操作标志
TThostFtdcActionFlagType    ActionFlag;
///价格
TThostFtdcPriceType LimitPrice;
///数量变化
TThostFtdcVolumeType    VolumeChange;
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
///业务单元
TThostFtdcBusinessUnitType  BusinessUnit;
///报单操作状态
TThostFtdcOrderActionStatusType OrderActionStatus;
///用户代码
TThostFtdcUserIDType    UserID;
///状态信息
TThostFtdcErrorMsgType  StatusMsg;
///保留的无效字段
TThostFtdcOldInstrumentIDType   reserve1;
///营业部编号
TThostFtdcBranchIDType  BranchID;
///投资单元代码
TThostFtdcInvestUnitIDType  InvestUnitID;
///保留的无效字段
TThostFtdcOldIPAddressType  reserve2;
///Mac地址
TThostFtdcMacAddressType    MacAddress;
///合约代码
TThostFtdcInstrumentIDType  InstrumentID;
///IP地址
TThostFtdcIPAddressType IPAddress;
///报单回显字段
TThostFtdcOrderMemoType OrderMemo;
///session上请求计数 api自动维护
TThostFtdcSequenceNo12Type  SessionReqSeq;
};

```

OrderSysID：交易所返回的报单编号

ActionFlag：撤单标志，现在只支持删除，不支持修改

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

<a id="524fff90-ed31-462e-adb3-b04614274722"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="158c005c-8bcd-4964-8c0e-d5aabeda9a60"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
