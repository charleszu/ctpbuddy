# OnRspQryCombLeg

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryCombLeg<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求组合腿信息查询响应，当执行[ReqQryCombLeg](pages/189-JYJK-CTHOSTFTDCTRADERAPI-REQQRYCOMBLEG.html.md)后，该方法被调用。
<a id="b4c05c7d-a7bb-41c2-85d1-4aa7bcaf318d"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryCombLeg(CThostFtdcCombLegField *pCombLeg, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="cfeeef1d-c248-4428-a048-0e32709cbcc2"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pCombLeg：组合腿信息

```
struct CThostFtdcCombLegField
{
    ///组合合约代码
    TThostFtdcInstrumentIDType  CombInstrumentID;
    ///单腿编号
    TThostFtdcLegIDType LegID;
    ///单腿合约代码
    TThostFtdcInstrumentIDType  LegInstrumentID;
    ///买卖方向
    TThostFtdcDirectionType Direction;
    ///单腿乘数
    TThostFtdcLegMultipleType   LegMultiple;
    ///派生层数
    TThostFtdcImplyLevelType    ImplyLevel;
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

LegMultiple：用于中金所组保业务，业务暂未上线

ImplyLevel：用于中金所组保业务，业务暂未上线

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="187876bc-f1f0-4e23-9fca-b200151acd7b"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="b8413781-7baa-410c-8040-d48d1e844e67"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
