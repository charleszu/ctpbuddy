# OnErrRtnCancelOffsetSetting

OnErrRtnCancelOffsetSetting

对冲设置撤销错误回报，当执行[ReqCancelOffsetSetting](../CTHOSTFTDCTRADERAPI/REQCANCELOFFSETSETTING.html)返回错误后，该方法被调用。

◇ 1. 函数原型

virtual void OnErrRtnCancelOffsetSetting(CThostFtdcCancelOffsetSettingField *pCancelOffsetSetting, CThostFtdcRspInfoField *pRspInfo) {};

◇ 2. 参数

pCancelOffsetSetting：撤销对冲设置

```
struct CThostFtdcCancelOffsetSettingField
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
    ///交易所交易员代码
    TThostFtdcTraderIDType  TraderID;
    ///安装编号
    TThostFtdcInstallIDType InstallID;
    ///会员代码
    TThostFtdcParticipantIDType ParticipantID;
    ///客户代码
    TThostFtdcClientIDType  ClientID;
    ///报单操作状态
    TThostFtdcOrderActionStatusType OrderActionStatus;
    ///状态信息
    TThostFtdcErrorMsgType  StatusMsg;
    ///操作本地编号
    TThostFtdcOrderLocalIDType  ActionLocalID;
    ///操作日期
    TThostFtdcDateType  ActionDate;
    ///操作时间
    TThostFtdcTimeType  ActionTime;
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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
