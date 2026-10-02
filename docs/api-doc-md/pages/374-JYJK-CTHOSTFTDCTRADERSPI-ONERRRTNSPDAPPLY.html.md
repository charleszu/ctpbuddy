# OnErrRtnSpdApply

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnErrRtnSpdApply<a id="content"></a>

<a id="left_menu"></a>

  ** **

套利申请录入错误回报，当执行[ReqSpdApply](pages/197-JYJK-CTHOSTFTDCTRADERAPI-REQSPDAPPLY.html.md)报错时返回此接口
<a id="a8fc3228-cca6-4df4-ac48-8cbdb304e877"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnErrRtnSpdApply(CThostFtdcInputSpdApplyField *pInputSpdApply, CThostFtdcRspInfoField *pRspInfo) {};

<a id="a8293c57-288c-4cd6-a553-38cb130d8a16"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputSpdApply：套利确认输入基本信息

```
struct CThostFtdcInputSpdApplyField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合约代码
    TThostFtdcInstrumentIDType  FirstLegInstrumentID;
    ///合约代码
    TThostFtdcInstrumentIDType  SecondLegInstrumentID;
    ///数量
    TThostFtdcVolumeType    Volume;
    ///买卖方向
    TThostFtdcDirectionType Direction;
    ///组合定单类型
    TThostFtdcCmbTypeType   CmbType;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///报单引用
    TThostFtdcOrderRefType  OrderRef;
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

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="1480a7b2-df10-43fe-85b6-d4938ad18f56"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

<a id="5d9ed376-21b3-4153-a13c-831a141e784f"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
