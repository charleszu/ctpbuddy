# OnRtnDepthMarketData

OnRtnDepthMarketData

深度行情通知，当[SubscribeMarketData](../CTHOSTFTDCMDAPI/SUBSCRIBEMARKETDATA.html)订阅行情后，行情通知由此推送。

◇ 1. 函数原型

virtual void OnRtnDepthMarketData(CThostFtdcDepthMarketDataField *pDepthMarketData) {};

◇ 2. 参数

pDepthMarketData：深度行情

```
struct CThostFtdcDepthMarketDataField
{
    ///交易日
    TThostFtdcDateType   TradingDay;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///交易所代码
    TThostFtdcExchangeIDType ExchangeID;
    ///合约在交易所的代码
    TThostFtdcExchangeInstIDType    ExchangeInstID;
    ///最新价
    TThostFtdcPriceType  LastPrice;
    ///上次结算价
    TThostFtdcPriceType  PreSettlementPrice;
    ///昨收盘
    TThostFtdcPriceType  PreClosePrice;
    ///昨持仓量
    TThostFtdcLargeVolumeType   PreOpenInterest;
    ///今开盘
    TThostFtdcPriceType  OpenPrice;
    ///最高价
    TThostFtdcPriceType  HighestPrice;
    ///最低价
    TThostFtdcPriceType  LowestPrice;
    ///数量
    TThostFtdcVolumeType Volume;
    ///成交金额
    TThostFtdcMoneyType  Turnover;
    ///持仓量
    TThostFtdcLargeVolumeType   OpenInterest;
    ///今收盘
    TThostFtdcPriceType  ClosePrice;
    ///本次结算价
    TThostFtdcPriceType  SettlementPrice;
    ///涨停板价
    TThostFtdcPriceType  UpperLimitPrice;
    ///跌停板价
    TThostFtdcPriceType  LowerLimitPrice;
    ///昨虚实度
    TThostFtdcRatioType  PreDelta;
    ///今虚实度
    TThostFtdcRatioType  CurrDelta;
    ///最后修改时间
    TThostFtdcTimeType   UpdateTime;
    ///最后修改毫秒
    TThostFtdcMillisecType   UpdateMillisec;
    ///申买价一
    TThostFtdcPriceType  BidPrice1;
    ///申买量一
    TThostFtdcVolumeType BidVolume1;
    ///申卖价一
    TThostFtdcPriceType  AskPrice1;
    ///申卖量一
    TThostFtdcVolumeType AskVolume1;
    ///申买价二
    TThostFtdcPriceType  BidPrice2;
    ///申买量二
    TThostFtdcVolumeType BidVolume2;
    ///申卖价二
    TThostFtdcPriceType  AskPrice2;
    ///申卖量二
    TThostFtdcVolumeType AskVolume2;
    ///申买价三
    TThostFtdcPriceType  BidPrice3;
    ///申买量三
    TThostFtdcVolumeType BidVolume3;
    ///申卖价三
    TThostFtdcPriceType  AskPrice3;
    ///申卖量三
    TThostFtdcVolumeType AskVolume3;
    ///申买价四
    TThostFtdcPriceType  BidPrice4;
    ///申买量四
    TThostFtdcVolumeType BidVolume4;
    ///申卖价四
    TThostFtdcPriceType  AskPrice4;
    ///申卖量四
    TThostFtdcVolumeType AskVolume4;
    ///申买价五
    TThostFtdcPriceType  BidPrice5;
    ///申买量五
    TThostFtdcVolumeType BidVolume5;
    ///申卖价五
    TThostFtdcPriceType  AskPrice5;
    ///申卖量五
    TThostFtdcVolumeType AskVolume5;
    ///当日均价
    TThostFtdcPriceType  AveragePrice;
    ///业务日期
    TThostFtdcDateType   ActionDay;
    ///上带价
    TThostFtdcPriceType BandingUpperPrice;
    ///下带价
    TThostFtdcPriceType BandingLowerPrice;
};

```

turnover:

郑商所算法：成交金额=成交均价*成交数量

其他交易所算法：成交金额=成交均价*成交数量*合约乘数

◇ 3. 返回

无

◇ 4. FAQ

行情中有些字段出现极大值，为什么？

| 行情通知中，例如结算价、收盘价、买卖价出现double极大值，则表示该字段没有值。例如涨停板的时候，因为没有卖价，会给出一个double极大值，同时卖量为0。 |
|---|

郑商所的结算价在盘中也会送出，但数值在盘中不会变化，这和盘后推送的结算价怎么区分？

| 郑商所夜盘收盘后会推送结算价，这也就是早上行情中有可能会收到结算价的原因，盘中行情中推送的结算价数值不会发生变化。无夜盘的合约，盘中不会收到结算价。白盘收盘后郑商所也会推送所有合约最新的结算价，CTP对交易所推送的结算价不做处理，直接转发。 |
|---|

郑商所的UDP行情里，Tradingday和AveragePrice字段都是空的，但是TCP行情里有，这是为什么？

| UDP行情不推送AveragePrice字段；TCP行情里的Tradingday和AveragePrice是CTP给的。 |
|---|

行情中哪些字段是CTP自己计算发出的？

| 郑商所的套利合约的涨跌停板价由CTP计算，交易所不发
成交总额郑商所不推送由CTP计算，另外cffex,dce,shfe,ine这几家的成交均价不推送，由ctp计算。 |
|---|

发现连上前置后，为什么收到的行情涨跌停价是0？

| 连上的如果是mdfront的话可能mdfront是级联的，也就是说上一级还是mdfront这两个mdfront如果版本不对，也同样会导致这个问题 |
|---|

订阅后大商所组合合约行情不跳动，其他交易终端（闪电王、快期等）的组合合约都有行情且在跳跃，为什么？

| 大商所只会推送单腿合约的行情，组合合约的行情需要下端自行计算。其他终端则是实时计算两条腿的价差，故行情看上去一直跳跃。 |
|---|

行情中为什么T日的OpenInterest和T+1日的PreOpenInterest手数不一致？

| 交易所有大笔报单产生，大笔报单不是通过交易方式报出的。不在盘中行情中体现，盘中交易所会公示出来，盘后才汇总到行情中。 |
|---|
