# OnRspQryRCAMSInterParameter

OnRspQryRCAMSInterParameter

请求RCAMS跨品种风险折抵参数查询响应，当执行[ReqQryRCAMSInterParameter](../CTHOSTFTDCTRADERAPI/REQQRYRCAMSINTERPARAMETER.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryRCAMSInterParameter(CThostFtdcRCAMSInterParameterField *pRCAMSInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pRCAMSInterParameter：RCAMS跨品种风险折抵参数

```
struct CThostFtdcRCAMSInterParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///商品群代码
    TThostFtdcProductIDType ProductGroupID;
    ///优先级
    TThostFtdcRCAMSPriorityType Priority;
    ///折抵率
    TThostFtdcHedgeRateType CreditRate;
    ///产品组合代码1
    TThostFtdcProductIDType CombProduct1;
    ///产品组合代码2
    TThostFtdcProductIDType CombProduct2;
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
