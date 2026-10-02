# OnRspQryRCAMSShortOptAdjustParam

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryRCAMSShortOptAdjustParam<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS空头期权风险调整参数查询响应，当执行[ReqQryRCAMSShortOptAdjustParam](pages/180-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSSHORTOPTADJUSTPARAM.html.md)后，该方法被调用。
<a id="442a0ea8-10aa-4e98-9439-39bf148beb40"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryRCAMSShortOptAdjustParam(CThostFtdcRCAMSShortOptAdjustParamField *pRCAMSShortOptAdjustParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="a7452c59-9aa3-4bd7-937d-80ea1dd2de51"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRCAMSShortOptAdjustParam：RCAMS空头期权风险调整参数

```
struct CThostFtdcRCAMSShortOptAdjustParamField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///产品组合代码
    TThostFtdcProductIDType CombProductID;
    ///投套标志
    TThostFtdcHedgeFlagType HedgeFlag;
    ///空头期权风险调整标准
    TThostFtdcAdjustValueType   AdjustValue;
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

<a id="caec014a-e769-45a5-830c-8319b8f79810"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="84a8a77f-456a-4c87-a9bd-707613fd5f2e"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
