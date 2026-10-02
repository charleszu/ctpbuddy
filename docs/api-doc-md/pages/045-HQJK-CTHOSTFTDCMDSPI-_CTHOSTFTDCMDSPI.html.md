# CThostFtdcMdSpi

CThostFtdcMdSpi

|  | ■ 6.7.13_API接口说明
└△ 行情接口
　└◆ CThostFtdcMdSpi |  |
|---|---|---|

CthostFtdcMdSpi类提供了行情相关的回调接口，用户需要继承该类并重载这些接口，以获取响应数据。

◇ 1. 接口

```
class CThostFtdcMdSpi
{
public:
    ///当客户端与交易后台建立起通信连接时（还未登录前），该方法被调用。
    virtual void OnFrontConnected(){};
    ///当客户端与交易后台通信连接断开时，该方法被调用。当发生这个情况后，API会自动重新连接，客户端可不做处理。
    ///@param nReason 错误原因
    ///        0x1001 网络读失败
    ///        0x1002 网络写失败
    ///        0x2001 接收心跳超时
    ///        0x2002 发送心跳失败
    ///        0x2003 收到错误报文
    virtual void OnFrontDisconnected(int nReason){};
    ///心跳超时警告。当长时间未收到报文时，该方法被调用。
    ///@param nTimeLapse 距离上次接收报文的时间
    virtual void OnHeartBeatWarning(int nTimeLapse){};
    ///登录请求响应
    virtual void OnRspUserLogin(CThostFtdcRspUserLoginField *pRspUserLogin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///登出请求响应
    virtual void OnRspUserLogout(CThostFtdcUserLogoutField *pUserLogout, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询组播合约响应
    virtual void OnRspQryMulticastInstrument(CThostFtdcMulticastInstrumentField *pMulticastInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///错误应答
    virtual void OnRspError(CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///订阅行情应答
    virtual void OnRspSubMarketData(CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///取消订阅行情应答
    virtual void OnRspUnSubMarketData(CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///订阅询价应答
    virtual void OnRspSubForQuoteRsp(CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///取消订阅询价应答
    virtual void OnRspUnSubForQuoteRsp(CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///深度行情通知
    virtual void OnRtnDepthMarketData(CThostFtdcDepthMarketDataField *pDepthMarketData) {};
    ///询价通知
    virtual void OnRtnForQuoteRsp(CThostFtdcForQuoteRspField *pForQuoteRsp) {};
};

```

◇ 2. 示例代码

```
// CMduserHandler继承CThostFtdcMdSpi
class CMduserHandler : public CThostFtdcMdSpi
{
public:
      //重载，接收行情
       void OnRtnDepthMarketData(CThostFtdcDepthMarketDataField *pDepthMarketData)
       {
                     printf("OnRtnDepthMarketData\n");
       }
}

```

请参阅：

    [OnFrontConnected](ONFRONTCONNECTED.html)

    [OnFrontDisconnected](ONFRONTDISCONNECTED.html)

    [OnHeartBeatWarning](ONHEARTBEATWARNING.html)

    [OnRspError](ONRSPERROR.html)

    [OnRspQryMulticastInstrument](ONRSPQRYMULTICASTINSTRUMENT.html)

    [OnRspSubForQuoteRsp](ONRSPSUBFORQUOTERSP.html)

    [OnRspSubMarketData](ONRSPSUBMARKETDATA.html)

    [OnRspUnSubForQuoteRsp](ONRSPUNSUBFORQUOTERSP.html)

    [OnRspUnSubMarketData](ONRSPUNSUBMARKETDATA.html)

    [OnRspUserLogin](ONRSPUSERLOGIN.html)

    [OnRspUserLogout](ONRSPUSERLOGOUT.html)

    [OnRtnDepthMarketData](ONRTNDEPTHMARKETDATA.html)

    [OnRtnForQuoteRsp](ONRTNFORQUOTERSP.html)
