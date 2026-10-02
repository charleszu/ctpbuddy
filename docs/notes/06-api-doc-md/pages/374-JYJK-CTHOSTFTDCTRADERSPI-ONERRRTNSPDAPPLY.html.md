# OnErrRtnSpdApply

OnErrRtnSpdApply

套利申请录入错误回报，当执行[ReqSpdApply](../CTHOSTFTDCTRADERAPI/REQSPDAPPLY.html)报错时返回此接口

◇ 1. 函数原型

virtual void OnErrRtnSpdApply(CThostFtdcInputSpdApplyField *pInputSpdApply, CThostFtdcRspInfoField *pRspInfo) {};

◇ 2. 参数

pInputSpdApply：套利确认输入基本信息

```
struct CThostFtdcInputSpdApplyField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合约代码
    TThostFtdcInstrumentIDType  FirstLegInstrumentID;
    ///合约代码
    TThostFtdcInstrumentIDType  SecondLegInstrumentID;
    ///数量
    TThostFtdcVolumeType    Volume;
    ///买卖方向
    TThostFtdcDirectionType Direction;
    ///组合定单类型
    TThostFtdcCmbTypeType   CmbType;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///报单引用
    TThostFtdcOrderRefType  OrderRef;
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
