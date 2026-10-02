# OnRtnOffsetSetting

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRtnOffsetSetting<a id="content"></a>

<a id="left_menu"></a>

  ** **

对冲设置通知，当执行[ReqOffsetSetting](pages/190-JYJK-CTHOSTFTDCTRADERAPI-REQOFFSETSETTING.html.md) 、[ReqCancelOffsetSetting](pages/191-JYJK-CTHOSTFTDCTRADERAPI-REQCANCELOFFSETSETTING.html.md)后，该方法被调用。
<a id="87a676c4-03a4-4f4b-aecb-0ebc05775013"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRtnOffsetSetting(CThostFtdcOffsetSettingField *pOffsetSetting) {};

<a id="2b1a9b71-9a63-463c-ad15-fb98a1eb9db4"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pOffsetSetting：对冲设置

```
struct CThostFtdcOffsetSettingField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///标的期货合约代码
    TThostFtdcInstrumentIDType  UnderlyingInstrID;
    ///产品代码
    TThostFtdcProductIDType ProductID;
    ///对冲类型
    TThostFtdcOffsetTypeType    OffsetType;
    ///申请对冲的合约数量
    TThostFtdcVolumeType    Volume;
    ///是否对冲
    TThostFtdcBoolType  IsOffset;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///IP地址
    TThostFtdcIPAddressType IPAddress;
    ///Mac地址
    TThostFtdcMacAddressType    MacAddress;
    ///交易所合约代码
    TThostFtdcExchangeInstIDType    ExchangeInstID;
    ///交易所期权系列号
    TThostFtdcExchangeInstIDType    ExchangeSerialNo;
    ///交易所产品代码
    TThostFtdcProductIDType ExchangeProductID;
    ///会员代码
    TThostFtdcParticipantIDType ParticipantID;
    ///客户代码
    TThostFtdcClientIDType  ClientID;
    ///交易所交易员代码
    TThostFtdcTraderIDType  TraderID;
    ///安装编号
    TThostFtdcInstallIDType InstallID;
    ///对冲提交状态
    TThostFtdcOrderSubmitStatusType OrderSubmitStatus;
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///结算编号
    TThostFtdcSettlementIDType  SettlementID;
    ///报单日期
    TThostFtdcDateType  InsertDate;
    ///插入时间
    TThostFtdcTimeType  InsertTime;
    ///撤销时间
    TThostFtdcTimeType  CancelTime;
    ///对冲设置结果
    TThostFtdcExecResultType    ExecResult;
    ///序号
    TThostFtdcSequenceNoType    SequenceNo;
    ///前置编号
    TThostFtdcFrontIDType   FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///状态信息
    TThostFtdcErrorMsgType  StatusMsg;
    ///操作用户代码
    TThostFtdcUserIDType    ActiveUserID;
    ///经纪公司报单编号
    TThostFtdcSequenceNoType    BrokerOffsetSettingSeq;
    ///申请来源
    TThostFtdcApplySrcType  ApplySrc;
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

<a id="3cf66da0-a1a2-49cf-8c45-7a7af6db0283"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="0039ed15-d50b-4bb4-ac04-d0743f5cb2ea"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
