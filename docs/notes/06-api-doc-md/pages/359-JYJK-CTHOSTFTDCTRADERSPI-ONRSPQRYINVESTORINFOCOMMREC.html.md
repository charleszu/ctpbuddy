# OnRspQryInvestorInfoCommRec

OnRspQryInvestorInfoCommRec

请求投资者申报费阶梯收取记录查询响应，当执行[ReqQryInvestorInfoCommRec](../CTHOSTFTDCTRADERAPI/REQQRYINVESTORINFOCOMMREC.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryInvestorInfoCommRec(CThostFtdcInvestorInfoCommRecField *pInvestorInfoCommRec, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pInvestorInfoCommRec：投资者申报费阶梯收取记录

```
struct CThostFtdcInvestorInfoCommRecField
{
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///商品代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///报单总笔数
    TThostFtdcVolumeType    OrderCount;
    ///撤单总笔数
    TThostFtdcVolumeType    OrderActionCount;
    ///询价总次数
    TThostFtdcVolumeType    ForQuoteCnt;
    ///申报费
    TThostFtdcMoneyType InfoComm;
    ///是否期权系列
    TThostFtdcBoolType  IsOptSeries;
    ///品种代码
    TThostFtdcProductIDType ProductID;
    ///信息量总量
    TThostFtdcVolumeType    InfoCnt;
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
