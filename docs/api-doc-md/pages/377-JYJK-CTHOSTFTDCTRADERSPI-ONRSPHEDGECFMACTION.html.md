# OnRspHedgeCfmAction

OnRspHedgeCfmAction

套保确认撤销回复,当执行[ReqHedgeCfmAction](../CTHOSTFTDCTRADERAPI/REQHEDGECFMACTION.html)返回错误后，返回此接口

◇ 1. 函数原型

virtual void OnRspHedgeCfmAction(CThostFtdcInputHedgeCfmActionField *pInputHedgeCfmAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInputHedgeCfmAction：套保申请撤销

```
struct CThostFtdcInputHedgeCfmActionField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合同编号
    TThostFtdcOrderSysIDType    OrderSysID;
    ///报单引用
    TThostFtdcOrderRefType  OrderRef;
    ///前置编号
    TThostFtdcFrontIDType   FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
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
