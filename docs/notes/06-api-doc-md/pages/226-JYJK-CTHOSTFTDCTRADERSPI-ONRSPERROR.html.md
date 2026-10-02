# OnRspError

OnRspError

针对用户请求的出错通知。

◇ 1. 函数原型

virtual void [OnRspError](../../HQJK/CTHOSTFTDCMDSPI/ONRSPERROR.html)(CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

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

查询时遇到报“[OnRspError](../../HQJK/CTHOSTFTDCMDSPI/ONRSPERROR.html)[90]: CTP：查询未就绪，请稍后重试”

| 前置返回的超流控报错，详情见：报单流控、查询流控和会话数控制。 |
|---|
