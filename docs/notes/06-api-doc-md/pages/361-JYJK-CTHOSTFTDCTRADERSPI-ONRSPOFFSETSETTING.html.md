# OnRspOffsetSetting

OnRspOffsetSetting

对冲设置请求响应，当执行[ReqOffsetSetting](../CTHOSTFTDCTRADERAPI/REQOFFSETSETTING.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspOffsetSetting(CThostFtdcInputOffsetSettingField *pInputOffsetSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInputOffsetSetting：输入的对冲设置

```
struct CThostFtdcInputOffsetSettingField
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

当查询无记录时，指针返回为null

◇ 4. FAQ

无
