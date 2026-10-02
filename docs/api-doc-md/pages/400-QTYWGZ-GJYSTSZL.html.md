# 各交易所特殊指令

各交易所特殊指令

由于各家交易所的报单指令略有不同，故在此列出各家交易所报单接口情况。

◇ 1. 各交易所指令

◇ 1.1. 上期所

- 1.上期所立即单FOK

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [SHFE]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1]

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [1] **有效期类型：立即完成，否则撤销**

VolumeCondition [3]   **成交量类型：全部数量**

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]  强平原因：非强平

- 2.上期所立即单FAK

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [SHFE]

VolumeTotalOriginal [1]   报单手数：1手

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [1] **有效期类型：立即完成，否则撤销**

VolumeCondition [2]   **成交量类型：最小数量**

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]

[ 注:支持VolumeCondition=AV(任何数量)]

- 3.上期所市价单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [SHFE]

VolumeTotalOriginal [1]   报单手数：1手

IsSwapOrder [0]

OrderPriceType [1]    **报单价格条件：任意价**

Direction [0] 买卖方向：买

TimeCondition [1] **有效期类型：立即完成，否则撤销**

VolumeCondition [3]   成交量类型：全部数量

ContingentCondition [1]   触发条件：立即触发

LimitPrice [2000.00000000]    **保护价(该价格应在涨跌停板价格之间)**

ForceCloseReason [0]

| 字段 | 取值 | 含义说明 |
|---|---|---|
| LimitPrice | 保护价 | 报单成交价不劣于该价格 |
| VolumeCondition | AV | 任何数量，对应市价 FAK |
| VolumeCondition | CV | 全部数量，对应市价 FOK |
| VolumeCondition | MV | 最小数量，按指定最小数量成交，剩余部分撤销 |

- 4.上期所市价单转限价

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [SHFE]

VolumeTotalOriginal [1]   报单手数：1手

IsSwapOrder [0]

OrderPriceType [1]    **报单价格条件：任意价**

Direction [0] 买卖方向：买

TimeCondition [3] **有效期类型：当日有效**

VolumeCondition [1]   **成交量类型：任何数量**

ContingentCondition [1]   触发条件：立即触发

LimitPrice [2000.00000000]    **保护价(该价格应在涨跌停板价格之间)**

ForceCloseReason [0]

[ 注:转限价的市价单先以任何数量成交，未成交部分以保护价为限价价格进入待成交队列]

- 5.上期所套利指令

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

InstrumentID [SP au2608&au2610]

Direction [0] 买卖方向：买

CombOffsetFlag[0] = THOST_FTDC_OF_Open **组合开平标注：左腿开仓**

CombOffsetFlag[1] = THOST_FTDC_OF_Open **组合开平标注：右腿开仓**

CombHedgeFlag[0] = THOST_FTDC_HF_Speculation **组合投机套保标志：左腿投机**

CombHedgeFlag[1] = THOST_FTDC_HF_Speculation **组合投机套保标志：右腿投机**

ExchangeID [SHFE]

VolumeTotalOriginal [1]   报单手数：1手

IsSwapOrder [0]

OrderPriceType [2]    **报单价格条件：限价**

TimeCondition [3] 有效期类型：当日有效

VolumeCondition [1]   成交量类型：任何数量

ContingentCondition [1]   触发条件：立即触发

LimitPrice [100] **价差**

ForceCloseReason [0]

[ 注:套利指令不支持市价单。只支持限价单，包括FAK、FOK]

◇ 1.2. 大商所

- 1.大商所立即单FOK

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [DCE]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1] 最小成交量：1手

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [1] **有效期类型：立即完成，否则撤销**

VolumeCondition [3]   **成交量类型：全部数量**

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]

- 2.大商所立即单FAK

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [DCE]

VolumeTotalOriginal [3]   报单手数：3手

MinVolume [1] 最小成交量：1手

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [1] **有效期类型：立即完成，否则撤销**

VolumeCondition [1]   **成交量类型：任何数量**

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]

[ 注:VolumeCondition=MV(最小数量)不生效]

- 3.大商所市价单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [DCE]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1] 最小成交量：1手

IsSwapOrder [0]

OrderPriceType [1]    **报单价格条件：任意价**

Direction [0] 买卖方向：买

TimeCondition [1] **有效期类型：立即完成，否则撤销**

VolumeCondition [3]   成交量类型：全部数量

ContingentCondition [1]   触发条件：立即触发

LimitPrice [0.00000000]   **价格：0**

ForceCloseReason [0]

- 4.大商所本节有效

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

注：报单及做市商报价均支持“本节有效”申报，该业务需要交易所上线后方可使用。

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [DCE]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1] 最小成交量：1手

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [2] **有效期类型：本节有效**

VolumeCondition [1]   成交量类型：任何数量

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]

- 5.大商所止盈单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [DCE]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1] 最小成交量：1手

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [3] **有效期类型：当日有效**

VolumeCondition [1]   成交量类型：任何数量

ContingentCondition [3]   **触发条件：止盈**

ForceCloseReason [0]

- 6.大商所止损单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [DCE]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1] 最小成交量：1手

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [3] **有效期类型：当日有效**

VolumeCondition [1]   成交量类型：任何数量

ContingentCondition [2]   **触发条件：止损**

ForceCloseReason [0]

- 7.大商所市价止盈单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [DCE]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1] 最小成交量：1手

IsSwapOrder [0]

OrderPriceType [1]    **报单价格条件：任意价**

Direction [0] 买卖方向：买

TimeCondition [3] **有效期类型：当日有效**

VolumeCondition [1]   成交量类型：任何数量

ContingentCondition [3]   **触发条件：止盈**

LimitPrice [0.00000000]   **价格：0**

ForceCloseReason [0]

- 8.大商所市价止损单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [DCE]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1] 最小成交量：1手

IsSwapOrder [0]

OrderPriceType [1]    **报单价格条件：任意价**

Direction [0] 买卖方向：买

TimeCondition [3] **有效期类型：当日有效**

VolumeCondition [1]   成交量类型：任何数量

ContingentCondition [2]   **触发条件：止损**

LimitPrice [0.00000000]   **价格：0**

ForceCloseReason [0]

- 9.大商所互换单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [1]    组合开平标注：平仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [DCE]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1] 最小成交量：1手

IsSwapOrder [1]   **互换单标志：是**

OrderPriceType [2]    报单价格条件：限价

Direction [1] 买卖方向：卖

TimeCondition [3] 有效期类型：当日有效

VolumeCondition [2]   成交量类型：最小数量

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]

- 10.大商所套利单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

InstrumentID [SP a2109&a2201] **合约填套利合约**

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [DCE]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1] 最小成交量：1手

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [3] 有效期类型：当日有效

VolumeCondition [2]   成交量类型：最小数量

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]

- 11.大商所申请组合

调用接口：[ReqCombActionInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQCOMBACTIONINSERT.html)

BrokerID [1008]

InvestorID [90096650]

InstrumentID [OPL m2109-C-2800&m2109-C-2800]

CombActionRef []

UserID [90096650]

ExchangeID [DCE]

IPAddress []

MacAddress []

InvestUnitID []

Volume [1]

FrontID [0]

SessionID [0]

Direction [0] 买卖方向：买

CombDirection [0] **组合指令方向：申请组合**

HedgeFlag [1] 投机套保标志：投机

nRequestID [1]

◇ 1.3. 中金所

- 1.中金所立即单FOK

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CFFEX]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]   互换单标志：否

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [1] **有效期类型：立即完成，否则撤销**

VolumeCondition [3]   **成交量类型：全部数量**

ContingentCondition [1]   **触发条件：立即触发**

ForceCloseReason [0]

- 2.中金所立即单FAK

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CFFEX]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]   互换单标志：否

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [1] **有效期类型：立即完成，否则撤销**

VolumeCondition [2]   **成交量类型：最小数量**

ContingentCondition [1]   **触发条件：立即触发**

ForceCloseReason [0]

[ 注:支持VolumeCondition=AV(任何数量)]

- 3.中金所市价单-最优价

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CFFEX]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]   互换单标志：否

OrderPriceType [3]    **报单价格条件：最优价**

Direction [0] 买卖方向：买

TimeCondition [1] 有效期类型：立即完成，否则撤销

VolumeCondition [1]   **成交量类型：任何数量**

ContingentCondition [1]   触发条件：立即触发

LimitPrice [0.00000000]   **价格：0**

ForceCloseReason [0]

- 4.中金所市价单-最优价转限价

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CFFEX]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]   互换单标志：否

OrderPriceType [3]    **报单价格条件：最优价**

Direction [0] 买卖方向：买

TimeCondition [3] **有效期类型：当日有效**

VolumeCondition [1]   **成交量类型：任何数量**

ContingentCondition [1]   触发条件：立即触发

LimitPrice [0.00000000]   **价格：0**

ForceCloseReason [0]

- 5.中金所市价单-五档价

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CFFEX]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]   互换单标志：否

OrderPriceType [G]    **报单价格条件：五档价**

Direction [0] 买卖方向：买

TimeCondition [1] 有效期类型：立即完成，否则撤销

VolumeCondition [1]   **成交量类型：任何数量**

ContingentCondition [1]   触发条件：立即触发

LimitPrice [0.00000000]   **价格：0**

ForceCloseReason [0]

- 6.中金所市价单-五档价转限价

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CFFEX]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]   互换单标志：否

OrderPriceType [G]    **报单价格条件：五档价**

Direction [0] 买卖方向：买

TimeCondition [3] **有效期类型：当日有效**

VolumeCondition [1]   **成交量类型：任何数量**

ContingentCondition [1]   触发条件：立即触发

LimitPrice [0.00000000]   **价格：0**

ForceCloseReason [0]

- 7.中金所套利单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [2] **组合投机套保标志：套利**

ExchangeID [CFFEX]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [3] 有效期类型：当日有效

VolumeCondition [2]   成交量类型：最小数量

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]

◇ 1.4. 郑商所

- 1.郑商所立即单FOK

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CZCE]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [1] **有效期类型：立即完成，否则撤销**

VolumeCondition [3]   **成交量类型：全部数量**

ContingentCondition [1]   **触发条件：立即触发**

ForceCloseReason [0]

- 2.郑商所立即单FAK

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CZCE]

VolumeTotalOriginal [3]

MinVolume [1]

IsSwapOrder [0]

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [1] **有效期类型：立即完成，否则撤销**

VolumeCondition [1]   **成交量类型：任何数量**

ContingentCondition [1]   **触发条件：立即触发**

ForceCloseReason [0]

[ 注:VolumeCondition=MV(最小数量)不生效]

- 3.郑商所市价单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CZCE]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]

OrderPriceType [1]    **报单价格条件：任意价**

Direction [0] 买卖方向：买

TimeCondition [3] 有效期类型：当日有效

VolumeCondition [3]   成交量类型：全部数量

ContingentCondition [1]   触发条件：立即触发

LimitPrice [0.00000000]   **价格：0**

ForceCloseReason [0]

- 4.郑商所套利单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

InstrumentID [SPD TA109&TA110]    **合约填套利合约**

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CZCE]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]   互换单标志：否

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [3] 有效期类型：当日有效

VolumeCondition [2]   成交量类型：最小数量

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]

- 5.郑商所互换单

调用接口：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

CombOffsetFlag [1]    组合开平标注：平仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CZCE]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [1]   **互换单标志：是**

OrderPriceType [2]    报单价格条件：限价

Direction [1] 买卖方向：卖

TimeCondition [3] 有效期类型：当日有效

VolumeCondition [2]   成交量类型：最小数量

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]

- 6.郑商所组合单

调用接口：[ReqCombActionInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQCOMBACTIONINSERT.html)

InstrumentID [SPD TA109&TA206]    **填组合合约**

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

ExchangeID [CZCE]

VolumeTotalOriginal [1]

MinVolume [1]

IsSwapOrder [0]   互换单标志：否

OrderPriceType [2]    报单价格条件：限价

Direction [0] 买卖方向：买

TimeCondition [3] 有效期类型：当日有效

VolumeCondition [2]   成交量类型：最小数量

ContingentCondition [1]   触发条件：立即触发

ForceCloseReason [0]

◇ 1.5. 广期所

报单指令同大商所，请参考大商所

◇ 1.6. 能源中心

报单指令同上期所，请参考上期所
