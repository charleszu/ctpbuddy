# OnRspQryInvestorPortfSetting

OnRspQryInvestorPortfSetting

请求投资者投资者新组保设置查询响应，当执行[ReqQryInvestorPortfSetting](../CTHOSTFTDCTRADERAPI/REQQRYINVESTORPORTFSETTING.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryInvestorPortfSetting(CThostFtdcInvestorPortfSettingField *pInvestorPortfSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInvestorPortfSetting：投资者新组保设置

```
struct CThostFtdcInvestorPortfSettingField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者编号
    TThostFtdcInvestorIDType    InvestorID;
    ///投机套保标志
    TThostFtdcHedgeFlagType HedgeFlag;
    ///是否开启新组保
    TThostFtdcBoolType  UsePortf;
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

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
