# OnRspQryInvestorProdRCAMSMargin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryInvestorProdRCAMSMargin<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者品种RCAMS保证金查询响应，当执行[ReqQryInvestorProdRCAMSMargin](pages/182-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPRODRCAMSMARGIN.html.md)后，该方法被调用。
<a id="3d034386-90f3-4ad2-9c86-4282b695a382"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryInvestorProdRCAMSMargin(CThostFtdcInvestorProdRCAMSMarginField *pInvestorProdRCAMSMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="18e52cfe-27a1-4690-8b1e-16e9a8587565"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

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

<a id="f61e9ab7-974e-4cba-bcdd-a0bca5466d86"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="d2011d3d-5350-4db6-9c87-f3730300cf5b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
