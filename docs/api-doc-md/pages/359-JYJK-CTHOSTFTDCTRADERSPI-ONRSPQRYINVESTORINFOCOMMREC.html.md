# OnRspQryInvestorInfoCommRec

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryInvestorInfoCommRec<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求投资者申报费阶梯收取记录查询响应，当执行[ReqQryInvestorInfoCommRec](pages/188-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORINFOCOMMREC.html.md)后，该方法被调用。
<a id="121f6b63-60aa-4ae4-ae36-4ba2cdedf0e0"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryInvestorInfoCommRec(CThostFtdcInvestorInfoCommRecField *pInvestorInfoCommRec, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="604112ec-7844-468d-bf34-4021d9854277"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

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

<a id="1423cc30-7f61-4b96-bfd8-20debc0d218a"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="679c7bda-c962-4129-9803-3151d9456682"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
