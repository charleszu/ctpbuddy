# OnErrRtnExecOrderAction

OnErrRtnExecOrderAction

执行宣告操作错误回报，当执行[ReqExecOrderAction](../CTHOSTFTDCTRADERAPI/REQEXECORDERACTION.html)后有字段填写不对之类的CTP报错则通过此接口返回

◇ 1. 函数原型

virtual void OnErrRtnExecOrderAction(CThostFtdcExecOrderActionField *pExecOrderAction, CThostFtdcRspInfoField *pRspInfo) {};

◇ 2. 参数

pExecOrderAction：执行宣告操作

```
struct CThostFtdcExecOrderActionField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///执行宣告操作引用
    TThostFtdcOrderActionRefType    ExecOrderActionRef;
    ///执行宣告引用
    TThostFtdcOrderRefType  ExecOrderRef;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///前置编号
    TThostFtdcFrontIDType   FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///执行宣告操作编号
    TThostFtdcExecOrderSysIDType    ExecOrderSysID;
    ///操作标志
    TThostFtdcActionFlagType    ActionFlag;
    ///操作日期
    TThostFtdcDateType  ActionDate;
    ///操作时间
    TThostFtdcTimeType  ActionTime;
    ///交易所交易员代码
    TThostFtdcTraderIDType  TraderID;
    ///安装编号
    TThostFtdcInstallIDType InstallID;
    ///本地执行宣告编号
    TThostFtdcOrderLocalIDType  ExecOrderLocalID;
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
    ///执行类型
    TThostFtdcActionTypeType    ActionType;
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
};

```

ActionFlag：支持删除操作，不支持修改

InstallID：CTP内部使用

OrderActionStatus：报单的状态

StatusMsg：错误的信息

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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
