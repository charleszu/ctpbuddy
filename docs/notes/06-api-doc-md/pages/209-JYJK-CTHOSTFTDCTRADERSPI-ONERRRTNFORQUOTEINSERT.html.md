# OnErrRtnForQuoteInsert

OnErrRtnForQuoteInsert

询价录入错误回报，当执行[ReqForQuoteInsert](../CTHOSTFTDCTRADERAPI/REQFORQUOTEINSERT.html)后有字段填写不对之类的CTP报错则通过此接口返回

详见[做市商询价和报价](../../QTYWGZ/BJHXJ.html)

◇ 1. 函数原型

virtual void OnErrRtnForQuoteInsert(CThostFtdcInputForQuoteField *pInputForQuote, CThostFtdcRspInfoField *pRspInfo) {};

◇ 2. 参数

pInputForQuote：输入的询价

```
struct CThostFtdcInputForQuoteField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///询价引用
    TThostFtdcOrderRefType  ForQuoteRef;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///投资单元代码
    TThostFtdcInvestUnitIDType  InvestUnitID;
    ///保留的无效字段
    TThostFtdcOldIPAddressType  reserve2;
    ///Mac地址
    TThostFtdcMacAddressType    MacAddress;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///IP地址
    TThostFtdcIPAddressType IPAddress;
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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
