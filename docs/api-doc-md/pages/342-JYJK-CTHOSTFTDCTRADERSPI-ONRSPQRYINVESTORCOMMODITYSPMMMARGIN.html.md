# OnRspQryInvestorCommoditySPMMMargin

OnRspQryInvestorCommoditySPMMMargin

请求投资者商品组SPMM记录查询响应，当执行[ReqQryInvestorCommoditySPMMMargin](../CTHOSTFTDCTRADERAPI/REQQRYINVESTORCOMMODITYSPMMMARGIN.html)后，该方法被调用。

◇ 1. 函数原型

virtual void [OnRspQryInvestorProdSPBMDetail](ONRSPQRYINVESTORPRODSPBMDETAIL.html)(CThostFtdcInvestorProdSPBMDetailField *pInvestorProdSPBMDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInvestorProdSPBMDetail：投资者商品组SPMM记录

```
struct CThostFtdcInvestorCommoditySPMMMarginField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///商品组代码
    TThostFtdcSPMMProductIDType CommodityID;
    ///优惠仓位应收保证金
    TThostFtdcMoneyType MarginBeforeDiscount;
    ///不优惠仓位应收保证金
    TThostFtdcMoneyType MarginNoDiscount;
    ///多头实仓风险
    TThostFtdcMoneyType LongPosRisk;
    ///多头开仓冻结风险
    TThostFtdcMoneyType LongOpenFrozenRisk;
    ///多头被平冻结风险
    TThostFtdcMoneyType LongCloseFrozenRisk;
    ///空头实仓风险
    TThostFtdcMoneyType ShortPosRisk;
    ///空头开仓冻结风险
    TThostFtdcMoneyType ShortOpenFrozenRisk;
    ///空头被平冻结风险
    TThostFtdcMoneyType ShortCloseFrozenRisk;
    ///SPMM品种内跨期优惠系数
    TThostFtdcSPMMDiscountRatioType IntraCommodityRate;
    ///SPMM期权优惠系数
    TThostFtdcSPMMDiscountRatioType OptionDiscountRate;
    ///实仓对冲优惠金额
    TThostFtdcMoneyType PosDiscount;
    ///开仓报单对冲优惠金额
    TThostFtdcMoneyType OpenFrozenDiscount;
    ///品种风险净头
    TThostFtdcMoneyType NetRisk;
    ///平仓冻结保证金
    TThostFtdcMoneyType CloseFrozenMargin;
    ///冻结的手续费
    TThostFtdcMoneyType FrozenCommission;
    ///手续费
    TThostFtdcMoneyType Commission;
    ///冻结的资金
    TThostFtdcMoneyType FrozenCash;
    ///资金差额
    TThostFtdcMoneyType CashIn;
    ///行权冻结资金
    TThostFtdcMoneyType StrikeFrozenMargin;
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
