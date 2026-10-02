# OnErrRtnOffsetSetting

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnErrRtnOffsetSetting<a id="content"></a>

<a id="left_menu"></a>

  ** **

对冲设置错误回报，当执行[ReqOffsetSetting](pages/190-JYJK-CTHOSTFTDCTRADERAPI-REQOFFSETSETTING.html.md)返回错误后，该方法被调用。
<a id="4f956578-04f6-48e0-ac56-677bb29457f6"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnErrRtnOffsetSetting(CThostFtdcInputOffsetSettingField *pInputOffsetSetting, CThostFtdcRspInfoField *pRspInfo) {};

<a id="593a6719-445e-4228-a8b0-2402cf1c4bc0"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputOffsetSetting：输入的对冲设置

```
struct CThostFtdcInputOffsetSettingField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///标的期货合约代码
    TThostFtdcInstrumentIDType  UnderlyingInstrID;
    ///产品代码
    TThostFtdcProductIDType ProductID;
    ///对冲类型
    TThostFtdcOffsetTypeType    OffsetType;
    ///申请对冲的合约数量
    TThostFtdcVolumeType    Volume;
    ///是否对冲
    TThostFtdcBoolType  IsOffset;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///IP地址
    TThostFtdcIPAddressType IPAddress;
    ///Mac地址
    TThostFtdcMacAddressType    MacAddress;
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

<a id="a795241f-a89d-475f-8669-ec3cf093785f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="465fc3a7-750c-471c-8b2b-1afe0ddae522"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
