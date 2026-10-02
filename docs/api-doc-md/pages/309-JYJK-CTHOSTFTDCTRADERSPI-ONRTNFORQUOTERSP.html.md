# OnRtnForQuoteRsp

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRtnForQuoteRsp<a id="content"></a>

<a id="left_menu"></a>

  ** **

询价通知，此接口暂时不用，目前使用的是行情的OnRtnForQuoteRsp。

详见[做市商询价和报价](pages/388-QTYWGZ-BJHXJ.html.md)
<a id="9a66c88a-45b3-4a65-8ab1-8cf7888b4e90"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRtnForQuoteRsp(CThostFtdcForQuoteRspField *pForQuoteRsp) {};

<a id="3c38ebed-4cf3-47b8-8360-c2c06c24e0f7"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pForQuoteRsp：发给做市商的询价请求

```
struct CThostFtdcForQuoteRspField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///询价编号
    TThostFtdcOrderSysIDType    ForQuoteSysID;
    ///询价时间
    TThostFtdcTimeType  ForQuoteTime;
    ///业务日期
    TThostFtdcDateType  ActionDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
};

```

ForQuoteSysID：交易所给的一个询价编号，以此定位一笔询价。

<a id="fcec88c5-6c43-4b01-8ed0-d18ee06115f4"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="826bc692-914d-43c2-b002-85d9f07189bc"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
