# OnRspQryRULEIntraParameter

OnRspQryRULEIntraParameter

请求RULE品种内对锁仓折扣参数查询响应，当执行[ReqQryRULEIntraParameter](../CTHOSTFTDCTRADERAPI/REQQRYRULEINTRAPARAMETER.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryRULEIntraParameter(CThostFtdcRULEIntraParameterField *pRULEIntraParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pRULEIntraParameter：RULE品种内对锁仓折扣参数

```
struct CThostFtdcRULEIntraParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///品种代码
    TThostFtdcInstrumentIDType  ProdFamilyCode;
    ///标准合约
    TThostFtdcInstrumentIDType  StdInstrumentID;
    ///标准合约保证金
    TThostFtdcMoneyType StdInstrMargin;
    ///一般月份合约组合保证金系数
    TThostFtdcRatioType UsualIntraRate;
    ///临近交割合约组合保证金系数
    TThostFtdcRatioType DeliveryIntraRate;
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
