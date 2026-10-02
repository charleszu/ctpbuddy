# OnRtnTrade

OnRtnTrade

成交通知，报单发出后有成交则通过此接口返回。私有流

详见[报单回调规则](../../QTYWGZ/DBHB.html)

关于Tas的说明详见[TAS介绍](../../QTYWGZ/TASJS.html)

◇ 1. 函数原型

virtual void OnRtnTrade(CThostFtdcTradeField *pTrade) {};

◇ 2. 参数

pTrade：成交

```
struct CThostFtdcTradeField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///报单引用
    TThostFtdcOrderRefType  OrderRef;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///成交编号
    TThostFtdcTradeIDType   TradeID;
    ///买卖方向
    TThostFtdcDirectionType Direction;
    ///报单编号
    TThostFtdcOrderSysIDType    OrderSysID;
    ///会员代码
    TThostFtdcParticipantIDType ParticipantID;
    ///客户代码
    TThostFtdcClientIDType  ClientID;
    ///交易角色
    TThostFtdcTradingRoleType   TradingRole;
    ///保留的无效字段
    TThostFtdcOldExchangeInstIDType reserve2;
    ///开平标志
    TThostFtdcOffsetFlagType    OffsetFlag;
    ///投机套保标志
    TThostFtdcHedgeFlagType HedgeFlag;
    ///价格
    TThostFtdcPriceType Price;
    ///数量
    TThostFtdcVolumeType    Volume;
    ///成交时期
    TThostFtdcDateType  TradeDate;
    ///成交时间
    TThostFtdcTimeType  TradeTime;
    ///成交类型
    TThostFtdcTradeTypeType TradeType;
    ///成交价来源
    TThostFtdcPriceSourceType   PriceSource;
    ///交易所交易员代码
    TThostFtdcTraderIDType  TraderID;
    ///本地报单编号
    TThostFtdcOrderLocalIDType  OrderLocalID;
    ///结算会员编号
    TThostFtdcParticipantIDType ClearingPartID;
    ///业务单元
    TThostFtdcBusinessUnitType  BusinessUnit;
    ///序号
    TThostFtdcSequenceNoType    SequenceNo;
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///结算编号
    TThostFtdcSettlementIDType  SettlementID;
    ///经纪公司报单编号
    TThostFtdcSequenceNoType    BrokerOrderSeq;
    ///成交来源
    TThostFtdcTradeSourceType   TradeSource;
    ///投资单元代码
    TThostFtdcInvestUnitIDType  InvestUnitID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///合约在交易所的代码
    TThostFtdcExchangeInstIDType    ExchangeInstID;
};

```

TradeType：成交类型，[报价中的情况](../../qtywgz/bjhxj.html#anchor-id-02)

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

不同交易所，为什么TradeDate有的是自然日有的是交易日？

| TradeDate字段，大商所、郑商所回报中该字段为交易日；上期所、能源回报为自然日。
建议确认一笔成交的时间用Tradingday+TradeTime这一组字段。 |
|---|

成交中的，PriceSource（成交价来源）是什么意思？

| 报单撮合产生的最新成交价取买价、卖价及前成交价三者居中的价格。
买价≥卖价≥前成交价, 最新成交价=卖价
买价≥前成交价≥卖价, 最新成交价=前成交价
前成交价≥买价≥卖价, 最新成交价=买价
和头文件的三个枚举值匹配 |
|---|
