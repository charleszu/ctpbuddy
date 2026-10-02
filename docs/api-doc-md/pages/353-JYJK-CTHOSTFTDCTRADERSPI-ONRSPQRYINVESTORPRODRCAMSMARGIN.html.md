# OnRspQryInvestorProdRCAMSMargin

OnRspQryInvestorProdRCAMSMargin

请求投资者品种RCAMS保证金查询响应，当执行[ReqQryInvestorProdRCAMSMargin](../CTHOSTFTDCTRADERAPI/REQQRYINVESTORPRODRCAMSMARGIN.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryInvestorProdRCAMSMargin(CThostFtdcInvestorProdRCAMSMarginField *pInvestorProdRCAMSMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInvestorProdRCAMSMargin：投资者品种RCAMS保证金

```
struct CThostFtdcInvestorProdRCAMSMarginField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///产品组合代码
    TThostFtdcProductIDType CombProductID;
    ///投套标志
    TThostFtdcHedgeFlagType HedgeFlag;
    ///商品群代码
    TThostFtdcProductIDType ProductGroupID;
    ///品种组合前风险
    TThostFtdcMoneyType RiskBeforeDiscount;
    ///同合约对冲风险
    TThostFtdcMoneyType IntraInstrRisk;
    ///品种买持仓风险
    TThostFtdcMoneyType BPosRisk;
    ///品种卖持仓风险
    TThostFtdcMoneyType SPosRisk;
    ///品种内对冲风险
    TThostFtdcMoneyType IntraProdRisk;
    ///品种净持仓风险
    TThostFtdcMoneyType NetRisk;
    ///品种间对冲风险
    TThostFtdcMoneyType InterProdRisk;
    ///空头期权风险调整
    TThostFtdcMoneyType ShortOptRiskAdj;
    ///空头期权权利金
    TThostFtdcMoneyType OptionRoyalty;
    ///大边组合平仓冻结保证金
    TThostFtdcMoneyType MMSACloseFrozenMargin;
    ///策略组合平仓/行权冻结保证金
    TThostFtdcMoneyType CloseCombFrozenMargin;
    ///平仓/行权冻结保证金
    TThostFtdcMoneyType CloseFrozenMargin;
    ///大边组合开仓冻结保证金
    TThostFtdcMoneyType MMSAOpenFrozenMargin;
    ///交割月期货开仓冻结保证金
    TThostFtdcMoneyType DeliveryOpenFrozenMargin;
    ///开仓冻结保证金
    TThostFtdcMoneyType OpenFrozenMargin;
    ///投资者冻结保证金
    TThostFtdcMoneyType UseFrozenMargin;
    ///大边组合交易所持仓保证金
    TThostFtdcMoneyType MMSAExchMargin;
    ///交割月期货交易所持仓保证金
    TThostFtdcMoneyType DeliveryExchMargin;
    ///策略组合交易所保证金
    TThostFtdcMoneyType CombExchMargin;
    ///交易所持仓保证金
    TThostFtdcMoneyType ExchMargin;
    ///投资者持仓保证金
    TThostFtdcMoneyType UseMargin;
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
