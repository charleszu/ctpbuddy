# OnRspQryDepthMarketData

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryDepthMarketData<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询行情响应，当执行[ReqQryDepthMarketData](pages/096-JYJK-CTHOSTFTDCTRADERAPI-REQQRYDEPTHMARKETDATA.html.md)后，该方法被调用。
<a id="7ce7400e-06c6-454d-8fa7-a2e57e80f537"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryDepthMarketData(CThostFtdcDepthMarketDataField *pDepthMarketData, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="3cbeacf4-73f6-4ae6-b658-83b67f12200f"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pDepthMarketData：深度行情

```
struct CThostFtdcDepthMarketDataField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///保留的无效字段
    TThostFtdcOldExchangeInstIDType reserve2;
    ///最新价
    TThostFtdcPriceType LastPrice;
    ///上次结算价
    TThostFtdcPriceType PreSettlementPrice;
    ///昨收盘
    TThostFtdcPriceType PreClosePrice;
    ///昨持仓量
    TThostFtdcLargeVolumeType   PreOpenInterest;
    ///今开盘
    TThostFtdcPriceType OpenPrice;
    ///最高价
    TThostFtdcPriceType HighestPrice;
    ///最低价
    TThostFtdcPriceType LowestPrice;
    ///数量
    TThostFtdcVolumeType    Volume;
    ///成交金额
    TThostFtdcMoneyType Turnover;
    ///持仓量
    TThostFtdcLargeVolumeType   OpenInterest;
    ///今收盘
    TThostFtdcPriceType ClosePrice;
    ///本次结算价
    TThostFtdcPriceType SettlementPrice;
    ///涨停板价
    TThostFtdcPriceType UpperLimitPrice;
    ///跌停板价
    TThostFtdcPriceType LowerLimitPrice;
    ///昨虚实度
    TThostFtdcRatioType PreDelta;
    ///今虚实度
    TThostFtdcRatioType CurrDelta;
    ///最后修改时间
    TThostFtdcTimeType  UpdateTime;
    ///最后修改毫秒
    TThostFtdcMillisecType  UpdateMillisec;
    ///申买价一
    TThostFtdcPriceType BidPrice1;
    ///申买量一
    TThostFtdcVolumeType    BidVolume1;
    ///申卖价一
    TThostFtdcPriceType AskPrice1;
    ///申卖量一
    TThostFtdcVolumeType    AskVolume1;
    ///申买价二
    TThostFtdcPriceType BidPrice2;
    ///申买量二
    TThostFtdcVolumeType    BidVolume2;
    ///申卖价二
    TThostFtdcPriceType AskPrice2;
    ///申卖量二
    TThostFtdcVolumeType    AskVolume2;
    ///申买价三
    TThostFtdcPriceType BidPrice3;
    ///申买量三
    TThostFtdcVolumeType    BidVolume3;
    ///申卖价三
    TThostFtdcPriceType AskPrice3;
    ///申卖量三
    TThostFtdcVolumeType    AskVolume3;
    ///申买价四
    TThostFtdcPriceType BidPrice4;
    ///申买量四
    TThostFtdcVolumeType    BidVolume4;
    ///申卖价四
    TThostFtdcPriceType AskPrice4;
    ///申卖量四
    TThostFtdcVolumeType    AskVolume4;
    ///申买价五
    TThostFtdcPriceType BidPrice5;
    ///申买量五
    TThostFtdcVolumeType    BidVolume5;
    ///申卖价五
    TThostFtdcPriceType AskPrice5;
    ///申卖量五
    TThostFtdcVolumeType    AskVolume5;
    ///当日均价
    TThostFtdcPriceType AveragePrice;
    ///业务日期
    TThostFtdcDateType  ActionDay;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///合约在交易所的代码
    TThostFtdcExchangeInstIDType    ExchangeInstID;
    ///上带价
    TThostFtdcPriceType BandingUpperPrice;
    ///下带价
    TThostFtdcPriceType BandingLowerPrice;
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

Turnover：成交金额。自6.7.2版本开始，普通行情前置front的郑商所成交金额从原来的【成交均价*成交数量】改为【成交均价*成交数量*合约乘数】，与其他交易所的算法保持一致（【成交均价*成交数量*合约乘数】）

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="6f77ab54-dcda-4246-8cb5-4b28238d622d"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="fd231e01-8539-4dcb-97a6-8ed1520c0156"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
