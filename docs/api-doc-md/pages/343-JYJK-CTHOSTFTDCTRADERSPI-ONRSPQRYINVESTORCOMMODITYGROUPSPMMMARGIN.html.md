# OnRspQryInvestorCommodityGroupSPMMMargin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryInvestorCommodityGroupSPMMMargin<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者商品群SPMM记录查询响应，当执行[ReqQryInvestorCommodityGroupSPMMMargin](pages/172-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORCOMMODITYGROUPSPMMMARGIN.html.md)后，该方法被调用。
<a id="b5bf0c51-ac6e-4ae4-bf9f-8c5b34e131a0"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryInvestorCommodityGroupSPMMMargin(CThostFtdcInvestorCommodityGroupSPMMMarginField *pInvestorCommodityGroupSPMMMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="71bd43f6-df6a-4c05-97d7-5af8c0a56f7a"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

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

<a id="aa1d05b1-5587-4743-92d5-39862f44ac1f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="b1692adf-4c1c-4822-b4e6-470e85eb5283"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
