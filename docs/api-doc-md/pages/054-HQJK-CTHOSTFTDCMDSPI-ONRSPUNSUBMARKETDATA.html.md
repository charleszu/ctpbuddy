# OnRspUnSubMarketData

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspUnSubMarketData<a id="content"></a>

<a id="left_menu"></a>

  ** **

取消订阅行情应答，当[UnSubscribeMarketData](pages/044-HQJK-CTHOSTFTDCMDAPI-UNSUBSCRIBEMARKETDATA.html.md)后，调用此接口。
<a id="a849c178-fae1-47b1-b26e-e08fcb025a11"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspUnSubMarketData(CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="edc33381-4a5f-4a60-aca8-d606235d0f50"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pSpecificInstrument：指定的合约

```
struct CThostFtdcSpecificInstrumentField
{
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
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

<a id="23b2679f-7471-4661-92b1-8c6f22d7ebaa"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="1b2023a8-d41c-4dea-aff0-eff69cd3be57"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
