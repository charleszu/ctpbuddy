# OnRspQryRiskSettleInvstPosition

OnRspQryRiskSettleInvstPosition

投资者风险结算持仓查询响应，当执行[ReqQryRiskSettleInvstPosition](../CTHOSTFTDCTRADERAPI/REQQRYRISKSETTLEINVSTPOSITION.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryRiskSettleInvstPosition(CThostFtdcRiskSettleInvstPositionField *pRiskSettleInvstPosition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pRiskSettleInvstPosition：投资者风险结算持仓

```
struct CThostFtdcRiskSettleInvstPositionField
{
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///持仓多空方向
    TThostFtdcPosiDirectionType PosiDirection;
    ///投机套保标志
    TThostFtdcHedgeFlagType HedgeFlag;
    ///持仓日期
    TThostFtdcPositionDateType  PositionDate;
    ///上日持仓
    TThostFtdcVolumeType    YdPosition;
    ///今日持仓
    TThostFtdcVolumeType    Position;
    ///多头冻结
    TThostFtdcVolumeType    LongFrozen;
    ///空头冻结
    TThostFtdcVolumeType    ShortFrozen;
    ///开仓冻结金额
    TThostFtdcMoneyType LongFrozenAmount;
    ///开仓冻结金额
    TThostFtdcMoneyType ShortFrozenAmount;
    ///开仓量
    TThostFtdcVolumeType    OpenVolume;
    ///平仓量
    TThostFtdcVolumeType    CloseVolume;
    ///开仓金额
    TThostFtdcMoneyType OpenAmount;
    ///平仓金额
    TThostFtdcMoneyType CloseAmount;
    ///持仓成本
    TThostFtdcMoneyType PositionCost;
    ///上次占用的保证金
    TThostFtdcMoneyType PreMargin;
    ///占用的保证金
    TThostFtdcMoneyType UseMargin;
    ///冻结的保证金
    TThostFtdcMoneyType FrozenMargin;
    ///冻结的资金
    TThostFtdcMoneyType FrozenCash;
    ///冻结的手续费
    TThostFtdcMoneyType FrozenCommission;
    ///资金差额
    TThostFtdcMoneyType CashIn;
    ///手续费
    TThostFtdcMoneyType Commission;
    ///平仓盈亏
    TThostFtdcMoneyType CloseProfit;
    ///持仓盈亏
    TThostFtdcMoneyType PositionProfit;
    ///上次结算价
    TThostFtdcPriceType PreSettlementPrice;
    ///本次结算价
    TThostFtdcPriceType SettlementPrice;
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///结算编号
    TThostFtdcSettlementIDType  SettlementID;
    ///开仓成本
    TThostFtdcMoneyType OpenCost;
    ///交易所保证金
    TThostFtdcMoneyType ExchangeMargin;
    ///组合成交形成的持仓
    TThostFtdcVolumeType    CombPosition;
    ///组合多头冻结
    TThostFtdcVolumeType    CombLongFrozen;
    ///组合空头冻结
    TThostFtdcVolumeType    CombShortFrozen;
    ///逐日盯市平仓盈亏
    TThostFtdcMoneyType CloseProfitByDate;
    ///逐笔对冲平仓盈亏
    TThostFtdcMoneyType CloseProfitByTrade;
    ///今日持仓
    TThostFtdcVolumeType    TodayPosition;
    ///保证金率
    TThostFtdcRatioType MarginRateByMoney;
    ///保证金率(按手数)
    TThostFtdcRatioType MarginRateByVolume;
    ///执行冻结
    TThostFtdcVolumeType    StrikeFrozen;
    ///执行冻结金额
    TThostFtdcMoneyType StrikeFrozenAmount;
    ///放弃执行冻结
    TThostFtdcVolumeType    AbandonFrozen;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///执行冻结的昨仓
    TThostFtdcVolumeType    YdStrikeFrozen;
    ///投资单元代码
    TThostFtdcInvestUnitIDType  InvestUnitID;
    ///大商所持仓成本差值，只有大商所使用（后台版本6.7.1后该字段废弃）
    TThostFtdcMoneyType PositionCostOffset;
    ///tas持仓手数
    TThostFtdcVolumeType    TasPosition;
    ///tas持仓成本
    TThostFtdcMoneyType TasPositionCost;
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
