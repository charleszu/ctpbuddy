# OnRspQrySPBMAddOnInterParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQrySPBMAddOnInterParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求SPBM附加跨品种抵扣参数查询响应，当执行[ReqQrySPBMAddOnInterParameter](pages/175-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPBMADDONINTERPARAMETER.html.md)后，该方法被调用。
<a id="f26b5435-846f-45fd-847e-f443e70c74ca"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQrySPBMAddOnInterParameter(CThostFtdcSPBMAddOnInterParameterField *pSPBMAddOnInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="52527251-2a16-4ea0-9781-0e7ef6d7208a"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pSPBMAddOnInterParameter：SPBM附加跨品种抵扣参数

```
struct CThostFtdcSPBMAddOnInterParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///优先级
    TThostFtdcSpreadIdType  SpreadId;
    ///品种间对锁仓附加费率折扣比例
    TThostFtdcRatioType AddOnInterRateZ2;
    ///第一腿构成品种
    TThostFtdcInstrumentIDType  Leg1ProdFamilyCode;
    ///第二腿构成品种
    TThostFtdcInstrumentIDType  Leg2ProdFamilyCode;
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

<a id="8642bddd-418a-4846-86d1-399c6dbc8b02"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="a64974d0-87ec-4a24-8b3e-c49d2a4962c5"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
