# OnRspQrySPMMInstParam

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQrySPMMInstParam<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求SPMM合约参数查询响应，当执行[ReqQrySPMMInstParam](pages/173-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPMMINSTPARAM.html.md)后，该方法被调用。
<a id="d70bdf69-ff2f-4e59-a976-20943b1c8d7a"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQrySPMMInstParam(CThostFtdcSPMMInstParamField *pSPMMInstParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="78928762-53dd-45cb-8354-db6d5c1117e6"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pSPMMInstParam：SPMM合约参数

```
struct CThostFtdcSPMMInstParamField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///SPMM合约保证金算法
    TThostFtdcInstMarginCalIDType   InstMarginCalID;
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

<a id="b4b179d3-5fc6-4c34-a420-cc9dedbfea1a"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="7e981bcd-85d8-4b58-b1c7-65cd19115669"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
