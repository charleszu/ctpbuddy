# OnRspQryCombLeg

OnRspQryCombLeg

请求组合腿信息查询响应，当执行[ReqQryCombLeg](../CTHOSTFTDCTRADERAPI/REQQRYCOMBLEG.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryCombLeg(CThostFtdcCombLegField *pCombLeg, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
