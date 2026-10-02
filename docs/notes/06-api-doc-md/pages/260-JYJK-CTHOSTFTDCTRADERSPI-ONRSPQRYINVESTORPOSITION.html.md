# OnRspQryInvestorPosition

OnRspQryInvestorPosition

请求查询投资者持仓响应，当执行[ReqQryInvestorPosition](../CTHOSTFTDCTRADERAPI/REQQRYINVESTORPOSITION.html)后，该方法被调用。

CTP 系统将持仓明细记录按合约，持仓方向，开仓日期（仅针对上期所，区分昨仓、今仓）进行汇总。

卖可平数量=多头仓-shortfrozen

买可平数量=空头仓-longfrozen

关于Tas的说明详见[TAS介绍](../../QTYWGZ/TASJS.html)

◇ 1. 函数原型

virtual void OnRspQryInvestorPosition(CThostFtdcInvestorPositionField *pInvestorPosition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInvestorPosition：投资者持仓

```
struct CThostFtdcInvestorPositionField
{
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
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
    ///持仓成本差值（柜台版本6.7.1后已废弃）
    TThostFtdcMoneyType PositionCostOffset;
    ///tas持仓手数
    TThostFtdcVolumeType    TasPosition;
    ///tas持仓成本
    TThostFtdcMoneyType TasPositionCost;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///期权市值
    TThostFtdcMoneyType OptionValue;
};

```

PositionProfit：期权没有持仓盈亏，为0

Position：表示当前持仓数量

TodayPosition：表示今新开仓

YdPosition：表示昨日收盘时持仓数量（≠ 当前的昨仓数量，静态，日间不随着开平仓而变化）

当前的昨仓数量 = ∑Position -∑TodayPosition

YdStrikeFrozen：该字段是给个股期权用的，期货期权里一直保持为0

PositionCost：持仓成本=上日持仓 * 昨结算价 * 合约乘数 + SUM（今日持仓 * 开仓价 * 合约乘数）

PositionDate：持仓日期（用于区分上海和能源的仓是今仓还是昨仓）

OpenAmount:开仓金额（所有开仓单的金额累加用于计算开仓手续费）开仓金额是当天一直累加，即使仓位平掉了手续费也要计算。

OpenCost：开仓成本，所有持仓明细的开仓成本的加权平均数，计算方法 = (开仓价*手数1*合约乘数+开仓价*手数2*合约乘数...)/(手数1*合约乘数+手数2*合约乘数+...)

SettlementPrice：该字段日初为昨结算价，期权空头持仓、期权多头持仓、期货持仓中该字段会实时跟随行情的最新价变动。

FrozenCommission：表示冻结手续费。注意交易盘中对开/平仓手续费会进行加收（冻结手续费和成交手续费均会加收），每手加收0.005元，以避免交易和结算的计算精度不同导致客户盘后资金不足的情况。

Commission：表示实收手续费。注意交易盘中对开/平仓手续费会进行加收（冻结手续费和成交手续费均会加收），每手加收0.005元，以避免交易和结算的计算精度不同导致客户盘后资金不足的情况。

OptionValue：期权市值=（多头期权手数-空头期权手数）*期权合约乘数*期权最新价。终端厂商可通过期权市值计算市值权益：市值权益=期权市值+权益。

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

投资者发现自己账号里面莫名其妙多了一条“PRT SR711&SR711C6000"的持仓，是什么原因？

| 这个合约是郑商所跨式，宽跨式期权组合合约。此合约不需要投资者使用特别的指令构建，只要客户持有可以构成备兑期权组合的持仓。在每日结算的时候，郑商所会将符合条件的期权和期货持仓自动确认为备兑期权套利持仓，包括备兑看涨期权套利和备兑看跌期权套利，并给予保证金优惠。
对于此期权合约，投资者需要分腿平仓，不需要管这个是否为备兑组合。 |
|---|

各大交易所保证金的单边收取规则有的是盘中生效，有的是结算后生效。那查询的持仓中的保证金占用是否有区别？

| 查询后的结果都是按照优惠收取。 |
|---|
