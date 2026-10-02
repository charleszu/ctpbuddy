# OnRspQryRULEInterParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryRULEInterParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RULE跨品种抵扣参数查询响应，当执行[ReqQryRULEInterParameter](pages/185-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRULEINTERPARAMETER.html.md)后，该方法被调用。
<a id="148168c4-2e63-4049-8faf-4a59f8cde3ac"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryRULEInterParameter(CThostFtdcRULEInterParameterField *pRULEInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="d8dd1d86-df2b-47b2-9990-2a58893b5e19"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pRULEInterParameter：RULE跨品种抵扣参数

```
struct CThostFtdcRULEInterParameterField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///优先级
    TThostFtdcSpreadIdType  SpreadId;
    ///品种间对锁仓费率折扣比例
    TThostFtdcRatioType InterRate;
    ///第一腿构成品种
    TThostFtdcInstrumentIDType  Leg1ProdFamilyCode;
    ///第二腿构成品种
    TThostFtdcInstrumentIDType  Leg2ProdFamilyCode;
    ///腿1比例系数
    TThostFtdcCommonIntType Leg1PropFactor;
    ///腿2比例系数
    TThostFtdcCommonIntType Leg2PropFactor;
    ///商品群号
    TThostFtdcCommodityGroupIDType  CommodityGroupID;
    ///商品群名称
    TThostFtdcInstrumentNameType    CommodityGroupName;
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

<a id="889b7e80-74af-432a-b632-ad25a98f91fd"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="2e4218ac-80af-4637-b0ee-b27fad6e253f"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
