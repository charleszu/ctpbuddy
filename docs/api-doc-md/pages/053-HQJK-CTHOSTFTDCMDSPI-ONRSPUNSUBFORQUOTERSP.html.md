# OnRspUnSubForQuoteRsp

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspUnSubForQuoteRsp<a id="content"></a>

<a id="left_menu"></a>

  ** **

取消订阅询价应答，当[UnSubscribeForQuoteRsp](pages/043-HQJK-CTHOSTFTDCMDAPI-UNSUBSCRIBEFORQUOTERSP.html.md)后，调用此接口。
<a id="21cee6f2-ac5a-42ed-b4c9-8d12e7936cc7"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspUnSubForQuoteRsp(CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="38f41317-8291-4ccb-a6ed-f83b531769cf"></a><a id="title2"></a>

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

<a id="d7839ea3-0665-42a6-8232-ae7eb68769e1"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="50b0feda-ec57-4365-88ac-38acc74fe31e"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
