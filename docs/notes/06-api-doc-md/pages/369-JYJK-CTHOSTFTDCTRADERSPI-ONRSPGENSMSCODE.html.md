# OnRspGenSMSCode

OnRspGenSMSCode

请求申请短信验证码响应,当执行[ReqGenSMSCode](../CTHOSTFTDCTRADERAPI/REQGENSMSCODE.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspGenSMSCode(CThostFtdcRspGenSMSCodeField *pRspGenSMSCode, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pRspGenSMSCode：申请短信验证码响应

```
struct CThostFtdcRspGenSMSCodeField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///生成时间
    TThostFtdcTimeType  GenTime;
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

◇ 4. FAQ

无
