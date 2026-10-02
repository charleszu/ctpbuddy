# 条件单规则

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

条件单规则<a id="content"></a>

<a id="left_menu"></a>

  ** **<a id="c003aaa8-8520-46cd-be47-f585310bce39"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 简介
<a id="panel1"></a>

条件单是一个带有触发条件的指令。该触发条件可以以市场上的最新行情为基准，也可以以指定价格为基准。比如：一个投资者有1手IF1910的空头持仓，并希望在市场价低于2200时买入平仓，他就可以使用条件单。这样当行情波动到满足该条件时，该报单就会被自动触发报出，而不需要他本人时刻盯着电脑屏幕去监视市场行情。

申报条件单时，条件单的报单回报中OrderSysID是以字符串“TJBD_”开头的由CTP系统自定义的报单编号，针对该交易所当日唯一；只对成功申报的条件单编号，错单不编号。因为这种情况下的编号由CTP系统自行维护，与交易所无关，仅用于管理本系统内的该条件单。

有效的使用条件单，可以做出限价止损指令（stop-and-limit order）和触及市价指令（market-if-touched order）。

CTP条件单为CTP后台系统自带指令，并非交易所官方支持指令。

<a id="7950cf1f-27a5-477b-a88f-a96a4747fe66"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 指令介绍
<a id="panel2"></a>

报入条件单指令使用[ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)函数

其中的核心数据结构是CThostFtdcInputOrderField

在结构体中和普通报单有区别的字段如下：

```
    ///触发条件
    TThostFtdcContingentConditionType   ContingentCondition;
    ///止损价
    TThostFtdcPriceType StopPrice;

```

条件单有效的触发条件如下：

```
    ///最新价大于条件价
    #define THOST_FTDC_CC_LastPriceGreaterThanStopPrice '5'
    ///最新价大于等于条件价
    #define THOST_FTDC_CC_LastPriceGreaterEqualStopPrice '6'
    ///最新价小于条件价
    #define THOST_FTDC_CC_LastPriceLesserThanStopPrice '7'
    ///最新价小于等于条件价
    #define THOST_FTDC_CC_LastPriceLesserEqualStopPrice '8'
    ///卖一价大于条件价
    #define THOST_FTDC_CC_AskPriceGreaterThanStopPrice '9'
    ///卖一价大于等于条件价
    #define THOST_FTDC_CC_AskPriceGreaterEqualStopPrice 'A'
    ///卖一价小于条件价
    #define THOST_FTDC_CC_AskPriceLesserThanStopPrice 'B'
    ///卖一价小于等于条件价
    #define THOST_FTDC_CC_AskPriceLesserEqualStopPrice 'C'
    ///买一价大于条件价
    #define THOST_FTDC_CC_BidPriceGreaterThanStopPrice 'D'
    ///买一价大于等于条件价
    #define THOST_FTDC_CC_BidPriceGreaterEqualStopPrice 'E'
    ///买一价小于条件价
    #define THOST_FTDC_CC_BidPriceLesserThanStopPrice 'F'
    ///买一价小于等于条件价
    #define THOST_FTDC_CC_BidPriceLesserEqualStopPrice 'H'

```

条件价对应的字段为StopPrice

<a id="02b18024-12dd-4146-beaf-47f455632292"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 条件单回调规则
<a id="panel3"></a>

- 测试场景一

报入一笔中金所条件单，触发后类型为限价单，报入后未成交。

调用接口：[ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

Direction [0] 买卖方向：买

ExchangeID [CFFEX]

CurrencyID [CNY]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1]

IsSwapOrder [0]

IsAutoSuspend [0]

ForceCloseReason [0]  强平原因：非强平

ContingentCondition [H]   **触发条件：买一价小于等于条件价**

StopPrice [50000]      **条件价**

OrderPriceType [2]    **触发后的报单价格条件：限价**

LimitPrice [5246] **触发后的委托价**

TimeCondition [3] **触发后的有效期类型：当日有效**

VolumeCondition [1]   **触发后的成交量类型：任何数量**

|  | 报入后，此时收到报单回报：
OnRtnOrder（OrderStatus [b]  尚未触发）
当触发条件满足时，收到报单回报：
OnRtnOrder（OrderStatus [c]  已触发）
OnRtnOrder（OrderStatus [a]  未知，报单已提交）
OnRtnOrder（OrderStatus [3]  未成交还在队列中） |  |
|---|---|---|

- 测试场景二

报入一笔中金所条件单，触发后类型为限价单，报入并触发后被CTP拒绝。

调用接口：[ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

Direction [0] 买卖方向：买

ExchangeID [CFFEX]

CurrencyID [CNY]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1]

IsSwapOrder [0]

IsAutoSuspend [0]

ForceCloseReason [0]  强平原因：非强平

ContingentCondition [H]   **触发条件：买一价小于等于条件价**

StopPrice [50000]      **条件价**

OrderPriceType [2]    **触发后的报单价格条件：限价**

LimitPrice [5246] **触发后的委托价**

TimeCondition [3] **触发后的有效期类型：当日有效**

VolumeCondition [1]   **触发后的成交量类型：任何数量**

|  | 报入后，此时收到报单回报：
OnRtnOrder（OrderStatus [b]  尚未触发）
当触发条件满足时，收到报单回报及CTP返回的报错：
OnRtnOrder（OrderStatus [c]  已触发）
OnRtnErrorConditionalOrder（ErrorMsg [CTP:资金不足]） |  |
|---|---|---|

注意：[OnRtnErrorConditionalOrder](pages/307-JYJK-CTHOSTFTDCTRADERSPI-ONRTNERRORCONDITIONALORDER.html.md)为私有流回报，可用私有流重传模式RESTART重新收取该回报。

- 测试场景三

报入一笔中金所条件单，触发后类型为限价单，报入并触发后被交易所拒绝。

调用接口：[ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

Direction [0] 买卖方向：买

ExchangeID [CFFEX]

CurrencyID [CNY]

VolumeTotalOriginal [1]   报单手数：1手

MinVolume [1]

IsSwapOrder [0]

IsAutoSuspend [0]

ForceCloseReason [0]  强平原因：非强平

ContingentCondition [H]   **触发条件：买一价小于等于条件价**

StopPrice [50000]      **条件价**

OrderPriceType [2]    **触发后的报单价格条件：限价**

LimitPrice [25600] **触发后的委托价**

TimeCondition [3] **触发后的有效期类型：当日有效**

VolumeCondition [1]   **触发后的成交量类型：任何数量**

|  | 报入后，此时收到报单回报：
OnRtnOrder（OrderStatus [b]  尚未触发）
当触发条件满足时，收到报单回报及交易所返回的报错：
OnRtnOrder（OrderStatus [c]  已触发）
OnRtnOrder（OrderStatus [a]  未知，报单已提交）
OnRtnOrder（OrderStatus [5] 已撤单报单被拒绝CFFEX:超出当时价格保护带上带价格） |  |
|---|---|---|

- 测试场景四

报入一笔中金所条件单，触发后类型为FOK，报入后未成交自动被撤销。

调用接口：[ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

Direction [0] 买卖方向：买

ExchangeID [CFFEX]

CurrencyID [CNY]

VolumeTotalOriginal [8]   报单手数：1手

MinVolume [8]

IsSwapOrder [0]

IsAutoSuspend [0]

ForceCloseReason [0]  强平原因：非强平

ContingentCondition [H]   **触发条件：买一价小于等于条件价**

StopPrice [50000]      **条件价**

OrderPriceType [2]    **触发后的报单价格条件：限价**

LimitPrice [5246] **触发后的委托价**

TimeCondition [1] **触发后的有效期类型：立即完成，否则撤销**

VolumeCondition [3]   **触发后的成交量类型：全部数量**

|  | 报入后，此时收到报单回报：
OnRtnOrder（OrderStatus [b]  尚未触发）
当触发条件满足时，收到报单回报及交易所返回的报错：
OnRtnOrder（OrderStatus [c]  已触发）
OnRtnOrder（OrderStatus [a]  未知，报单已提交）
OnRtnOrder（OrderStatus [5] 已撤单报单已提交） |  |
|---|---|---|

- 测试场景五

报入一笔中金所条件单，触发后类型为FAK，报入后部成部撤。

调用接口：[ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)

CombOffsetFlag [0]    组合开平标注：开仓

CombHedgeFlag [1] 组合投机套保标志：投机

Direction [0] 买卖方向：买

ExchangeID [CFFEX]

CurrencyID [CNY]

VolumeTotalOriginal [8]   报单手数：1手

MinVolume [1]

IsSwapOrder [0]

IsAutoSuspend [0]

ForceCloseReason [0]  强平原因：非强平

ContingentCondition [H]   **触发条件：买一价小于等于条件价**

StopPrice [50000]      **条件价**

OrderPriceType [2]    **触发后的报单价格条件：限价**

LimitPrice [5246] **触发后的委托价**

TimeCondition [1] **触发后的有效期类型：立即完成，否则撤销**

VolumeCondition [2]   **触发后的成交量类型：最小数量**

|  | 报入后，此时收到报单回报：
OnRtnOrder（OrderStatus [b]  尚未触发）
当触发条件满足时，收到报单回报及交易所返回的报错：
OnRtnOrder（OrderStatus [c]  已触发）
OnRtnOrder（OrderStatus [a]  未知，报单已提交）
此时CTP收到了交易所推送的撤单状态的报单回报：
OnRtnOrder（OrderStatus [5] 已撤单报单已提交）
此时CTP收到了交易所推送的成交回报：
OnRtnOrder（OrderStatus [5] 已撤单报单已提交）
OnRtnTrade |  |
|---|---|---|

<a id="author"></a>

<a id="theme_switcher"></a>
