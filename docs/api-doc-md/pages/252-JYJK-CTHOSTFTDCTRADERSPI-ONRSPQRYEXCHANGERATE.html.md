# OnRspQryExchangeRate

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryExchangeRate<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询汇率响应，当执行[ReqQryExchangeRate](pages/101-JYJK-CTHOSTFTDCTRADERAPI-REQQRYEXCHANGERATE.html.md)后，该方法被调用。
<a id="fbc89cb4-4be5-47c0-a04a-e12b44a0c78c"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryExchangeRate(CThostFtdcExchangeRateField *pExchangeRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="bd9d288b-2dee-4174-a4a5-952d2e9a0951"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pExchangeRate：汇率

```
struct CThostFtdcExchangeRateField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///源币种
    TThostFtdcCurrencyIDType FromCurrencyID;
    ///源币种单位数量
    TThostFtdcCurrencyUnitType FromCurrencyUnit;
    ///目标币种
    TThostFtdcCurrencyIDType ToCurrencyID;
    ///汇率
    TThostFtdcExchangeRateType ExchangeRate;
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

<a id="b7541a4c-0149-4c4c-9314-22c2eb99e7d0"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="06ee12b5-5f38-44a4-8560-f8c9cea443bb"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
