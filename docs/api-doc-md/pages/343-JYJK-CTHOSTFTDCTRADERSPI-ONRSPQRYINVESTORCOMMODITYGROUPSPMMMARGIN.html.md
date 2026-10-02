# OnRspQryInvestorCommodityGroupSPMMMargin

OnRspQryInvestorCommodityGroupSPMMMargin

请求投资者商品群SPMM记录查询响应，当执行[ReqQryInvestorCommodityGroupSPMMMargin](../CTHOSTFTDCTRADERAPI/REQQRYINVESTORCOMMODITYGROUPSPMMMARGIN.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryInvestorCommodityGroupSPMMMargin(CThostFtdcInvestorCommodityGroupSPMMMarginField *pInvestorCommodityGroupSPMMMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInvestorCommodityGroupSPMMMargin：投资者商品群SPMM记录

```
struct CThostFtdcInvestorCommodityGroupSPMMMarginField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///商品群代码
    TThostFtdcSPMMProductIDType CommodityGroupID;
    ///优惠仓位应收保证金
    TThostFtdcMoneyType MarginBeforeDiscount;
    ///不优惠仓位应收保证金
    TThostFtdcMoneyType MarginNoDiscount;
    ///多头风险
    TThostFtdcMoneyType LongRisk;
    ///空头风险
    TThostFtdcMoneyType ShortRisk;
    ///商品群平仓冻结保证金
    TThostFtdcMoneyType CloseFrozenMargin;
    ///SPMM跨品种优惠系数
    TThostFtdcSPMMDiscountRatioType InterCommodityRate;
    ///商品群最小保证金比例
    TThostFtdcSPMMDiscountRatioType MiniMarginRatio;
    ///投资者保证金和交易所保证金的比例
    TThostFtdcRatioType AdjustRatio;
    ///SPMM品种内优惠汇总
    TThostFtdcMoneyType IntraCommodityDiscount;
    ///SPMM跨品种优惠
    TThostFtdcMoneyType InterCommodityDiscount;
    ///交易所保证金
    TThostFtdcMoneyType ExchMargin;
    ///投资者保证金
    TThostFtdcMoneyType InvestorMargin;
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
