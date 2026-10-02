# OnRspQryInvestorPortfMarginRatio

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryInvestorPortfMarginRatio<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者新型组合保证金系数查询响应，当执行[ReqQryInvestorPortfMarginRatio](pages/169-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPORTFMARGINRATIO.html.md)后，该方法被调用。
<a id="2820e096-0d01-4dab-8e53-fdaf555f271d"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryInvestorPortfMarginRatio(CThostFtdcInvestorPortfMarginRatioField *pInvestorPortfMarginRatio, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="09292549-a21e-4a0d-8cb4-d4aff9ab91f9"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInvestorPortfMarginRatio：投资者新型组合保证金系数

```
struct CThostFtdcInvestorPortfMarginRatioField
{
    ///投资者范围
    TThostFtdcInvestorRangeType InvestorRange;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///会员对投资者收取的保证金和交易所对投资者收取的保证金的比例
    TThostFtdcRatioType MarginRatio;
    ///产品群代码
    TThostFtdcProductIDType ProductGroupID;
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

<a id="9b5dceda-ab6d-47d2-a28d-019a7a7a2e7f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="7b1df055-2335-4e0b-8606-f4613eeb5958"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
