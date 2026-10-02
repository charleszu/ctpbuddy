# OnRtnInstrumentStatus

OnRtnInstrumentStatus

合约交易状态通知，主动推送。公有流回报。

各交易所的合约状态变化详见[合约状态变化说明](../../QTYWGZ/HYZTBHSM2.html)。

◇ 1. 函数原型

virtual void OnRtnInstrumentStatus(CThostFtdcInstrumentStatusField *pInstrumentStatus) {};

◇ 2. 参数

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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

如何获取期权合约的状态？

| 合约状态推送到产品级别，期权也是推送到产品级别。 |
|---|
