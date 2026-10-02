# 大商所组保

大商所组保

大商所期权上市后，为了更好的服务市场，大商所启动了期权六期项目，主要从行权功能优化、期权套保、组合保证金优惠三方面完善期权相关业务。

大商所目前支持的组合有：期货对锁，期货跨期，期货跨品种，备兑买权，备兑卖权，期权跨式，期权宽跨式共7种，投保标志支持投机-投机，套保-套保，投机-套保，套保-投机共4种。

本文主要就API针对组保业务的一些调整和特性做相关说明，具体业务规则请参考交易所文档。

◇ 1. 接口说明

录入请求：[ReqCombActionInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQCOMBACTIONINSERT.html)

错误响应：[OnErrRtnCombActionInsert](../JYJK/CTHOSTFTDCTRADERSPI/ONERRRTNCOMBACTIONINSERT.html)，[OnRspCombActionInsert](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPCOMBACTIONINSERT.html)

正确响应：[OnRtnCombAction](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNCOMBACTION.html)

注意，这些接口并非新增，而是大商所的组合业务复用了此接口。

◇ 2. 可申请组合的合约

组合和拆分的合约为大商所组合优惠表中所列的合约，但目前CTP暂时不支持此优惠表的查询。在实际操作中，用户可通过[OnRspQryInstrument](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPQRYINSTRUMENT.html)或[OnRtnDepthMarketData](../HQJK/CTHOSTFTDCMDSPI/ONRTNDEPTHMARKETDATA.html)里返回的组合合约来填写。

目前的套利单合约（期货跨期和跨品种）基本都在大商所组合优惠表中。但是有一种情况，就是新合约上市，套利合约也是新合约的时候，当天是不包括在组合优惠表中的。

◇ 3. 申请组合填写规则

- 合约（InstrumentID）：

针对各组合类型举例如下：

a)  期权跨式组合合约申请和拆分（STD m1905-c-2700&m1905-p-2700）

b)  期权宽跨式组合合约申请和拆分（STG m1905-p-2400&m1905-c-2700）

c)  备兑组合合约申请和拆分（PRT m1905-c-2400&m1905）

d)  期货跨期组合合约申请和拆分（SP a1903&a1905）

e)  期货跨品种组合合约申请和拆分（SPC c1903&cs1909）

f)  期货对锁组合合约申请和拆分（SP a1903& a1903）

g)  期权买入垂直价差组合（BLS m1809-P-3150&m1809-P-3100）

h)  期权卖出垂直价差组合（BES m1809-P-3350&m1809-P-3150）

i)  期权日历价差组合（CAS m1807-P-3150&m1809-P-3150）

j)  买入期权期货组合（BFO m1809-P-3150&m1809）

k)  期权对锁组合合约申请和拆分（OPL m1809-P-3150&m1809-P-3150）

- 买卖方向（Direction）

以上面所列组合类型为例：

a、  b：方向为卖。这种组合类型需要左右两腿都是卖持仓，即空头持仓；

c：方向为卖。因为目前交易所只支持备兑卖权，不支持备兑买权，即支持左腿是卖持仓的组合，不支持左腿是买持仓的组合。

若右腿组合的是多头的期货合约，则称为“备兑看涨期权组合持仓”；若右腿组合的是空头的期货合约，则称为“备兑看跌期权组合持仓”；

d、e：当方向是买的时候就是左买持右卖持，当方向是卖的时候就是左卖持右买持；

f：方向只支持买，所以是只有左买持右卖持。

g：买入垂直价差组合：当方向是买时就是左买持仓右卖持，若为卖则会报错“CTP:找不到合约”

h：卖出垂直价差组合：当方向是卖时就是左卖持仓右买持，若为买则会报错“CTP:找不到合约”

i：日历价差：

1）卖出较近月份的看涨期权合约，买入较远月份相同执行价格的看涨期权合约，组合的买卖方向为卖

2）卖出较近月份的看跌期权合约，买入较远月份相同执行价格的看跌期权合约，组合的买卖方向为卖

j：买入期权期货组合：组合买卖方向为买

1）买入看涨期权，同时卖出对应期货合约

2）买入看跌期权，同时买入对应期货合约

k：期权对锁组合：组合买卖方向为买

1）在同一期权品种同一系列同一合约上建立数量相等、方向相反的头寸

- 组合方向（CombDirection）

组合方向支持‘申请组合’和‘拆分组合’。

- 投机套保类型（HedgeFlag）

投机套保类型支持‘投机’（指左右两腿都是投机），‘保值’（指左右两腿都是套保），‘投套（指左腿投机右腿套保）’，‘套投（指左腿套保右腿投机）’。

盘中的话，如果通过套利报单的话（[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)），是只支持投机-投机，套保-套保；如果是通过申请组合形成组合单的话，支持投机-投机，套保-套保，投机-套保，套保-投机。

- 交易所（ExchangeID）

该字段必填。

◇ 4. 仓位计算规则

组合合约申请接受后，会将现有的两腿持仓组合到一起，形成组合持仓。此时，在持仓汇总（[OnRspQryInvestorPosition](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPQRYINVESTORPOSITION.html)）里会显示三条持仓记录，分别是新增的组合合约持仓，原多头合约持仓和原空头合约持仓。而持仓明细（[OnRspQryInvestorPositionDetail](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPQRYINVESTORPOSITIONDETAIL.html)）里不会新增组合合约记录。

1)  比如投资者在大商所买入1手“SP a0905&a0909”合约，形成的持仓汇总如下：

| 合约 | 多头持仓 | 空头持仓 |
|---|---|---|
| SP a0905&a0909 | 1 | 0 |
| a0905 | 1 | 0 |
| a0909 | 1 | 0 |

2)  比如投资者在大商所组合1手“STG c1909-P-1680&c1909-C-2020”合约，形成的持仓汇总如下：

| 合约 | 多头持仓 | 空头持仓 |
|---|---|---|
| STG c1909-P-1680&c1909-C-2020 | 0 | 1 |
| c1909-P-1680 | 0 | 1 |
| c1909-C-2020 | 0 | 1 |

◇ 5. 平仓盈亏计算和保证金释放计算原则

平仓盈亏计算和保证金释放计算原则如下：

1）在计算平仓盈亏时，采用先开先平原则。

2）在计算保证金释放时，采用先平单腿持仓，后平优惠组合持仓的原则，优惠组合按照优先级从低到高的顺序进行打破。

3）投资者持仓明细里面的TimeFirstVolume，是指大商所的持仓按照先开先平的平仓顺序平仓后的剩余手数

4）投资者持仓明细里面的Volume如果是大商所的持仓则按照先单一后组合的平仓顺序平仓后的剩余手数

5）投资者持仓接口里面的PositionCostOffset，因为目前大商所平仓还是按照先单一后组合进行平仓，而计算平仓盈亏需要按照先开先平进行计算，所以增加该字段用以计算按照先开先平的原则的持仓盈亏。

综上所述，在通过持仓明细去计算持仓成本、持仓盈亏、平仓盈亏时，需要用TimeFirstVolume；计算保证金时，需要用Volume。

通过持仓汇总去计算持仓成本和持仓盈亏规则如下：

后台版本6.7.1之前

```
大商所先开先平持仓成本 = PositionCost- PositionCostOffset
大商所持仓盈亏：
多头：PositionProfit = LastPrice*Position*VolumeMultiple -
（PositionCost - PositionCostOffset）
空头：PositionProfit = （PositionCost - PositionCostOffset） -
LastPrice*Position*VolumeMultiple

```

后台版本6.7.1及之后，PositionCostOffset字段废弃

```
大商所先开先平持仓成本 = PositionCost
大商所持仓盈亏：
多头：PositionProfit = LastPrice*Position*VolumeMultiple - PositionCost
空头：PositionProfit = PositionCost - LastPrice*Position*VolumeMultiple

```

◇ 6. 代码示例

场景：现有c1909-P-1680和c1909-C-2020各一手的投机卖仓，申请组合代码示例如下：

```
    CThostFtdcInputCombActionField a = { 0 };
    strcpy_s(a.BrokerID, “9999”);
    strcpy_s(a.InvestorID, “00001”);
    strcpy_s(a.InstrumentID, “STG c1909-P-1680&c1909-C-2020”);
    strcpy_s(a.CombActionRef, "1");
    strcpy_s(a.UserID, “00001”);
    a.Direction = THOST_FTDC_D_Sell;
    a.Volume = 1;
    a.CombDirection = THOST_FTDC_CMDR_Comb;
    a.HedgeFlag = THOST_FTDC_HF_Speculation;
    strcpy_s(a.ExchangeID, “DCE”);
    m_pUserApi->ReqCombActionInsert(&a, nRequestID++);

```

◇ 7. FAQ

查询合约的时候发现合约id不全

| 对于老版本api如果有查询组合合约，或者收到的组合合约id超过了30个字符，超过的部分会被截断，行情中也是如此。 |
|---|
