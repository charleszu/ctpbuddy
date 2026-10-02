# OnRspQryInvestorProdSPBMDetail

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryInvestorProdSPBMDetail<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者产品SPBM明细查询响应，当执行[ReqQryInvestorProdSPBMDetail](pages/170-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPRODSPBMDETAIL.html.md)后，该方法被调用。
<a id="a90dcd76-3e40-45fe-b897-0245c39fdb61"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryInvestorProdSPBMDetail(CThostFtdcInvestorProdSPBMDetailField *pInvestorProdSPBMDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="27019ad3-8049-48a5-8ba4-06ea5a819fd7"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInvestorProdSPBMDetail：投资者产品SPBM明细

```
struct CThostFtdcInvestorProdSPBMDetailField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///品种代码
    TThostFtdcInstrumentIDType  ProdFamilyCode;
    ///合约内对锁保证金
    TThostFtdcMoneyType IntraInstrMargin;
    ///买归集保证金
    TThostFtdcMoneyType BCollectingMargin;
    ///卖归集保证金
    TThostFtdcMoneyType SCollectingMargin;
    ///品种内合约间对锁保证金
    TThostFtdcMoneyType IntraProdMargin;
    ///净保证金
    TThostFtdcMoneyType NetMargin;
    ///产品间对锁保证金
    TThostFtdcMoneyType InterProdMargin;
    ///裸保证金
    TThostFtdcMoneyType SingleMargin;
    ///附加保证金
    TThostFtdcMoneyType AddOnMargin;
    ///交割月保证金
    TThostFtdcMoneyType DeliveryMargin;
    ///看涨期权最低风险
    TThostFtdcMoneyType CallOptionMinRisk;
    ///看跌期权最低风险
    TThostFtdcMoneyType PutOptionMinRisk;
    ///卖方期权最低风险
    TThostFtdcMoneyType OptionMinRisk;
    ///买方期权冲抵价值
    TThostFtdcMoneyType OptionValueOffset;
    ///卖方期权权利金
    TThostFtdcMoneyType OptionRoyalty;
    ///价值冲抵
    TThostFtdcMoneyType RealOptionValueOffset;
    ///保证金
    TThostFtdcMoneyType Margin;
    ///交易所保证金
    TThostFtdcMoneyType ExchMargin;
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

<a id="272cebd3-de74-4661-bf1f-424890be5fc3"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="68a315a9-c830-468e-a000-75bba175952a"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
