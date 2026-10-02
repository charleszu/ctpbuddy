# OnRspQrySPMMProductParam

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQrySPMMProductParam<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求SPMM产品参数查询响应，当执行[ReqQrySPMMProductParam](pages/174-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPMMPRODUCTPARAM.html.md)后，该方法被调用。
<a id="a2b77a35-b983-480c-9895-639395113739"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQrySPMMProductParam(CThostFtdcSPMMProductParamField *pSPMMProductParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="dad7c59c-a85b-4d2d-9ecb-08d86ae9ec01"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pSPMMProductParam：SPMM产品参数

```
struct CThostFtdcSPMMProductParamField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///产品代码
    TThostFtdcSPMMProductIDType ProductID;
    ///商品组代码
    TThostFtdcSPMMProductIDType CommodityID;
    ///商品群代码
    TThostFtdcSPMMProductIDType CommodityGroupID;
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

<a id="b3a25f98-b901-481d-b40b-48dd14363dd6"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="b8c48bc1-b926-4e2d-8d50-bb2f8f4e72ce"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
