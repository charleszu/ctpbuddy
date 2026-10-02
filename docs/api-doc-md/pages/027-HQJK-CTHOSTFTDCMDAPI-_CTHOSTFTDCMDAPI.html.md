# CThostFtdcMdApi

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

CThostFtdcMdApi<a id="content"></a>

<a id="left_menu"></a>

  ** **

|  | ■ 6.7.13_API接口说明
└△ 行情接口
　└◆ CThostFtdcMdApi |  |
|---|---|---|

CthostFtdcMdApi类提供了行情api的初始化、登录、订阅等功能。
<a id="bc2d6539-c2f4-4566-a3a4-ff29e6e5ab6f"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 接口
<a id="panel1"></a>

```
class MD_API_EXPORT CThostFtdcMdApi
{
public:
    ///创建MdApi
    ///@param pszFlowPath 存贮订阅信息文件的目录，默认为当前目录
    ///@return 创建出的UserApi
    ///modify for udp marketdata
    static CThostFtdcMdApi *CreateFtdcMdApi(const char *pszFlowPath = "", const bool bIsUsingUdp=false, const bool bIsMulticast=false, bool bIsProductionMode=true);
    ///获取API的版本信息
    ///@retrun 获取到的版本号
    static const char *GetApiVersion();
    ///删除接口对象本身
    ///@remark 不再使用本接口对象时,调用该函数删除接口对象
    virtual void Release() = 0;
    ///初始化
    ///@remark 初始化运行环境,只有调用后,接口才开始工作
    virtual void Init() = 0;
    ///等待接口线程结束运行
    ///@return 线程退出代码
    virtual int Join() = 0;
    ///获取当前交易日
    ///@retrun 获取到的交易日
    ///@remark 只有登录成功后,才能得到正确的交易日
    virtual const char *GetTradingDay() = 0;
    ///注册前置机网络地址
    ///@param pszFrontAddress：前置机网络地址。
    ///@remark 网络地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:17001”。
    ///@remark “tcp”代表传输协议，“127.0.0.1”代表服务器地址。”17001”代表服务器端口号。
    virtual void RegisterFront(char *pszFrontAddress) = 0;
    ///注册名字服务器网络地址
    ///@param pszNsAddress：名字服务器网络地址。
    ///@remark 网络地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:12001”。
    ///@remark “tcp”代表传输协议，“127.0.0.1”代表服务器地址。”12001”代表服务器端口号。
    ///@remark RegisterNameServer优先于RegisterFront
    virtual void RegisterNameServer(char *pszNsAddress) = 0;
    ///注册名字服务器用户信息
    ///@param pFensUserInfo：用户信息。
    virtual void RegisterFensUserInfo(CThostFtdcFensUserInfoField * pFensUserInfo) = 0;
    ///注册回调接口
    ///@param pSpi 派生自回调接口类的实例
    virtual void RegisterSpi(CThostFtdcMdSpi *pSpi) = 0;
    ///订阅行情。
    ///@param ppInstrumentID 合约ID
    ///@param nCount 要订阅/退订行情的合约个数
    ///@remark
    virtual int SubscribeMarketData(char *ppInstrumentID[], int nCount) = 0;
    ///退订行情。
    ///@param ppInstrumentID 合约ID
    ///@param nCount 要订阅/退订行情的合约个数
    ///@remark
    virtual int UnSubscribeMarketData(char *ppInstrumentID[], int nCount) = 0;
    ///订阅询价。
    ///@param ppInstrumentID 合约ID
    ///@param nCount 要订阅/退订行情的合约个数
    ///@remark
    virtual int SubscribeForQuoteRsp(char *ppInstrumentID[], int nCount) = 0;
    ///退订询价。
    ///@param ppInstrumentID 合约ID
    ///@param nCount 要订阅/退订行情的合约个数
    ///@remark
    virtual int UnSubscribeForQuoteRsp(char *ppInstrumentID[], int nCount) = 0;
    ///用户登录请求
    virtual int ReqUserLogin(CThostFtdcReqUserLoginField *pReqUserLoginField, int nRequestID) = 0;
    ///登出请求
    virtual int ReqUserLogout(CThostFtdcUserLogoutField *pUserLogout, int nRequestID) = 0;
    ///请求查询组播合约
    virtual int ReqQryMulticastInstrument(CThostFtdcQryMulticastInstrumentField *pQryMulticastInstrument, int nRequestID) = 0;
protected:
    ~CThostFtdcMdApi(){};
};

```

<a id="74e58a2a-c9b1-4b09-8a55-ca581f053126"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 示例代码
<a id="panel2"></a>

```
class CMduserHandler : public CThostFtdcMdSpi
{
private:
    CThostFtdcMdApi *m_mdApi;
public:
    void connect()
    {
        //创建并初始化API，按照以下顺序
        m_mdApi = CThostFtdcMdApi::CreateFtdcMdApi("", true, true);
        m_mdApi->RegisterSpi(this);
        m_mdApi->RegisterFront("tcp://127.0.0.1:41413");
        m_mdApi->Init();
    }
}

```

请参阅：

    [CreateFtdcMdApi](pages/028-HQJK-CTHOSTFTDCMDAPI-CREATEFTDCMDAPI.html.md)

    [GetApiVersion](pages/029-HQJK-CTHOSTFTDCMDAPI-GETAPIVERSION.html.md)

    [GetTradingDay](pages/030-HQJK-CTHOSTFTDCMDAPI-GETTRADINGDAY.html.md)

    [Init](pages/031-HQJK-CTHOSTFTDCMDAPI-INIT.html.md)

    [Join](pages/032-HQJK-CTHOSTFTDCMDAPI-JOIN.html.md)

    [RegisterFensUserInfo](pages/033-HQJK-CTHOSTFTDCMDAPI-REGISTERFENSUSERINFO.html.md)

    [RegisterFront](pages/034-HQJK-CTHOSTFTDCMDAPI-REGISTERFRONT.html.md)

    [RegisterNameServer](pages/035-HQJK-CTHOSTFTDCMDAPI-REGISTERNAMESERVER.html.md)

    [RegisterSpi](pages/036-HQJK-CTHOSTFTDCMDAPI-REGISTERSPI.html.md)

    [Release](pages/037-HQJK-CTHOSTFTDCMDAPI-RELEASE.html.md)

    [ReqQryMulticastInstrument](pages/038-HQJK-CTHOSTFTDCMDAPI-REQQRYMULTICASTINSTRUMENT.html.md)

    [ReqUserLogin](pages/039-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGIN.html.md)

    [ReqUserLogout](pages/040-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGOUT.html.md)

    [SubscribeForQuoteRsp](pages/041-HQJK-CTHOSTFTDCMDAPI-SUBSCRIBEFORQUOTERSP.html.md)

    [SubscribeMarketData](pages/042-HQJK-CTHOSTFTDCMDAPI-SUBSCRIBEMARKETDATA.html.md)

    [UnSubscribeForQuoteRsp](pages/043-HQJK-CTHOSTFTDCMDAPI-UNSUBSCRIBEFORQUOTERSP.html.md)

    [UnSubscribeMarketData](pages/044-HQJK-CTHOSTFTDCMDAPI-UNSUBSCRIBEMARKETDATA.html.md)

<a id="author"></a>

<a id="theme_switcher"></a>
