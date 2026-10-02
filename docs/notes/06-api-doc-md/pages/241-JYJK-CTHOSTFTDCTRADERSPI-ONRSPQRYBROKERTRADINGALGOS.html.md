# OnRspQryBrokerTradingAlgos

OnRspQryBrokerTradingAlgos

请求查询经纪公司交易算法响应，当执行[ReqQryBrokerTradingAlgos](../CTHOSTFTDCTRADERAPI/REQQRYBROKERTRADINGALGOS.html)后，该方法被调用

◇ 1. 函数原型

virtual void OnRspQryBrokerTradingAlgos(CThostFtdcBrokerTradingAlgosField *pBrokerTradingAlgos, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
