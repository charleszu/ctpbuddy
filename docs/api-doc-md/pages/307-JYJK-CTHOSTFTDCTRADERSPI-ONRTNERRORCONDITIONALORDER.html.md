# OnRtnErrorConditionalOrder

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRtnErrorConditionalOrder<a id="content"></a>

<a id="left_menu"></a>

  ** **

提示条件单校验错误。
<a id="b01b1ae7-7b0e-4d4a-a93e-e9659f98bcf8"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRtnErrorConditionalOrder(CThostFtdcErrorConditionalOrderField *pErrorConditionalOrder) {};

<a id="f9c5905c-abd8-48ef-b076-0a65b0e5948a"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pErrorConditionalOrder：查询错误报单操作

```
struct CThostFtdcErrorConditionalOrderField
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
    ///报单价格条件
    TThostFtdcOrderPriceTypeType    OrderPriceType;
    ///买卖方向
    TThostFtdcDirectionType Direction;
    ///组合开平标志
    TThostFtdcCombOffsetFlagType    CombOffsetFlag;
    ///组合投机套保标志
    TThostFtdcCombHedgeFlagType CombHedgeFlag;
    ///价格
    TThostFtdcPriceType LimitPrice;
    ///数量
    TThostFtdcVolumeType    VolumeTotalOriginal;
    ///有效期类型
    TThostFtdcTimeConditionType TimeCondition;
    ///GTD日期
    TThostFtdcDateType  GTDDate;
    ///成交量类型
    TThostFtdcVolumeConditionType   VolumeCondition;
    ///最小成交量
    TThostFtdcVolumeType    MinVolume;
    ///触发条件
    TThostFtdcContingentConditionType   ContingentCondition;
    ///止损价
    TThostFtdcPriceType StopPrice;
    ///强平原因
    TThostFtdcForceCloseReasonType  ForceCloseReason;
    ///自动挂起标志
    TThostFtdcBoolType  IsAutoSuspend;
    ///业务单元
    TThostFtdcBusinessUnitType  BusinessUnit;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///本地报单编号
    TThostFtdcOrderLocalIDType  OrderLocalID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///会员代码
    TThostFtdcParticipantIDType ParticipantID;
    ///客户代码
    TThostFtdcClientIDType  ClientID;
    ///保留的无效字段
    TThostFtdcOldExchangeInstIDType reserve2;
    ///交易所交易员代码
    TThostFtdcTraderIDType  TraderID;
    ///安装编号
    TThostFtdcInstallIDType InstallID;
    ///报单提交状态
    TThostFtdcOrderSubmitStatusType OrderSubmitStatus;
    ///报单提示序号
    TThostFtdcSequenceNoType    NotifySequence;
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///结算编号
    TThostFtdcSettlementIDType  SettlementID;
    ///报单编号
    TThostFtdcOrderSysIDType    OrderSysID;
    ///报单来源
    TThostFtdcOrderSourceType   OrderSource;
    ///报单状态
    TThostFtdcOrderStatusType   OrderStatus;
    ///报单类型
    TThostFtdcOrderTypeType OrderType;
    ///今成交数量
    TThostFtdcVolumeType    VolumeTraded;
    ///剩余数量
    TThostFtdcVolumeType    VolumeTotal;
    ///报单日期
    TThostFtdcDateType  InsertDate;
    ///委托时间
    TThostFtdcTimeType  InsertTime;
    ///激活时间
    TThostFtdcTimeType  ActiveTime;
    ///挂起时间
    TThostFtdcTimeType  SuspendTime;
    ///最后修改时间
    TThostFtdcTimeType  UpdateTime;
    ///撤销时间
    TThostFtdcTimeType  CancelTime;
    ///最后修改交易所交易员代码
    TThostFtdcTraderIDType  ActiveTraderID;
    ///结算会员编号
    TThostFtdcParticipantIDType ClearingPartID;
    ///序号
    TThostFtdcSequenceNoType    SequenceNo;
    ///前置编号
    TThostFtdcFrontIDType   FrontID;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///用户端产品信息
    TThostFtdcProductInfoType   UserProductInfo;
    ///状态信息
    TThostFtdcErrorMsgType  StatusMsg;
    ///用户强平标志
    TThostFtdcBoolType  UserForceClose;
    ///操作用户代码
    TThostFtdcUserIDType    ActiveUserID;
    ///经纪公司报单编号
    TThostFtdcSequenceNoType    BrokerOrderSeq;
    ///相关报单
    TThostFtdcOrderSysIDType    RelativeOrderSysID;
    ///郑商所成交数量
    TThostFtdcVolumeType    ZCETotalTradedVolume;
    ///错误代码
    TThostFtdcErrorIDType   ErrorID;
    ///错误信息
    TThostFtdcErrorMsgType  ErrorMsg;
    ///互换单标志
    TThostFtdcBoolType  IsSwapOrder;
    ///营业部编号
    TThostFtdcBranchIDType  BranchID;
    ///投资单元代码
    TThostFtdcInvestUnitIDType  InvestUnitID;
    ///资金账号
    TThostFtdcAccountIDType AccountID;
    ///币种代码
    TThostFtdcCurrencyIDType    CurrencyID;
    ///保留的无效字段
    TThostFtdcOldIPAddressType  reserve3;
    ///Mac地址
    TThostFtdcMacAddressType    MacAddress;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///合约在交易所的代码
    TThostFtdcExchangeInstIDType    ExchangeInstID;
    ///IP地址
    TThostFtdcIPAddressType IPAddress;
};

```

StopPrice：条件单触发价格

ForceCloseReason：填0“非强平”

InstallID：CTP内部使用

OrderSubmitStatus：报单提交状态，查看是否通过ctp校验

OrderStatus：报单最终状态

ActiveTraderID：是从交易所收回来，原值填写，CTP核心不处理该字段。在撤单回报中，由报单席位号修改为撤单席位号，以方便操作员在撤单回报中看出撤单是从哪个席位发起的

BusinessUnit：是ctp内部使用字段，建议投资者不要使用

TraderID：是席位号

ActiveUserID：是ctp内部使用字段，建议投资者不要使用

<a id="95fcf369-5f79-49f6-aca8-9d67b2ee2d97"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="6ad64db6-91b9-4fe5-9bdf-52b2289bf355"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
