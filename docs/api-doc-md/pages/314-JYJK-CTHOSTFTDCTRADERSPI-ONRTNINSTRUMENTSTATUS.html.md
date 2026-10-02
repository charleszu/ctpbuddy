# OnRtnInstrumentStatus

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRtnInstrumentStatus<a id="content"></a>

<a id="left_menu"></a>

  ** **

合约交易状态通知，主动推送。公有流回报。

各交易所的合约状态变化详见[合约状态变化说明](pages/395-QTYWGZ-HYZTBHSM2.html.md)。
<a id="9f3506a8-3a79-4faa-af99-b33cf9b6f9d3"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRtnInstrumentStatus(CThostFtdcInstrumentStatusField *pInstrumentStatus) {};

<a id="a77531dd-d681-4bc4-8325-2a710e85b3da"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInstrumentStatus：合约状态

```
struct CThostFtdcInstrumentStatusField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///保留的无效字段
    TThostFtdcOldExchangeInstIDType reserve1;
    ///结算组代码
    TThostFtdcSettlementGroupIDType SettlementGroupID;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve2;
    ///合约交易状态
    TThostFtdcInstrumentStatusType  InstrumentStatus;
    ///交易阶段编号
    TThostFtdcTradingSegmentSNType  TradingSegmentSN;
    ///进入本状态时间
    TThostFtdcTimeType  EnterTime;
    ///进入本状态原因
    TThostFtdcInstStatusEnterReasonType EnterReason;
    ///合约在交易所的代码
    TThostFtdcExchangeInstIDType    ExchangeInstID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
};

```

EnterTime：只有郑商所的时间戳是CTP的本地时间，其他交易所的是交易所时间

注：TThostFtdcInstrumentStatusType 合约交易状态类型新增THOST_FTDC_IS_TransactionProcessing 为'7'的枚举值，与上期所API合约状态保持一致。

对于到期日合约，上期所收盘后推送的合约状态值是7。交易所会根据此枚举值做业务判断，该合约状态表示到期日合约在15:00至15:30分可以做执行宣告、放弃执行宣告、期权自对冲等业务。对于CTP来讲，这个状态是无用。

<a id="05af1734-929c-492c-9450-a94e74c1c614"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="d4321845-4d1e-4ba7-b508-7f02df4b9780"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

<a id="region_header_1"></a>

如何获取期权合约的状态？<a id="region_panel_1"></a>

| 合约状态推送到产品级别，期权也是推送到产品级别。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
