# OnRspQryInstrument

OnRspQryInstrument

请求查询合约响应，当执行[ReqQryInstrument](../CTHOSTFTDCTRADERAPI/REQQRYINSTRUMENT.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryInstrument(CThostFtdcInstrumentField *pInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInstrument：合约

```
struct CThostFtdcInstrumentField
{
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合约名称
    TThostFtdcInstrumentNameType    InstrumentName;
    ///保留的无效字段
    TThostFtdcOldExchangeInstIDType reserve2;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve3;
    ///产品类型
    TThostFtdcProductClassType  ProductClass;
    ///交割年份
    TThostFtdcYearType  DeliveryYear;
    ///交割月
    TThostFtdcMonthType DeliveryMonth;
    ///市价单最大下单量
    TThostFtdcVolumeType    MaxMarketOrderVolume;
    ///市价单最小下单量
    TThostFtdcVolumeType    MinMarketOrderVolume;
    ///限价单最大下单量
    TThostFtdcVolumeType    MaxLimitOrderVolume;
    ///限价单最小下单量
    TThostFtdcVolumeType    MinLimitOrderVolume;
    ///合约数量乘数
    TThostFtdcVolumeMultipleType    VolumeMultiple;
    ///最小变动价位
    TThostFtdcPriceType PriceTick;
    ///创建日
    TThostFtdcDateType  CreateDate;
    ///上市日
    TThostFtdcDateType  OpenDate;
    ///到期日
    TThostFtdcDateType  ExpireDate;
    ///开始交割日
    TThostFtdcDateType  StartDelivDate;
    ///结束交割日
    TThostFtdcDateType  EndDelivDate;
    ///合约生命周期状态
    TThostFtdcInstLifePhaseType InstLifePhase;
    ///当前是否交易
    TThostFtdcBoolType  IsTrading;
    ///持仓类型
    TThostFtdcPositionTypeType  PositionType;
    ///持仓日期类型
    TThostFtdcPositionDateTypeType  PositionDateType;
    ///多头保证金率
    TThostFtdcRatioType LongMarginRatio;
    ///空头保证金率
    TThostFtdcRatioType ShortMarginRatio;
    ///是否使用大额单边保证金算法
    TThostFtdcMaxMarginSideAlgorithmType    MaxMarginSideAlgorithm;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve4;
    ///执行价
    TThostFtdcPriceType StrikePrice;
    ///期权类型
    TThostFtdcOptionsTypeType   OptionsType;
    ///合约基础商品乘数
    TThostFtdcUnderlyingMultipleType    UnderlyingMultiple;
    ///组合类型
    TThostFtdcCombinationTypeType   CombinationType;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///合约在交易所的代码
    TThostFtdcExchangeInstIDType    ExchangeInstID;
    ///产品代码
    TThostFtdcInstrumentIDType  ProductID;
    ///基础商品代码
    TThostFtdcInstrumentIDType  UnderlyingInstrID;
};

```

VolumeMultiple：合约乘数（取自交易所）

PriceTick：最小变动价位（取自交易所）

IsTrading：是否活跃（取自交易所）

DeliveryYear：交割年份（取自交易所）

DeliveryMonth：交割月（取自交易所）

MaxMarketOrderVolume:（取自交易所，郑商所的由柜台产品设置）

MinMarketOrderVolume:（取自交易所，郑商所的由柜台产品设置）

MaxLimitOrderVolume:（取自交易所）

MinLimitOrderVolume:（取自交易所）

柜台版本6.6.5及以上版本支持每日从交易所更新市价单、限价单最大最小报单量；

CTP不会根据合约信息里的下单量限制去判断下单手数，交由交易所判断。

OpenDate：上市日（取自交易所）

CreateDate：创建日（取自交易所）

ExpireDate：到期日（取自交易所）

StartDeliveDate：开始交割日（取自交易所）

EndDelivDate：结束交割日（取自交易所）

取自交易所表示这些字段每天更新自交易所，其余字段为柜台设置值。

PositionDateType:持仓日期类型（区分产品是否有昨仓和今仓的区别，只有上海和能源是使用历史持仓）

LongMarginRatio：多头保证金率（终值），算法如下：

跟随交易所多头保证金率：LongMarginRatioByMoney（由[OnRspQryExchangeMarginRate](ONRSPQRYEXCHANGEMARGINRATE.html)查到）+LongMarginRatioByMoney（由[OnRspQryExchangeMarginRateAdjust](ONRSPQRYEXCHANGEMARGINRATEADJUST.html)查到）

不跟随交易所多头保证金率：NoLongMarginRatioByMoney（由[OnRspQryExchangeMarginRateAdjust](ONRSPQRYEXCHANGEMARGINRATEADJUST.html)查到）

ShortMarginRatio：空头保证金率（终值），算法如下：

跟随交易所空头保证金率：ShortMarginRatioByMoney（由[OnRspQryExchangeMarginRate](ONRSPQRYEXCHANGEMARGINRATE.html)查到）+ShortMarginRatioByMoney（由[OnRspQryExchangeMarginRateAdjust](ONRSPQRYEXCHANGEMARGINRATEADJUST.html)查到）

不跟随交易所空头保证金率：NoShortMarginRatioByMoney（由[OnRspQryExchangeMarginRateAdjust](ONRSPQRYEXCHANGEMARGINRATEADJUST.html)查到）

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

月均价合约代码是否和其他期货品种代码一样吗？

| 不一样，例：pp2607F(聚乙烯均价合约) |
|---|
