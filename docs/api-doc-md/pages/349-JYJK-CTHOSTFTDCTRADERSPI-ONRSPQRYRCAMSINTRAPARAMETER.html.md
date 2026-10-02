# OnRspQryRCAMSIntraParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryRCAMSIntraParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS品种内风险对冲参数查询响应，当执行[ReqQryRCAMSIntraParameter](pages/178-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSINTRAPARAMETER.html.md)后，该方法被调用。
<a id="8bbd3095-1473-424c-9f0d-cc983a748e1d"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryRCAMSIntraParameter(CThostFtdcRCAMSIntraParameterField *pRCAMSIntraParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="4a7d9768-663c-4d23-90f4-8e6617aaa5f8"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRCAMSIntraParameter：RCAMS品种内风险对冲参数

```
struct CThostFtdcRCAMSIntraParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///产品组合代码
    TThostFtdcProductIDType CombProductID;
    ///品种内对冲比率
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

<a id="9c526832-54f0-4f9b-a82b-0a6ee3fae4ad"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="bf890e3e-94e7-4ef0-9fbc-0733cf0e0101"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
