# OnRspQryBrokerTradingAlgos

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryBrokerTradingAlgos<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询经纪公司交易算法响应，当执行[ReqQryBrokerTradingAlgos](pages/090-JYJK-CTHOSTFTDCTRADERAPI-REQQRYBROKERTRADINGALGOS.html.md)后，该方法被调用
<a id="c8bdff5b-bfd5-48db-94d2-59cf11dab017"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryBrokerTradingAlgos(CThostFtdcBrokerTradingAlgosField *pBrokerTradingAlgos, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="d2f31712-0f02-41b5-bcb2-18065e5e27f1"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pBrokerTradingAlgos：经纪公司交易算法

```
struct CThostFtdcBrokerTradingAlgosField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///持仓处理算法编号
    TThostFtdcHandlePositionAlgoIDType  HandlePositionAlgoID;
    ///寻找保证金率算法编号
    TThostFtdcFindMarginRateAlgoIDType  FindMarginRateAlgoID;
    ///资金处理算法编号
    TThostFtdcHandleTradingAccountAlgoIDType    HandleTradingAccountAlgoID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
};

```

HandlePositionAlgoID：CTP内部使用

FindMarginRateAlgoID：CTP内部使用

HandleTradingAccountAlgoID：CTP内部使用

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

<a id="7d2c566b-f296-4cc1-a45c-5e6ed6ffcd2e"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="ce505501-a179-4688-83e5-8a1d72212021"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
