# OnRtnForQuoteRsp

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRtnForQuoteRsp<a id="content"></a>

<a id="left_menu"></a>

  ** **

询价通知，使用[SubscribeForQuoteRsp](pages/041-HQJK-CTHOSTFTDCMDAPI-SUBSCRIBEFORQUOTERSP.html.md)订阅该询价通知。私有流回报。

详见[做市商询价和报价](pages/388-QTYWGZ-BJHXJ.html.md)
<a id="02604136-ceb3-410b-8dfb-c7eb72ba3c23"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [OnRtnForQuoteRsp](pages/309-JYJK-CTHOSTFTDCTRADERSPI-ONRTNFORQUOTERSP.html.md)(CThostFtdcForQuoteRspField *pForQuoteRsp) {};

<a id="9cb1935f-f969-4543-ba50-4a47b66757a2"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pForQuoteRsp：发给做市商的询价请求

```
struct CThostFtdcForQuoteRspField
{
    ///交易日
    TThostFtdcDateType TradingDay;
    ///合约代码
    TThostFtdcInstrumentIDType InstrumentID;
    ///询价编号
    TThostFtdcOrderSysIDType ForQuoteSysID;
    ///询价时间
    TThostFtdcTimeType ForQuoteTime;
    ///业务日期
    TThostFtdcDateType ActionDay;
    ///交易所代码
    TThostFtdcExchangeIDType ExchangeID;
};

```

ForQuoteSysID：交易所给的一个询价编号，以此定位一笔询价。

<a id="2c7e40ac-5edf-4439-91e8-abe27510455f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="753caa1f-dffb-4637-929a-270f38f3d7d9"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
