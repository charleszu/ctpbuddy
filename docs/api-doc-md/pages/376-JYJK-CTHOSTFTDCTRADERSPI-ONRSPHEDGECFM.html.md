# OnRspHedgeCfm

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspHedgeCfm<a id="content"></a>

<a id="left_menu"></a>

  ** **

套保确认回复,当执行[ReqHedgeCfm](pages/200-JYJK-CTHOSTFTDCTRADERAPI-REQHEDGECFM.html.md)返回错误后，返回此接口
<a id="328c308e-2f7e-4f9d-80e7-7d9dd4d5214c"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspHedgeCfm(CThostFtdcInputHedgeCfmField *pInputHedgeCfm, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="c07f1aa3-c801-4542-9d81-0ee9b3fb9364"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputHedgeCfm：套保确认输入基本信息

```
struct CThostFtdcInputHedgeCfmField
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
    TThostFtdcInstrumentIDType  InstrumentID;
    ///数量
    TThostFtdcVolumeType    Volume;
    ///买卖方向
    TThostFtdcDirectionType Direction;
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

<a id="26ab45bd-e9d9-4985-9e7c-cfc497ea5d98"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

<a id="8b668f61-a426-44d7-9efa-1beba6f7c52b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
