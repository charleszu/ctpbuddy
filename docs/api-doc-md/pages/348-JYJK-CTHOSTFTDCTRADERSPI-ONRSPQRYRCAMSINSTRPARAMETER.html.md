# OnRspQryRCAMSInstrParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryRCAMSInstrParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS同合约风险对冲参数查询响应，当执行[ReqQryRCAMSInstrParameter](pages/177-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSINSTRPARAMETER.html.md)后，该方法被调用。
<a id="6a246b71-fc66-40aa-93c8-0cc475f66c02"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryRCAMSInstrParameter(CThostFtdcRCAMSInstrParameterField *pRCAMSInstrParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="a3cbc065-a36c-411a-a255-4fde5ae7b232"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRCAMSInstrParameter：RCAMS同合约风险对冲参数

```
struct CThostFtdcRCAMSInstrParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///产品代码
    TThostFtdcProductIDType ProductID;
    ///同合约风险对冲比率
    TThostFtdcHedgeRateType HedgeRate;
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

<a id="8ada1412-61ce-4f8b-aaa0-5d597ab1a3ba"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="72c694c3-75df-4046-8198-647ffbcb1704"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
