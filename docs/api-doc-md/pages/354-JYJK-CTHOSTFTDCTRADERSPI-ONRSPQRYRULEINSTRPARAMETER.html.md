# OnRspQryRULEInstrParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryRULEInstrParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RULE合约保证金参数查询响应，当执行[ReqQryRULEInstrParameter](pages/183-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRULEINSTRPARAMETER.html.md)后，该方法被调用。
<a id="7945d487-632e-4265-8486-dfe28d885f0c"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryRULEInstrParameter(CThostFtdcRULEInstrParameterField *pRULEInstrParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="9e38c4c3-9e78-4378-bf24-ec9ee426bad2"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRULEInstrParameter：RULE合约保证金参数

```
struct CThostFtdcRULEInstrParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///合约类型
    TThostFtdcInstrumentClassType   InstrumentClass;
    ///标准合约
    TThostFtdcInstrumentIDType  StdInstrumentID;
    ///投机买折算系数
    TThostFtdcRatioType BSpecRatio;
    ///投机卖折算系数
    TThostFtdcRatioType SSpecRatio;
    ///套保买折算系数
    TThostFtdcRatioType BHedgeRatio;
    ///套保卖折算系数
    TThostFtdcRatioType SHedgeRatio;
    ///买附加风险保证金
    TThostFtdcMoneyType BAddOnMargin;
    ///卖附加风险保证金
    TThostFtdcMoneyType SAddOnMargin;
    ///商品群号
    TThostFtdcCommodityGroupIDType  CommodityGroupID;
};

```

pRspInfo：响应信息

```
struct CThostFtdcRspInfoField
{
    ///错误代码
    TThostFtdcErrorIDType   ErrorID;
    ///错误信息
    TThostFtdcErrorMsgType  ErrorMsg;
};

```

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="9ba536f4-384f-486e-a5fc-dac917bc761f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="b6804b02-d93a-4a38-80da-88177cd8eb6e"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
