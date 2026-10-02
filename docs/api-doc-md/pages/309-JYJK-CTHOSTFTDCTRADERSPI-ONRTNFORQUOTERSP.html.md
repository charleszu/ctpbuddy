# OnRtnForQuoteRsp

OnRtnForQuoteRsp

询价通知，此接口暂时不用，目前使用的是行情的OnRtnForQuoteRsp。

详见[做市商询价和报价](../../QTYWGZ/BJHXJ.html)

◇ 1. 函数原型

virtual void OnRtnForQuoteRsp(CThostFtdcForQuoteRspField *pForQuoteRsp) {};

◇ 2. 参数

pForQuoteRsp：发给做市商的询价请求

```
struct CThostFtdcForQuoteRspField
{
    ///交易日
    TThostFtdcDateType  TradingDay;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///询价编号
    TThostFtdcOrderSysIDType    ForQuoteSysID;
    ///询价时间
    TThostFtdcTimeType  ForQuoteTime;
    ///业务日期
    TThostFtdcDateType  ActionDay;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
};

```

ForQuoteSysID：交易所给的一个询价编号，以此定位一笔询价。

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
