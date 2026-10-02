# OnRspQryInvestorPortfSetting

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryInvestorPortfSetting<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者投资者新组保设置查询响应，当执行[ReqQryInvestorPortfSetting](pages/187-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPORTFSETTING.html.md)后，该方法被调用。
<a id="e2caa9c2-3f2e-45df-a8e3-1356a2f582e2"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryInvestorPortfSetting(CThostFtdcInvestorPortfSettingField *pInvestorPortfSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="6e64b472-5435-49a5-b3fd-1b14ae6d2fe7"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInvestorPortfSetting：投资者新组保设置

```
struct CThostFtdcInvestorPortfSettingField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者编号
    TThostFtdcInvestorIDType    InvestorID;
    ///投机套保标志
    TThostFtdcHedgeFlagType HedgeFlag;
    ///是否开启新组保
    TThostFtdcBoolType  UsePortf;
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

<a id="f869dd1a-4a4e-4578-8d72-035c9ca31529"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="e6e44114-eca1-42e0-9c41-c8eeda4d51c4"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
