# OnRspQryRULEInterParameter

OnRspQryRULEInterParameter

请求RULE跨品种抵扣参数查询响应，当执行[ReqQryRULEInterParameter](../CTHOSTFTDCTRADERAPI/REQQRYRULEINTERPARAMETER.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryRULEInterParameter(CThostFtdcRULEInterParameterField *pRULEInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pRULEInterParameter：RULE跨品种抵扣参数

```
struct CThostFtdcRULEInterParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///优先级
    TThostFtdcSpreadIdType  SpreadId;
    ///品种间对锁仓费率折扣比例
    TThostFtdcRatioType InterRate;
    ///第一腿构成品种
    TThostFtdcInstrumentIDType  Leg1ProdFamilyCode;
    ///第二腿构成品种
    TThostFtdcInstrumentIDType  Leg2ProdFamilyCode;
    ///腿1比例系数
    TThostFtdcCommonIntType Leg1PropFactor;
    ///腿2比例系数
    TThostFtdcCommonIntType Leg2PropFactor;
    ///商品群号
    TThostFtdcCommodityGroupIDType  CommodityGroupID;
    ///商品群名称
    TThostFtdcInstrumentNameType    CommodityGroupName;
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
