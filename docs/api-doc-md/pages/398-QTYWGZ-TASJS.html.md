# TAS介绍

TAS介绍

本文旨在介绍CTP API中与TAS（Trading at Settlement, TAS 盘中结算价交易机制）业务相关的内容，具体交易细则请参考交易所官方文档。

◇ 1. 交易时间

交易日9：00-10：15、10：30-11：30

◇ 2. 报单

接口名称：[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)

-

合约（InstrumentID）：填写TAS合约，如对sc1901进行TAS交易，则合约填写sc1911TAS

-

价格（LimitPrice）：在升贴水上下限之间（tas行情涨跌停板价格之间）。

-

类型（OrderPriceType）：只支持限价指令THOST_FTDC_OPT_LimitPrice。

其余字段与标的期货对应字段意义相同（买卖方向、平今平昨、投机套保等字段都需填写）。

TAS开仓，盘中成交后直接计入期货持仓；TAS平仓区分平今/平昨，平对应的期货持仓；平仓的仓位冻结按照标的合约进行冻结。

◇ 3. 成交

接口名称：[OnRtnTrade](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNTRADE.html)

-

交易成交类型(TradeType)：为普通成交THOST_FTDC_TRDT_Common。

-

成交价（Price）：对于TAS成交，成交价无意义。

◇ 4. 持仓

接口名称：[OnRspQryInvestorPositionDetail](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPQRYINVESTORPOSITIONDETAIL.html)

- 持仓明细新增字段特殊持仓标志（SpecPosiType）：标识该明细为tas衍生成交。

接口名称：[OnRspQryInvestorPosition](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPQRYINVESTORPOSITION.html)

-

持仓汇总新增字段tas持仓手数（TasPosition）：记录该汇总中TAS开仓的数量。

-

持仓汇总新增字段tas持仓成本（TasPositionCost）：记录该汇总中TAS持仓成本。

tas只有当天有效，不计入昨仓。结算完就变成标的持仓了，不再是tas合约持仓。可以理解成标的合约的昨持仓。

◇ 5. 手续费和保证金

开仓保证金和手续费冻结都按照标的合约停板价冻结。

实收保证金率和手续费率都按照标的合约计算。

保证金率和手续费率使用标的合约的费率，通过费率查询接口只能查询到标的合约的费率。

◇ 6. 盈亏

计算持仓盈亏的时候，对于(总数量-TAS数量)部分计算持仓盈亏和保证金。对于TAS数量不计算盈亏。

◇ 7. 合约行情

合约代码、合约状态、买卖方向、报单量、成交量、日期、时间等字段正常发布。其余字段无意义，均为空。

盘中TAS成交量、成交额不计入标的合约的成交量、成交额。

◇ 8. 升贴水

能源交易所TAS的委托价格可以浮动正负N个最小变动价位，N即为tas的升贴水，N范围为tas的涨跌停板价之间

◇ 9. 代码示例

```c++

```
void orderinsert()
{
    CThostFtdcInputOrderField t = { 0 };
    strcpy_s(t.BrokerID, "1007");
    strcpy_s(t.InvestorID, "10000001");
    strcpy_s(t.UserID, "10000001");
    t.Direction = THOST_FTDC_D_Sell;
    t.CombOffsetFlag[0] = THOST_FTDC_OF_Open;
    t.CombHedgeFlag[0] = THOST_FTDC_HF_Speculation;
    t.ContingentCondition = THOST_FTDC_CC_Immediately;
    strcpy_s(t.InstrumentID, "sc2006TAS");  //tas合约代码
    t.ForceCloseReason = THOST_FTDC_FCC_NotForceClose;;
    t.LimitPrice = 0;   //价格为0
    t.StopPrice = 0;
    t.OrderPriceType = THOST_FTDC_OPT_LimitPrice;
    t.VolumeCondition = THOST_FTDC_VC_AV;
    t.TimeCondition = THOST_FTDC_TC_GFD;
    t.VolumeTotalOriginal = 10;
    m_ptraderapi->ReqOrderInsert(&t, m_requestid++);
}

```

```
