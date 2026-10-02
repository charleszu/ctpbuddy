# OnErrRtnHedgeCfm

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnErrRtnHedgeCfm<a id="content"></a>

<a id="left_menu"></a>

  ** **

套保额度录入错误回报,当执行[ReqHedgeCfm](pages/200-JYJK-CTHOSTFTDCTRADERAPI-REQHEDGECFM.html.md)后，如果返回错误，通过此接口返回
<a id="0ea32f8c-9d4f-4791-98f8-06be6caa71fd"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnErrRtnHedgeCfm(CThostFtdcInputHedgeCfmField *pInputHedgeCfm, CThostFtdcRspInfoField *pRspInfo) {};

<a id="b848a62e-d7d3-402e-96cb-c1820de633de"></a><a id="title2"></a>

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

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="97f5cc8d-7472-4d1f-8bc3-a826452f68a4"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

<a id="41bf5a91-d265-4c02-91f2-7bc94bd6dd4b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
