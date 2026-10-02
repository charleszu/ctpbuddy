# OnRspQryRULEInstrParameter

OnRspQryRULEInstrParameter

请求RULE合约保证金参数查询响应，当执行[ReqQryRULEInstrParameter](../CTHOSTFTDCTRADERAPI/REQQRYRULEINSTRPARAMETER.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryRULEInstrParameter(CThostFtdcRULEInstrParameterField *pRULEInstrParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pRULEInstrParameter：RULE合约保证金参数

```
struct CThostFtdcRULEInstrParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///合约类型
    TThostFtdcInstrumentClassType   InstrumentClass;
    ///标准合约
    TThostFtdcInstrumentIDType  StdInstrumentID;
    ///投机买折算系数
    TThostFtdcRatioType BSpecRatio;
    ///投机卖折算系数
    TThostFtdcRatioType SSpecRatio;
    ///套保买折算系数
    TThostFtdcRatioType BHedgeRatio;
    ///套保卖折算系数
    TThostFtdcRatioType SHedgeRatio;
    ///买附加风险保证金
    TThostFtdcMoneyType BAddOnMargin;
    ///卖附加风险保证金
    TThostFtdcMoneyType SAddOnMargin;
    ///商品群号
    TThostFtdcCommodityGroupIDType  CommodityGroupID;
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
