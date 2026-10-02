# OnRtnTradingNotice

OnRtnTradingNotice

交易通知，无Req属于私有流回报。

注：风控终端发送的风险通知由此接口接收。

◇ 1. 函数原型

virtual void OnRtnTradingNotice(CThostFtdcTradingNoticeInfoField *pTradingNoticeInfo) {};

◇ 2. 参数

pTradingNoticeInfo：用户事件通知信息

```
struct CThostFtdcTradingNoticeInfoField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType InvestorID;
    ///发送时间
    TThostFtdcTimeType SendTime;
    ///消息正文
    TThostFtdcContentType FieldContent;
    ///序列系列号
    TThostFtdcSequenceSeriesType SequenceSeries;
    ///序列号
    TThostFtdcSequenceNoType SequenceNo;
    ///投资单元代码
    TThostFtdcInvestUnitIDType InvestUnitID;
};

```

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
