# OnErrRtnCancelOffsetSetting

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnErrRtnCancelOffsetSetting<a id="content"></a>

<a id="left_menu"></a>

  ** **

对冲设置撤销错误回报，当执行[ReqCancelOffsetSetting](pages/191-JYJK-CTHOSTFTDCTRADERAPI-REQCANCELOFFSETSETTING.html.md)返回错误后，该方法被调用。
<a id="01914b89-2396-4055-84bd-626171e48e76"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnErrRtnCancelOffsetSetting(CThostFtdcCancelOffsetSettingField *pCancelOffsetSetting, CThostFtdcRspInfoField *pRspInfo) {};

<a id="b5893bf7-2e5d-46dd-86d7-19bcf72409d3"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pCancelOffsetSetting：撤销对冲设置

```
struct CThostFtdcCancelOffsetSettingField
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
    ///交易所合约代码
    TThostFtdcExchangeInstIDType    ExchangeInstID;
    ///交易所期权系列号
    TThostFtdcExchangeInstIDType    ExchangeSerialNo;
    ///交易所产品代码
    TThostFtdcProductIDType ExchangeProductID;
    ///交易所交易员代码
    TThostFtdcTraderIDType  TraderID;
    ///安装编号
    TThostFtdcInstallIDType InstallID;
    ///会员代码
    TThostFtdcParticipantIDType ParticipantID;
    ///客户代码
    TThostFtdcClientIDType  ClientID;
    ///报单操作状态
    TThostFtdcOrderActionStatusType OrderActionStatus;
    ///状态信息
    TThostFtdcErrorMsgType  StatusMsg;
    ///操作本地编号
    TThostFtdcOrderLocalIDType  ActionLocalID;
    ///操作日期
    TThostFtdcDateType  ActionDate;
    ///操作时间
    TThostFtdcTimeType  ActionTime;
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

<a id="6e69bc7a-e0cb-4e6a-940c-6307e969aab0"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="00af15d0-1003-42e5-8602-fef42e85ff55"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
