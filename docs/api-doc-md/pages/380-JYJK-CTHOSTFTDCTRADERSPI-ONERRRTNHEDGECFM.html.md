# OnErrRtnHedgeCfm

OnErrRtnHedgeCfm

套保额度录入错误回报,当执行[ReqHedgeCfm](../CTHOSTFTDCTRADERAPI/REQHEDGECFM.html)后，如果返回错误，通过此接口返回

◇ 1. 函数原型

virtual void OnErrRtnHedgeCfm(CThostFtdcInputHedgeCfmField *pInputHedgeCfm, CThostFtdcRspInfoField *pRspInfo) {};

◇ 2. 参数

pInputHedgeCfm：套保确认输入基本信息

```
struct CThostFtdcInputHedgeCfmField
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
    TThostFtdcInstrumentIDType  InstrumentID;
    ///数量
    TThostFtdcVolumeType    Volume;
    ///买卖方向
    TThostFtdcDirectionType Direction;
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

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

◇ 3. 返回

◇ 4. FAQ

无
