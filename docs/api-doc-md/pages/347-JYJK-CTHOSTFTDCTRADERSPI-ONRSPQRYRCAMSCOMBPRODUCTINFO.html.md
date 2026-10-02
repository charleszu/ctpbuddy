# OnRspQryRCAMSCombProductInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryRCAMSCombProductInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS产品组合信息查询响应，当执行[ReqQryRCAMSCombProductInfo](pages/176-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSCOMBPRODUCTINFO.html.md)后，该方法被调用。
<a id="c08af99b-c3a9-47bc-ae16-8185fdf13c0d"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [OnRspQrySPBMAddOnInterParameter](pages/346-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMADDONINTERPARAMETER.html.md)(CThostFtdcSPBMAddOnInterParameterField *pSPBMAddOnInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="3b61c951-6382-4de6-9f12-06ad87fbf836"></a><a id="title2"></a>

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

<a id="4355ea30-68ff-4e26-8ef6-c52cf84d98fc"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="66e1a6bb-6b57-4b90-a9ed-938500f17ffa"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
