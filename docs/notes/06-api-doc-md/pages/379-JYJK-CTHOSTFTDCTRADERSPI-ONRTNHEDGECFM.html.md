# OnRtnHedgeCfm

OnRtnHedgeCfm

套保确认撤销回复,当执行[ReqHedgeCfm](../CTHOSTFTDCTRADERAPI/REQHEDGECFM.html)后，正确返回套保确认通知

◇ 1. 函数原型

virtual void OnRtnHedgeCfm(CThostFtdcHedgeCfmField *pHedgeCfm) {};

◇ 2. 参数

pHedgeCfm：套保申请回报

```
struct CThostFtdcHedgeCfmField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
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
    ///成功处理数量
    TThostFtdcVolumeType    DealVolume;
    ///报单日期
    TThostFtdcDateType  InsertDate;
    ///委托时间
    TThostFtdcTimeType  InsertTime;
    ///撤销时间
    TThostFtdcTimeType  CancelTime;
    ///日期
    TThostFtdcDateType  ReqDate;
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
    ///状态信息
    TThostFtdcErrorMsgType  StatusMsg;
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

◇ 3. 返回

◇ 4. FAQ

无
