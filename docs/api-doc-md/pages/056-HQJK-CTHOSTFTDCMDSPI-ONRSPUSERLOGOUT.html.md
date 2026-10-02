# OnRspUserLogout

OnRspUserLogout

登出请求响应，当[ReqUserLogout](../CTHOSTFTDCMDAPI/REQUSERLOGOUT.html)后，该方法被调用。

◇ 1. 函数原型

virtual void [OnRspUserLogout](../../JYJK/CTHOSTFTDCTRADERSPI/ONRSPUSERLOGOUT.html)(CThostFtdcUserLogoutField *pUserLogout, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pUserLogout：用户登出请求

```
struct CThostFtdcUserLogoutField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///用户代码
    TThostFtdcUserIDType UserID;
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

nRequestID：返回用户操作请求的ID，该ID由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

◇ 3. 返回

无

◇ 4. FAQ

无
