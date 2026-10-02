# OnRspQrySPBMOptionParameter

OnRspQrySPBMOptionParameter

请求SPBM期权合约参数查询响应，当执行[ReqQrySPBMOptionParameter](../CTHOSTFTDCTRADERAPI/REQQRYSPBMOPTIONPARAMETER.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQrySPBMOptionParameter(CThostFtdcSPBMOptionParameterField *pSPBMOptionParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pSPBMOptionParameter：SPBM期权合约保证金参数

```
struct CThostFtdcSPBMOptionParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///品种代码
    TThostFtdcInstrumentIDType  ProdFamilyCode;
    ///期权合约因子
    TThostFtdcVolumeMultipleType    Cvf;
    ///期权冲抵价格
    TThostFtdcPriceType DownPrice;
    ///Delta值
    TThostFtdcDeltaType Delta;
    ///卖方期权风险转换最低值
    TThostFtdcDeltaType SlimiDelta;
    ///昨结算价
    TThostFtdcPriceType PreSettlementPrice;
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
