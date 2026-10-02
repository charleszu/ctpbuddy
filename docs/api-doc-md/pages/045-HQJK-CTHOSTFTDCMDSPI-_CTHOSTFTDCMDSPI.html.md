# CThostFtdcMdSpi

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

CThostFtdcMdSpi<a id="content"></a>

<a id="left_menu"></a>

  ** **

|  | ■ 6.7.13_API接口说明
└△ 行情接口
　└◆ CThostFtdcMdSpi |  |
|---|---|---|

CthostFtdcMdSpi类提供了行情相关的回调接口，用户需要继承该类并重载这些接口，以获取响应数据。
<a id="7ef1aa1f-b6ee-4e9d-8568-9bab21c8f852"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 接口
<a id="panel1"></a>

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

<a id="a95c33ef-3ae8-4de5-a971-6a12181d430f"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 示例代码
<a id="panel2"></a>

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

    [OnFrontConnected](pages/046-HQJK-CTHOSTFTDCMDSPI-ONFRONTCONNECTED.html.md)

    [OnFrontDisconnected](pages/047-HQJK-CTHOSTFTDCMDSPI-ONFRONTDISCONNECTED.html.md)

    [OnHeartBeatWarning](pages/048-HQJK-CTHOSTFTDCMDSPI-ONHEARTBEATWARNING.html.md)

    [OnRspError](pages/049-HQJK-CTHOSTFTDCMDSPI-ONRSPERROR.html.md)

    [OnRspQryMulticastInstrument](pages/050-HQJK-CTHOSTFTDCMDSPI-ONRSPQRYMULTICASTINSTRUMENT.html.md)

    [OnRspSubForQuoteRsp](pages/051-HQJK-CTHOSTFTDCMDSPI-ONRSPSUBFORQUOTERSP.html.md)

    [OnRspSubMarketData](pages/052-HQJK-CTHOSTFTDCMDSPI-ONRSPSUBMARKETDATA.html.md)

    [OnRspUnSubForQuoteRsp](pages/053-HQJK-CTHOSTFTDCMDSPI-ONRSPUNSUBFORQUOTERSP.html.md)

    [OnRspUnSubMarketData](pages/054-HQJK-CTHOSTFTDCMDSPI-ONRSPUNSUBMARKETDATA.html.md)

    [OnRspUserLogin](pages/055-HQJK-CTHOSTFTDCMDSPI-ONRSPUSERLOGIN.html.md)

    [OnRspUserLogout](pages/056-HQJK-CTHOSTFTDCMDSPI-ONRSPUSERLOGOUT.html.md)

    [OnRtnDepthMarketData](pages/057-HQJK-CTHOSTFTDCMDSPI-ONRTNDEPTHMARKETDATA.html.md)

    [OnRtnForQuoteRsp](pages/058-HQJK-CTHOSTFTDCMDSPI-ONRTNFORQUOTERSP.html.md)

<a id="author"></a>

<a id="theme_switcher"></a>
