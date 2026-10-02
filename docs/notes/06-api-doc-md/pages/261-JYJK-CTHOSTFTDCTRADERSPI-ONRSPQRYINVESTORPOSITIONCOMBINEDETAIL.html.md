# OnRspQryInvestorPositionCombineDetail

OnRspQryInvestorPositionCombineDetail

请求查询投资者持仓明细响应，当执行[ReqQryInvestorPositionCombineDetail](../CTHOSTFTDCTRADERAPI/REQQRYINVESTORPOSITIONCOMBINEDETAIL.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryInvestorPositionCombineDetail(CThostFtdcInvestorPositionCombineDetailField *pInvestorPositionCombineDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInvestorPositionCombineDetail：投资者组合持仓明细

```
struct CThostFtdcInvestorPositionCombineDetailField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///开仓日期
    TThostFtdcDateType  OpenDate;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///结算编号
    TThostFtdcSettlementIDType  SettlementID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///组合编号
    TThostFtdcTradeIDType   ComTradeID;
    ///撮合编号
    TThostFtdcTradeIDType   TradeID;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///投机套保标志
    TThostFtdcHedgeFlagType HedgeFlag;
    ///买卖
    TThostFtdcDirectionType Direction;
    ///持仓量
    TThostFtdcVolumeType    TotalAmt;
    ///投资者保证金
    TThostFtdcMoneyType Margin;
    ///交易所保证金
    TThostFtdcMoneyType ExchMargin;
    ///保证金率
    TThostFtdcRatioType MarginRateByMoney;
    ///保证金率(按手数)
    TThostFtdcRatioType MarginRateByVolume;
    ///单腿编号
    TThostFtdcLegIDType LegID;
    ///单腿乘数
    TThostFtdcLegMultipleType   LegMultiple;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve2;
    ///成交组号
    TThostFtdcTradeGroupIDType  TradeGroupID;
    ///投资单元代码
    TThostFtdcInvestUnitIDType  InvestUnitID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///组合持仓合约编码
    TThostFtdcInstrumentIDType  CombInstrumentID;
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

组合持仓明细应该如何去判断谁和谁是一对？

| 用ComTradeID判断。注意以下几点：
1) ComTradeID是CTP定义的。盘中新开仓的时候ComTradeID比如是115，在盘后结算完成后为了防止重复会重新打上编号，编号里带上交易日，如20170720000003；
2) ComTradeID并不一定是成对出现。比如下单五手组合合约：左腿全部成交，右腿是分笔成交的1，1，3的方式成交的。所以总的收到的成交回报就会有4条。这四条的ComTradeID，在盘中是一致的。 |
|---|
