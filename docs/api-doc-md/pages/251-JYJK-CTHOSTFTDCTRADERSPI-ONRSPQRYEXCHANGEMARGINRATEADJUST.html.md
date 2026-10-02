# OnRspQryExchangeMarginRateAdjust

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryExchangeMarginRateAdjust<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询交易所调整保证金率响应，当执行[ReqQryExchangeMarginRateAdjust](pages/100-JYJK-CTHOSTFTDCTRADERAPI-REQQRYEXCHANGEMARGINRATEADJUST.html.md)后，该方法被调用。
<a id="80028330-d340-4368-892c-da45b99d1cc5"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryExchangeMarginRateAdjust(CThostFtdcExchangeMarginRateAdjustField *pExchangeMarginRateAdjust, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="cb5e17b9-1fce-4bfe-ba11-179c2aaba0c2"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pExchangeMarginRateAdjust：交易所保证金率调整

```
struct CThostFtdcExchangeMarginRateAdjustField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///投机套保标志
    TThostFtdcHedgeFlagType HedgeFlag;
    ///跟随交易所投资者多头保证金率
    TThostFtdcRatioType LongMarginRatioByMoney;
    ///跟随交易所投资者多头保证金费
    TThostFtdcMoneyType LongMarginRatioByVolume;
    ///跟随交易所投资者空头保证金率
    TThostFtdcRatioType ShortMarginRatioByMoney;
    ///跟随交易所投资者空头保证金费
    TThostFtdcMoneyType ShortMarginRatioByVolume;
    ///交易所多头保证金率
    TThostFtdcRatioType ExchLongMarginRatioByMoney;
    ///交易所多头保证金费
    TThostFtdcMoneyType ExchLongMarginRatioByVolume;
    ///交易所空头保证金率
    TThostFtdcRatioType ExchShortMarginRatioByMoney;
    ///交易所空头保证金费
    TThostFtdcMoneyType ExchShortMarginRatioByVolume;
    ///不跟随交易所投资者多头保证金率
    TThostFtdcRatioType NoLongMarginRatioByMoney;
    ///不跟随交易所投资者多头保证金费
    TThostFtdcMoneyType NoLongMarginRatioByVolume;
    ///不跟随交易所投资者空头保证金率
    TThostFtdcRatioType NoShortMarginRatioByMoney;
    ///不跟随交易所投资者空头保证金费
    TThostFtdcMoneyType NoShortMarginRatioByVolume;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
};

```

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

<a id="7766ab2d-5028-4bfe-8864-91d524791c1f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="94964ba7-1612-4449-84f9-8e832c3fcb6b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

<a id="anchor-id-01"></a>

<a id="region_header_1"></a>

跟随交易所多头保证金率，交易所多头保证金率，不跟随交易所多头保证金率，这三个有什么区别？<a id="region_panel_1"></a>

| 投资者交易所保证金率分两种属性（跟随交易所，不跟随交易所）
跟随交易所：投资者交易所保证金率=max(期货公司交易所保证金率，跟随交易所投资者多头保证金率+交易所多头保证金率)
不跟随交易所：投资者交易所保证金率=不跟随交易所投资者多头保证金率 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
