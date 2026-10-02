# OnRtnCFMMCTradingAccountToken

OnRtnCFMMCTradingAccountToken

保证金监控中心用户令牌，当执行[ReqQueryCFMMCTradingAccountToken](../CTHOSTFTDCTRADERAPI/REQQUERYCFMMCTRADINGACCOUNTTOKEN.html)后并且报出后，收到返回则调用此接口，私有流回报。

◇ 1. 函数原型

virtual void OnRtnCFMMCTradingAccountToken(CThostFtdcCFMMCTradingAccountTokenField *pCFMMCTradingAccountToken) {};

◇ 2. 参数

pCFMMCTradingAccountToken：监控中心用户令牌

```
struct CThostFtdcCFMMCTradingAccountTokenField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///经纪公司统一编码
    TThostFtdcParticipantIDType ParticipantID;
    ///投资者帐号
    TThostFtdcAccountIDType AccountID;
    ///密钥编号
    TThostFtdcSequenceNoType KeyID;
    ///动态令牌
    TThostFtdcCFMMCTokenType Token;
};

```

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
