# OnRspQryInvestorProdRULEMargin

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryInvestorProdRULEMargin<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者产品RULE保证金查询响应，当执行[ReqQryInvestorProdRULEMargin](pages/186-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPRODRULEMARGIN.html.md)后，该方法被调用。
<a id="955a3523-054c-40b5-915b-60327dc8827f"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryInvestorProdRULEMargin(CThostFtdcInvestorProdRULEMarginField *pInvestorProdRULEMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="f50c6ba3-8fef-4825-971e-f99b4fd2c439"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInvestorProdRULEMargin：投资者产品RULE保证金

```
struct CThostFtdcInvestorProdRULEMarginField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///品种代码
    TThostFtdcInstrumentIDType  ProdFamilyCode;
    ///合约类型
    TThostFtdcInstrumentClassType   InstrumentClass;
    ///商品群号
    TThostFtdcCommodityGroupIDType  CommodityGroupID;
    ///买标准持仓
    TThostFtdcStdPositionType   BStdPosition;
    ///卖标准持仓
    TThostFtdcStdPositionType   SStdPosition;
    ///买标准开仓冻结
    TThostFtdcStdPositionType   BStdOpenFrozen;
    ///卖标准开仓冻结
    TThostFtdcStdPositionType   SStdOpenFrozen;
    ///买标准平仓冻结
    TThostFtdcStdPositionType   BStdCloseFrozen;
    ///卖标准平仓冻结
    TThostFtdcStdPositionType   SStdCloseFrozen;
    ///品种内对冲标准持仓
    TThostFtdcStdPositionType   IntraProdStdPosition;
    ///品种内单腿标准持仓
    TThostFtdcStdPositionType   NetStdPosition;
    ///品种间对冲标准持仓
    TThostFtdcStdPositionType   InterProdStdPosition;
    ///单腿标准持仓
    TThostFtdcStdPositionType   SingleStdPosition;
    ///品种内对锁保证金
    TThostFtdcMoneyType IntraProdMargin;
    ///品种间对锁保证金
    TThostFtdcMoneyType InterProdMargin;
    ///跨品种单腿保证金
    TThostFtdcMoneyType SingleMargin;
    ///非组合合约保证金
    TThostFtdcMoneyType NonCombMargin;
    ///附加保证金
    TThostFtdcMoneyType AddOnMargin;
    ///交易所保证金
    TThostFtdcMoneyType ExchMargin;
    ///附加冻结保证金
    TThostFtdcMoneyType AddOnFrozenMargin;
    ///开仓冻结保证金
    TThostFtdcMoneyType OpenFrozenMargin;
    ///平仓冻结保证金
    TThostFtdcMoneyType CloseFrozenMargin;
    ///品种保证金
    TThostFtdcMoneyType Margin;
    ///冻结保证金
    TThostFtdcMoneyType FrozenMargin;
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

<a id="c9939990-10eb-4224-940b-10a5a4805a06"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="d49a5ba2-5c7e-4103-bfb0-668e35479a13"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
