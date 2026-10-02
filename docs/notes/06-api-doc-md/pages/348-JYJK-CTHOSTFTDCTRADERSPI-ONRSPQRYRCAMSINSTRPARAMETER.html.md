# OnRspQryRCAMSInstrParameter

OnRspQryRCAMSInstrParameter

请求RCAMS同合约风险对冲参数查询响应，当执行[ReqQryRCAMSInstrParameter](../CTHOSTFTDCTRADERAPI/REQQRYRCAMSINSTRPARAMETER.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryRCAMSInstrParameter(CThostFtdcRCAMSInstrParameterField *pRCAMSInstrParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pRCAMSInstrParameter：RCAMS同合约风险对冲参数

```
struct CThostFtdcRCAMSInstrParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///产品代码
    TThostFtdcProductIDType ProductID;
    ///同合约风险对冲比率
    TThostFtdcHedgeRateType HedgeRate;
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
