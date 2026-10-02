# OnRspQrySPBMInvestorPortfDef

OnRspQrySPBMInvestorPortfDef

请求投资者SPBM套餐选择查询响应，当执行[ReqQrySPBMInvestorPortfDef](../CTHOSTFTDCTRADERAPI/REQQRYSPBMINVESTORPORTFDEF.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQrySPBMInvestorPortfDef(CThostFtdcSPBMInvestorPortfDefField *pSPBMInvestorPortfDef, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pSPBMInvestorPortfDef：投资者套餐选择

```
struct CThostFtdcSPBMInvestorPortfDefField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///组合保证金套餐代码
    TThostFtdcPortfolioDefIDType    PortfolioDefID;
};

```

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

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
