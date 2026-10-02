# 看穿式监管数据采集说明

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

看穿式监管数据采集说明<a id="content"></a>

<a id="left_menu"></a>

  ** **

|  | ■ 6.7.13_API接口说明
└◆ 看穿式监管数据采集说明 |  |
|---|---|---|

<a id="dc062324-2039-4206-843d-d5c0c5c411b0"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1.定义
<a id="panel1"></a>

| 术语 | 术语说明 |
|---|---|
| 采集API链接库 | 负责采集终端信息的动态链接库(windows版的WinDataCollect.dll，linux版的LinuxDataCollect.so，安卓版的InfoCollection_single_release.aar)，只有连接中继服务器的终端需要调用，直连模式或中继服务器无需调用 |
| 直连类型终端 | 直接连接到CTP交易系统的客户交易终端 |
| 中继类型终端 | 先连接到中继服务器,中继服务器再调用TraderAPI连接到CTP交易系统的客户交易终端 |
| 多对多类型中继服务器 | 为每个客户终端，建立CTP API实例，每个用户独占一个交易API实例的中继服务器 |
| 一对多类型中继服务器 | 为多个客户终端，建立一个CTP API实例，使用操作员为多个客户进行交易的中继服务器 |
| AppID | 每个期货终端软件需要向期货公司申请自己的AppID |
| RelayAppID | 中继服务器软件需要向期货公司申请自己的RelayAppID |

**AppID和RelayAppID必须符合监控中心的格式要求**。见监控中心官网技术规范。

直连类型终端无需加载采集链接库，登录时会自动采集并上报终端信息。

中继类型服务器无需加载采集链接库，仅负责接收客户端的采集信息并上报CTP。

中继类型终端需要加载采集链接库，并将采集到的信息发送给中继服务器。

<a id="f79e8d17-ae4a-42d0-8d85-352614264fc1"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2.说明
<a id="panel2"></a>

期货公司确认终端软件集成了正确的数据采集模块后，为该AppID的终端软件分配授权码。终端软件需要保护好自己的AppID和授权码，防止被其他软件盗用。CTP对终端的认证流程如下：

采集信息为utf-8编码

- 鸿蒙版本的采集库需要目前需要以下系统权限：

const permissions: Array\ = ['ohos.permission.GET*NETWORK*INFO', 'ohos.permission.GET*WIFI*INFO', 'ohos.permission.LOCATION','ohos.permission.APPROXIMATELY_LOCATION', 'ohos.permission.INTERNET'];

- 直连终端认证流程

对于直接连接期货公司交易柜台的终端，期货公司确认终端软件集成了正确的数据采集模块后。给该终端软件（根据AppID）分配一个授权码。这个授权码和AppID是绑定的，当终端试图登录期货公司交易软件的时候，交易后台会验证该终端是否持有合法的AppID和授权码。

- 中继和中继下属终端的认证流程

对于使用中继服务器连接期货公司交易柜台的终端，期货公司确认终端软件集成了正确的数据采集模块和确认中继可以正常报送终端信息后。期货公司给该终端软件（根据AppID）分配一个授权码，给中继服务器（根据RelayAppID） 分配一个授权码。当终端登录中继服务器时，中继服务器负责验证终端的合法性；当中继服务器登录期货公司交易软件的时候，交易后台会验证该中继服务器是否持有合法的RelayAppID和授权码。

**注:信息采集过程中，遇到某些信息没有采集到，采集信息不全。请下载穿透式采集自检工具，工具用于检查采集信息是否完整，下载地址：https://www.simnow.com.cn/DocumentDown/api_3/5_2_8/tool250107.zip**

<a id="anchor-id-03"></a>

<a id="2648962f-a741-4ffb-8618-d64c35a38881"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3.权限要求
<a id="panel3"></a>

采集库需要u+s权限

<a id="515a7b7c-c8cd-44a5-9026-9c394c29b59f"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4.相关接口
<a id="panel4"></a>

```
///客户端认证请求
virtual int ReqAuthenticate(CThostFtdcReqAuthenticateField *pReqAuthenticateField, int nRequestID) = 0;
///客户端认证响应
virtual void OnRspAuthenticate(CThostFtdcRspAuthenticateField *pRspAuthenticateField, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
///上报用户终端信息，用于中继服务器操作员登录模式
///操作员登录后，可以多次调用该接口上报客户信息
virtual int SubmitUserSystemInfo(CThostFtdcUserSystemInfoField *pUserSystemInfo) = 0;
///注册用户终端信息，用于中继服务器多连接模式
///需要在终端认证成功后，用户登录前调用该接口
virtual int RegisterUserSystemInfo(CThostFtdcUserSystemInfoField *pUserSystemInfo) = 0;

```

注意，采集库不是线程安全的。多线程调用采集库时要加锁，如果是直连模式则要在登录函数上加锁。

<a id="dda05283-8bf3-4b31-a9ab-07023c423358"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5.示例代码
<a id="panel5"></a>

**每个类型的终端调用流程不同，请注意区分！**
<a id="db6925cd-6be4-4e4e-b93a-e793e70debda"></a><a id="title6"></a>

<a id="header_span6"></a>◇ 5.1.直连终端的采集使用流程示例代码：
<a id="panel6"></a>

Step 1 在API连接后发起认证

```
void CUser::OnFrontConnected()
{
    cout << "OnFrontConnected." << endl;
    static const char *version = m_pUserApi->GetApiVersion();
    cout << "------当前版本号 ：" << version << " ------" << endl;
    ReqAuthenticate();
}
int CUser::ReqAuthenticate()
{
    CThostFtdcReqAuthenticateField field;
    memset(&field, 0, sizeof(field));
    strcpy(field.BrokerID, "8000");
    strcpy(field.UserID, "001888");
    strcpy(field.AppID, "XY_Q7_V1.0.0");
    strcpy(field.AuthCode, "5A5P4V7AZ5LCFEAK");
    return m_pUserApi->ReqAuthenticate(&field, 5);
}

```

Step 2 认证成功后发起登录

```
void CUser::OnRspAuthenticate(CThostFtdcRspAuthenticateField *pRspAuthenticateField, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
{
    printf("OnRspAuthenticate\n");
    if (pRspInfo != NULL && pRspInfo->ErrorID == 0)
    {
        printf("认证成功,ErrorID=0x%04x, ErrMsg=%s\n\n", pRspInfo->ErrorID, pRspInfo->ErrorMsg);
        ReqUserLogin();
    }
    else
    cout << "认证失败，" << "ErrorID=" << pRspInfo->ErrorID << "  ,ErrMsg=" << pRspInfo->ErrorMsg << endl;
}
int CUser::ReqUserLogin()
{
    printf("====ReqUserLogin====,用户登录中...\n\n");
    CThostFtdcReqUserLoginField reqUserLogin;
    memset(&reqUserLogin, 0, sizeof(reqUserLogin));
    strcpy_s(reqUserLogin.BrokerID, "8000");
    strcpy(reqUserLogin.UserID, "001888");
    strcpy(reqUserLogin.Password, "1");
    strcpy(reqUserLogin.TradingDay, "20150715");
    return m_pUserApi->ReqUserLogin(&reqUserLogin, ++RequestID);
}

```

<a id="ceda4ef1-a2bb-4de7-90e4-f9d18223454c"></a><a id="title7"></a>

<a id="header_span7"></a>◇ 5.2.多对多中继终端使用流程示例代码：
<a id="panel7"></a>

Step 1 终端侧采集信息，向中继发起登录，并将终端信息发送给中继

```
char pSystemInfo[344];
int len;
CTP_GetSystemInfo(pSystemInfo, len);
cout << "CTP_GetSystemInfo once" << endl;

```

Step 2 中继收到终端的登录请求，新建一个API实例，并发起连接

```
void CUser::OnFrontConnected()
{
    cout << "OnFrontConnected." << endl;
    static const char *version = m_pUserApi->GetApiVersion();
    cout << "------当前版本号 ：" << version << " ------" << endl;
    ReqAuthenticate();
}

```

Step 3 中继连上前置后，发起认证请求

```
int CUser::ReqAuthenticate()
{
    CThostFtdcReqAuthenticateField field;
    memset(&field, 0, sizeof(field));
    strcpy(field.BrokerID, "8000");
    strcpy(field.UserID, "001888");
    strcpy(field.AppID, "XY_RELAY_V1.0.0");
    strcpy(field.AuthCode, "5A5P4V7AZ5LCFEAK");
    return m_pUserApi->ReqAuthenticate(&field, 5);
}

```

Step 4 中继认证成功后，注册终端的采集信息

```
void CUser::OnRspAuthenticate(CThostFtdcRspAuthenticateField *pRspAuthenticateField, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
{
    printf("OnRspAuthenticate\n");
    if (pRspInfo != NULL && pRspInfo->ErrorID == 0)
    {
        printf("认证成功,ErrorID=0x%04x, ErrMsg=%s\n\n", pRspInfo->ErrorID, pRspInfo->ErrorMsg);
        RegSystemInfo();
        ReqUserLogin();
    }
    else
        cout << "认证失败，" << "ErrorID=" << pRspInfo->ErrorID << "  ,ErrMsg=" << pRspInfo->ErrorMsg << endl;
}
void CUser::RegSystemInfo()
{
    //char pSystemInfo[344];
    //int len;
    ////将前面从终端得到的信息赋值给下面结构体
    CThostFtdcUserSystemInfoField field;
    memset(&field, 0, sizeof(field));
    strcpy(field.BrokerID, "8000");
    strcpy(field.UserID, "001888");
    //strcpy(field.ClientSystemInfo, pSystemInfo); 不能用 因为不是字符串
    memcpy(field.ClientSystemInfo, pSystemInfo, len);
    field.ClientSystemInfoLen = len;
    strcpy(field.ClientPublicIP, "198.4.4.124");
    field.ClientIPPort = 65535;
    strcpy(field.ClientLoginTime, "11:28:28");
    strcpy(field.ClientAppID, "Q7");
    int ret = m_pUserApi->RegisterUserSystemInfo(&field);
    cout << "retd = " << ret << endl;
}

```

Step 5 中继注册好终端的信息后，发起登录

```
int CUser::ReqUserLogin()
{
    printf("====ReqUserLogin====,用户登录中...\n\n");
    CThostFtdcReqUserLoginField reqUserLogin;
    memset(&reqUserLogin, 0, sizeof(reqUserLogin));
    strcpy_s(reqUserLogin.BrokerID, "8000");
    strcpy(reqUserLogin.UserID, "001888");
    strcpy(reqUserLogin.Password, "1");
    strcpy(reqUserLogin.TradingDay, "20150715");
    return m_pUserApi->ReqUserLogin(&reqUserLogin, ++RequestID);
}

```

<a id="4cfc85d5-6a6c-4dd3-9789-df18a47b2e63"></a><a id="title8"></a>

<a id="header_span8"></a>◇ 5.3.一对多中继终端使用流程示例代码：
<a id="panel8"></a>

Step 1 中继在启动后，在API连接后发起认证

```
void CUser::OnFrontConnected()
{
    cout << "OnFrontConnected." << endl;
    static const char *version = m_pUserApi->GetApiVersion();
    cout << "------当前版本号 ：" << version << " ------" << endl;
    ReqAuthenticate();
}
int CUser::ReqAuthenticate()
{
    CThostFtdcReqAuthenticateField field;
    memset(&field, 0, sizeof(field));
    strcpy(field.BrokerID, "8000");
    strcpy(field.UserID, "8000_admin");
    strcpy(field.AppID, " XY_RELAYB_V1.0.0");
    strcpy(field.AuthCode, "BLFLTYGD16FZZ91T");
    return m_pUserApi->ReqAuthenticate(&field, 5);
}

```

Step 2 中继认证成功后，使用操作员发起登录

```
void CUser::OnRspAuthenticate(CThostFtdcRspAuthenticateField *pRspAuthenticateField, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
{
    printf("OnRspAuthenticate\n");
    if (pRspInfo != NULL && pRspInfo->ErrorID == 0)
   {
        printf("认证成功,ErrorID=0x%04x, ErrMsg=%s\n\n", pRspInfo->ErrorID, pRspInfo->ErrorMsg);
        ReqUserLogin();
    }
    else
        cout << "认证失败，" << "ErrorID=" << pRspInfo->ErrorID << "  ,ErrMsg=" << pRspInfo->ErrorMsg << endl;
}
int CUser::ReqUserLogin()
{
    printf("====ReqUserLogin====,用户登录中...\n\n");
    CThostFtdcReqUserLoginField reqUserLogin;
    memset(&reqUserLogin, 0, sizeof(reqUserLogin));
    strcpy_s(reqUserLogin.BrokerID, "8000");
    strcpy(reqUserLogin.UserID, "8000_admin");
    strcpy(reqUserLogin.Password, "1");
    strcpy(reqUserLogin.TradingDay, "");
    strcpy(reqUserLogin.ClientIPAddress, "1.1.1.1");
    reqUserLogin.ClientIPPort = 11;
    return m_pUserApi->ReqUserLogin(&reqUserLogin, ++RequestID);
}

```

Step 3 此时有终端登录中继，并将采集信息发送给中继

```
char pSystemInfo[344];
int len;
CTP_GetSystemInfo(pSystemInfo, len);
cout << "CTP_GetSystemInfo once" << endl;

```

Step 4 中继接收到采集信息后，发起终端信息的上报

```
void CUser::SubSystemInfo()
{
    //char pSystemInfo[344];
    //int len;
    ///CTP_GetSystemInfo(pSystemInfo, len);
    /////终端信息由终端发送到中继，并提交信息
    cout << "SubSystemInfo 1" << endl;
    CThostFtdcUserSystemInfoField field;
    memset(&field, 0, sizeof(field));
    strcpy(field.BrokerID, "8000");
    strcpy(field.UserID, "0018881");
    //strcpy(field.ClientSystemInfo, pSystemInfo); 不能用 因为不是字符串
    memcpy(field.ClientSystemInfo, pSystemInfo, len);
    field.ClientSystemInfoLen = len;
    strcpy(field.ClientPublicIP, "198.114.114.124");
    field.ClientIPPort = 65535;
    strcpy(field.ClientLoginTime, "11:28:28");
    strcpy(field.ClientAppID, "Q7");
    int retx = m_pUserApi->SubmitUserSystemInfo(&field);
    cout << "ret = " << retx << endl;
     //为另一个用户提交采集信息
    CThostFtdcUserSystemInfoField field1;
    memset(&field1, 0, sizeof(field1));
    strcpy(field1.BrokerID, "8000");
    strcpy(field1.UserID, "0018882");
    memcpy(field1.ClientSystemInfo, pSystemInfo, len);
    field1.ClientSystemInfoLen = len;
    strcpy(field1.ClientPublicIP, "198.4.4.120");
    field1.ClientIPPort = 65532;
    strcpy(field1.ClientLoginTime, "11:28:29");
    strcpy(field1.ClientAppID, "Q7");
    m_pUserApi->SubmitUserSystemInfo(&field1);
}

```

请参阅：

    [常见FAQ](pages/023-CTSJGSJCJJK-CJFAQ.html.md)

    [CTP-GetSystemInfo](pages/024-CTSJGSJCJJK-CTP-GETSYSTEMINFO.html.md)

    [CTP-GetDataCollectApiVersion](pages/025-CTSJGSJCJJK-CTP-GETDATACOLLECTAPIVERSION.html.md)

<a id="author"></a>

<a id="theme_switcher"></a>
