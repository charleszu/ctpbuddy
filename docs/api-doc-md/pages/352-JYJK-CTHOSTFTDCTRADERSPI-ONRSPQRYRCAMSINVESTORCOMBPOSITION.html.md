# OnRspQryRCAMSInvestorCombPosition

OnRspQryRCAMSInvestorCombPosition

请求RCAMS策略组合持仓查询响应，当执行[ReqQryRCAMSInvestorCombPosition](../CTHOSTFTDCTRADERAPI/REQQRYRCAMSINVESTORCOMBPOSITION.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryRCAMSInvestorCombPosition(CThostFtdcRCAMSInvestorCombPositionField *pRCAMSInvestorCombPosition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pRCAMSInvestorCombPosition：RCAMS策略组合持仓

```
struct CThostFtdcRCAMSInvestorCombPositionField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///投套标志
    TThostFtdcHedgeFlagType HedgeFlag;
    ///持仓多空方向
    TThostFtdcPosiDirectionType PosiDirection;
    ///组合合约代码
    TThostFtdcInstrumentIDType  CombInstrumentID;
    ///单腿编号
    TThostFtdcLegIDType LegID;
    ///交易所组合合约代码
    TThostFtdcExchangeInstIDType    ExchangeInstID;
    ///持仓量
    TThostFtdcVolumeType    TotalAmt;
    ///交易所保证金
    TThostFtdcMoneyType ExchMargin;
    ///投资者保证金
    TThostFtdcMoneyType Margin;
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
