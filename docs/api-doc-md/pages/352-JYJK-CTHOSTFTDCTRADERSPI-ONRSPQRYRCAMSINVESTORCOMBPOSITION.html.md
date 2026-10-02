# OnRspQryRCAMSInvestorCombPosition

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryRCAMSInvestorCombPosition<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS策略组合持仓查询响应，当执行[ReqQryRCAMSInvestorCombPosition](pages/181-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSINVESTORCOMBPOSITION.html.md)后，该方法被调用。
<a id="9d1b0348-c434-4e40-b86c-17405643c81c"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryRCAMSInvestorCombPosition(CThostFtdcRCAMSInvestorCombPositionField *pRCAMSInvestorCombPosition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="1431d66a-35b5-47e4-9c12-125159431e80"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

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

<a id="1e4de077-4f2d-4dd0-b5a4-36fe72ce6ef2"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="4f1d9237-1569-4da0-af9f-a6277c134a5b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
