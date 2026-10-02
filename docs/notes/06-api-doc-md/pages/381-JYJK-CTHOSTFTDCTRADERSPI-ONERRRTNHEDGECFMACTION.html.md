# OnErrRtnHedgeCfmAction

OnErrRtnHedgeCfmAction

套保确认撤销通知,当执行[ReqHedgeCfmAction](../CTHOSTFTDCTRADERAPI/REQHEDGECFMACTION.html)后，如果返回错误，通过此接口返回

◇ 1. 函数原型

virtual void OnErrRtnHedgeCfmAction(CThostFtdcHedgeCfmActionField *pHedgeCfmAction, CThostFtdcRspInfoField *pRspInfo) {};

◇ 2. 参数

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

◇ 3. 返回

◇ 4. FAQ

无
