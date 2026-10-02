# OnRspQryTraderOffer

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryTraderOffer<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询交易员报盘机响应，当执行[ReqQryTraderOffer](pages/162-JYJK-CTHOSTFTDCTRADERAPI-REQQRYTRADEROFFER.html.md)后，该方法被调用。
<a id="7cee78fd-6f6b-44fc-9c0b-c1ba002c824b"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryTraderOffer(CThostFtdcTraderOfferField *pTraderOffer, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="f5480c4b-480e-4381-8df5-7ddb1711371a"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pTraderOffer：交易所交易员报盘机

```
struct CThostFtdcTraderOfferField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///交易所交易员代码
    TThostFtdcTraderIDType  TraderID;
    ///会员代码
    TThostFtdcParticipantIDType ParticipantID;
    ///密码
    TThostFtdcPasswordType  Password;
    ///安装编号
    TThostFtdcInstallIDType InstallID;
    ///本地报单编号
    TThostFtdcOrderLocalIDType  OrderLocalID;
    ///交易所交易员连接状态
    TThostFtdcTraderConnectStatusType   TraderConnectStatus;
    ///发出连接请求的日期
    TThostFtdcDateType  ConnectRequestDate;
    ///发出连接请求的时间
    TThostFtdcTimeType  ConnectRequestTime;
    ///上次报告日期
    TThostFtdcDateType  LastReportDate;
    ///上次报告时间
    TThostFtdcTimeType  LastReportTime;
    ///完成连接日期
    TThostFtdcDateType  ConnectDate;
    ///完成连接时间
    TThostFtdcTimeType  ConnectTime;
    ///启动日期
    TThostFtdcDateType  StartDate;
    ///启动时间
    TThostFtdcTimeType  StartTime;
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///本席位最大成交编号
    TThostFtdcTradeIDType   MaxTradeID;
    ///本席位最大报单备拷
    TThostFtdcReturnCodeType    MaxOrderMessageReference;
    ///撤单时选择席位算法
    TThostFtdcOrderCancelAlgType    OrderCancelAlg;
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

<a id="1d0ba753-3fdf-4a93-a46f-f6448e20911c"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="7aac3391-097a-4faf-afe5-a295f25dc259"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
