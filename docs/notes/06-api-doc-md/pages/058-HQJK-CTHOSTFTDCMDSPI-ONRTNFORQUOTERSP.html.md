# OnRtnForQuoteRsp

OnRtnForQuoteRsp

询价通知，使用[SubscribeForQuoteRsp](../CTHOSTFTDCMDAPI/SUBSCRIBEFORQUOTERSP.html)订阅该询价通知。私有流回报。

详见[做市商询价和报价](../../QTYWGZ/BJHXJ.html)

◇ 1. 函数原型

virtual void [OnRtnForQuoteRsp](../../JYJK/CTHOSTFTDCTRADERSPI/ONRTNFORQUOTERSP.html)(CThostFtdcForQuoteRspField *pForQuoteRsp) {};

◇ 2. 参数

pForQuoteRsp：发给做市商的询价请求

```
struct CThostFtdcForQuoteRspField
{
    ///交易日
    TThostFtdcDateType TradingDay;
    ///合约代码
    TThostFtdcInstrumentIDType InstrumentID;
    ///询价编号
    TThostFtdcOrderSysIDType ForQuoteSysID;
    ///询价时间
    TThostFtdcTimeType ForQuoteTime;
    ///业务日期
    TThostFtdcDateType ActionDay;
    ///交易所代码
    TThostFtdcExchangeIDType ExchangeID;
};

```

ForQuoteSysID：交易所给的一个询价编号，以此定位一笔询价。

◇ 3. 返回

无

◇ 4. FAQ

无
