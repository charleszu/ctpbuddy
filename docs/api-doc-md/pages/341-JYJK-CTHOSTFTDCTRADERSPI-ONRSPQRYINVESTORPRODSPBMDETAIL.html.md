# OnRspQryInvestorProdSPBMDetail

OnRspQryInvestorProdSPBMDetail

请求投资者产品SPBM明细查询响应，当执行[ReqQryInvestorProdSPBMDetail](../CTHOSTFTDCTRADERAPI/REQQRYINVESTORPRODSPBMDETAIL.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryInvestorProdSPBMDetail(CThostFtdcInvestorProdSPBMDetailField *pInvestorProdSPBMDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInvestorProdSPBMDetail：投资者产品SPBM明细

```
struct CThostFtdcInvestorProdSPBMDetailField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///品种代码
    TThostFtdcInstrumentIDType  ProdFamilyCode;
    ///合约内对锁保证金
    TThostFtdcMoneyType IntraInstrMargin;
    ///买归集保证金
    TThostFtdcMoneyType BCollectingMargin;
    ///卖归集保证金
    TThostFtdcMoneyType SCollectingMargin;
    ///品种内合约间对锁保证金
    TThostFtdcMoneyType IntraProdMargin;
    ///净保证金
    TThostFtdcMoneyType NetMargin;
    ///产品间对锁保证金
    TThostFtdcMoneyType InterProdMargin;
    ///裸保证金
    TThostFtdcMoneyType SingleMargin;
    ///附加保证金
    TThostFtdcMoneyType AddOnMargin;
    ///交割月保证金
    TThostFtdcMoneyType DeliveryMargin;
    ///看涨期权最低风险
    TThostFtdcMoneyType CallOptionMinRisk;
    ///看跌期权最低风险
    TThostFtdcMoneyType PutOptionMinRisk;
    ///卖方期权最低风险
    TThostFtdcMoneyType OptionMinRisk;
    ///买方期权冲抵价值
    TThostFtdcMoneyType OptionValueOffset;
    ///卖方期权权利金
    TThostFtdcMoneyType OptionRoyalty;
    ///价值冲抵
    TThostFtdcMoneyType RealOptionValueOffset;
    ///保证金
    TThostFtdcMoneyType Margin;
    ///交易所保证金
    TThostFtdcMoneyType ExchMargin;
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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
