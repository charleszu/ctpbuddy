# OnRtnSpdApply

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRtnSpdApply<a id="content"></a>

<a id="left_menu"></a>

  ** **

套利确认通知,当执行[ReqSpdApply](pages/197-JYJK-CTHOSTFTDCTRADERAPI-REQSPDAPPLY.html.md)后，返回Rtn通知
<a id="be1b389f-4eb6-4f70-add1-1c3fc832ddc6"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRtnSpdApply(CThostFtdcSpdApplyField *pSpdApply) {};

<a id="818765d2-aa2a-4d3d-8992-34c839114c93"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pSpdApply：套利申请回报

```
struct CThostFtdcSpdApplyField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///合约代码
    TThostFtdcInstrumentIDType  FirstLegInstrumentID;
    ///合约代码
    TThostFtdcInstrumentIDType  SecondLegInstrumentID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///数量
    TThostFtdcVolumeType    Volume;
    ///买卖方向
    TThostFtdcDirectionType Direction;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///前置编号
    TThostFtdcFrontIDType   FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///报单引用
    TThostFtdcOrderRefType  OrderRef;
    ///操作用户代码
    TThostFtdcUserIDType    ActiveUserID;
    ///经纪公司报单编号
    TThostFtdcSequenceNoType    BrokerOrderSeq;
    ///报单编号
    TThostFtdcOrderSysIDType    OrderSysID;
    ///申请状态
    TThostFtdcApplyStatusType   ApplyStatus;
    ///序号
    TThostFtdcSequenceNoType    SequenceNo;
    ///报单日期
    TThostFtdcDateType  InsertDate;
    ///委托时间
    TThostFtdcTimeType  InsertTime;
    ///撤销时间
    TThostFtdcTimeType  CancelTime;
    ///本地报单编号
    TThostFtdcOrderLocalIDType  OrderLocalID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///会员代码
    TThostFtdcParticipantIDType ParticipantID;
    ///客户代码
    TThostFtdcClientIDType  ClientID;
    ///合约在交易所的代码
    TThostFtdcExchangeInstIDType    ExchangeInstID;
    ///交易所交易员代码
    TThostFtdcTraderIDType  TraderID;
    ///安装编号
    TThostFtdcInstallIDType InstallID;
    ///报单提交状态
    TThostFtdcOrderSubmitStatusType OrderSubmitStatus;
    ///报单提示序号
    TThostFtdcSequenceNoType    NotifySequence;
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///结算编号
    TThostFtdcSettlementIDType  SettlementID;
    ///IP地址
    TThostFtdcIPAddressType IPAddress;
    ///Mac地址
    TThostFtdcMacAddressType    MacAddress;
    ///组合定单类型
    TThostFtdcCmbTypeType   CmbType;
    ///状态信息
    TThostFtdcErrorMsgType  StatusMsg;
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

<a id="acb3e37f-ff1e-4193-ba5f-bb8a9a73f65b"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

<a id="d4799854-bb2f-46d6-bf75-adfd0ca4d0ad"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
