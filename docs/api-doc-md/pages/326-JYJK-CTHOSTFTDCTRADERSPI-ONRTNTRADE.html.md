# OnRtnTrade

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRtnTrade<a id="content"></a>

<a id="left_menu"></a>

  ** **

成交通知，报单发出后有成交则通过此接口返回。私有流

详见[报单回调规则](pages/389-QTYWGZ-DBHB.html.md)

关于Tas的说明详见[TAS介绍](pages/398-QTYWGZ-TASJS.html.md)
<a id="4eb51202-c027-401f-91fb-20b387d9370d"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRtnTrade(CThostFtdcTradeField *pTrade) {};

<a id="4fd00a0d-30b1-4673-9e46-354f1e64a8db"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

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

TradeType：成交类型，[报价中的情况](pages/388-QTYWGZ-BJHXJ.html.md#anchor-id-02)

<a id="18af9b62-5e7f-4bbd-a05b-f017ef65cc9e"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="15865782-53c8-47e3-83b0-10508ae80c7b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

<a id="anchor-id-10"></a>

<a id="region_header_1"></a>

不同交易所，为什么TradeDate有的是自然日有的是交易日？<a id="region_panel_1"></a>

| TradeDate字段，大商所、郑商所回报中该字段为交易日；上期所、能源回报为自然日。
建议确认一笔成交的时间用Tradingday+TradeTime这一组字段。 |
|---|

<a id="region_tail_1"></a>

<a id="anchor-id-11"></a>

<a id="region_header_2"></a>

成交中的，PriceSource（成交价来源）是什么意思？<a id="region_panel_2"></a>

| 报单撮合产生的最新成交价取买价、卖价及前成交价三者居中的价格。
买价≥卖价≥前成交价, 最新成交价=卖价
买价≥前成交价≥卖价, 最新成交价=前成交价
前成交价≥买价≥卖价, 最新成交价=买价
和头文件的三个枚举值匹配 |
|---|

<a id="region_tail_2"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
