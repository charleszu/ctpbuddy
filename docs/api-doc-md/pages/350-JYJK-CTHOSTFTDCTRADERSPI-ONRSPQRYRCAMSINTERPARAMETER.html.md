# OnRspQryRCAMSInterParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryRCAMSInterParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS跨品种风险折抵参数查询响应，当执行[ReqQryRCAMSInterParameter](pages/179-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSINTERPARAMETER.html.md)后，该方法被调用。
<a id="f3895204-da03-46bc-b29a-ba0bd9b59da7"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryRCAMSInterParameter(CThostFtdcRCAMSInterParameterField *pRCAMSInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="734f75c7-e04a-47d9-bca5-ed7101222dbf"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRCAMSInterParameter：RCAMS跨品种风险折抵参数

```
struct CThostFtdcRCAMSInterParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///商品群代码
    TThostFtdcProductIDType ProductGroupID;
    ///优先级
    TThostFtdcRCAMSPriorityType Priority;
    ///折抵率
    TThostFtdcHedgeRateType CreditRate;
    ///产品组合代码1
    TThostFtdcProductIDType CombProduct1;
    ///产品组合代码2
    TThostFtdcProductIDType CombProduct2;
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

<a id="5472a73b-0667-420b-af3b-1c3bb70aaa9a"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="2b944750-cb85-4b9e-9436-08c8db6a677d"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
