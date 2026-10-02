# OnRspQryInvestorPositionDetail

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryInvestorPositionDetail<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询投资者持仓明细响应，当执行[ReqQryInvestorPositionDetail](pages/111-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPOSITIONDETAIL.html.md)后，该方法被调用。

关于Tas的说明详见[TAS介绍](pages/398-QTYWGZ-TASJS.html.md)
<a id="5e59e36a-7737-42a1-9efc-18262085e288"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryInvestorPositionDetail(CThostFtdcInvestorPositionDetailField *pInvestorPositionDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="c86a346e-65ca-49ca-9a38-d18a9ddb523b"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInvestorPositionDetail：投资者持仓明细

```
struct CThostFtdcInvestorPositionDetailField
{
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///投机套保标志
    TThostFtdcHedgeFlagType HedgeFlag;
    ///买卖
    TThostFtdcDirectionType Direction;
    ///开仓日期
    TThostFtdcDateType  OpenDate;
    ///成交编号
    TThostFtdcTradeIDType   TradeID;
    ///数量
    TThostFtdcVolumeType    Volume;
    ///开仓价
    TThostFtdcPriceType OpenPrice;
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///结算编号
    TThostFtdcSettlementIDType  SettlementID;
    ///成交类型
    TThostFtdcTradeTypeType TradeType;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve2;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///逐日盯市平仓盈亏
    TThostFtdcMoneyType CloseProfitByDate;
    ///逐笔对冲平仓盈亏
    TThostFtdcMoneyType CloseProfitByTrade;
    ///逐日盯市持仓盈亏
    TThostFtdcMoneyType PositionProfitByDate;
    ///逐笔对冲持仓盈亏
    TThostFtdcMoneyType PositionProfitByTrade;
    ///投资者保证金
    TThostFtdcMoneyType Margin;
    ///交易所保证金
    TThostFtdcMoneyType ExchMargin;
    ///保证金率
    TThostFtdcRatioType MarginRateByMoney;
    ///保证金率(按手数)
    TThostFtdcRatioType MarginRateByVolume;
    ///昨结算价
    TThostFtdcPriceType LastSettlementPrice;
    ///结算价
    TThostFtdcPriceType SettlementPrice;
    ///平仓量
    TThostFtdcVolumeType    CloseVolume;
    ///平仓金额
    TThostFtdcMoneyType CloseAmount;
    ///先开先平剩余数量
    TThostFtdcVolumeType    TimeFirstVolume;
    ///投资单元代码
    TThostFtdcInvestUnitIDType  InvestUnitID;
    ///特殊持仓标志
    TThostFtdcSpecPosiTypeType  SpecPosiType;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///组合合约代码
    TThostFtdcInstrumentIDType  CombInstrumentID;
};

```

SettlementPrice：该字段日初为昨结算价，仓位变动时，会更新为当时的最新价，不随行情变动。例如某持仓对应的合约最新价为10，持仓手数为2，此时以成交价20平仓了1手，平仓后该多头持仓的SettlementPrice会变更为10。

<a id="anchor-id-01"></a>

Volume：如果是大商所的持仓则按照先单一后组合的平仓顺序显示平仓后的剩余手数。

<a id="anchor-id-02"></a>

PositionProfitByTrade：大商所开启rule后，大商所的期权会计算持仓盈亏，其他交易所无影响（不计算持仓盈亏）。

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

<a id="ae29c906-6bdc-4179-b201-be42ce3bcea6"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="9d8c1e57-dca1-4e50-afce-aa75ddb74f12"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
