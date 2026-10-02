# TAS介绍

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

TAS介绍<a id="content"></a>

<a id="left_menu"></a>

  ** **

本文旨在介绍CTP API中与TAS（Trading at Settlement, TAS 盘中结算价交易机制）业务相关的内容，具体交易细则请参考交易所官方文档。
<a id="b8ba5fc3-3190-4ba5-8920-1cb037f3925b"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 交易时间
<a id="panel1"></a>

交易日9：00-10：15、10：30-11：30

<a id="22b56d50-fa35-42af-a6f5-91979025156c"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 报单
<a id="panel2"></a>

接口名称：[ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)

-

合约（InstrumentID）：填写TAS合约，如对sc1901进行TAS交易，则合约填写sc1911TAS

-

价格（LimitPrice）：在升贴水上下限之间（tas行情涨跌停板价格之间）。

-

类型（OrderPriceType）：只支持限价指令THOST_FTDC_OPT_LimitPrice。

其余字段与标的期货对应字段意义相同（买卖方向、平今平昨、投机套保等字段都需填写）。

TAS开仓，盘中成交后直接计入期货持仓；TAS平仓区分平今/平昨，平对应的期货持仓；平仓的仓位冻结按照标的合约进行冻结。

<a id="83af6187-15a9-4b04-9ae3-ee8ee4c6c6d8"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 成交
<a id="panel3"></a>

接口名称：[OnRtnTrade](pages/326-JYJK-CTHOSTFTDCTRADERSPI-ONRTNTRADE.html.md)

-

交易成交类型(TradeType)：为普通成交THOST_FTDC_TRDT_Common。

-

成交价（Price）：对于TAS成交，成交价无意义。

<a id="1cd39897-1ea8-47fd-9a5f-d0a7a5edb0bc"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 持仓
<a id="panel4"></a>

接口名称：[OnRspQryInvestorPositionDetail](pages/262-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPOSITIONDETAIL.html.md)

- 持仓明细新增字段特殊持仓标志（SpecPosiType）：标识该明细为tas衍生成交。

接口名称：[OnRspQryInvestorPosition](pages/260-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPOSITION.html.md)

-

持仓汇总新增字段tas持仓手数（TasPosition）：记录该汇总中TAS开仓的数量。

-

持仓汇总新增字段tas持仓成本（TasPositionCost）：记录该汇总中TAS持仓成本。

tas只有当天有效，不计入昨仓。结算完就变成标的持仓了，不再是tas合约持仓。可以理解成标的合约的昨持仓。

<a id="39b6cf67-5082-4932-b919-f6b6eb351496"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. 手续费和保证金
<a id="panel5"></a>

开仓保证金和手续费冻结都按照标的合约停板价冻结。

实收保证金率和手续费率都按照标的合约计算。

保证金率和手续费率使用标的合约的费率，通过费率查询接口只能查询到标的合约的费率。

<a id="6cdb059e-aff9-442e-90e1-22f1338889ea"></a><a id="title6"></a>

<a id="header_span6"></a>◇ 6. 盈亏
<a id="panel6"></a>

计算持仓盈亏的时候，对于(总数量-TAS数量)部分计算持仓盈亏和保证金。对于TAS数量不计算盈亏。

<a id="0040394a-751e-45e2-a32d-e5f3340cd26d"></a><a id="title7"></a>

<a id="header_span7"></a>◇ 7. 合约行情
<a id="panel7"></a>

合约代码、合约状态、买卖方向、报单量、成交量、日期、时间等字段正常发布。其余字段无意义，均为空。

盘中TAS成交量、成交额不计入标的合约的成交量、成交额。

<a id="4ee5871a-f3f7-4840-8bc2-1e4aad0d3f69"></a><a id="title8"></a>

<a id="header_span8"></a>◇ 8. 升贴水
<a id="panel8"></a>

能源交易所TAS的委托价格可以浮动正负N个最小变动价位，N即为tas的升贴水，N范围为tas的涨跌停板价之间

<a id="84342fd2-d4a3-40f8-899c-da0fc846fba8"></a><a id="title9"></a>

<a id="header_span9"></a>◇ 9. 代码示例
<a id="panel9"></a>

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

<a id="author"></a>

<a id="theme_switcher"></a>
