# RegisterNameServer

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

RegisterNameServer<a id="content"></a>

<a id="left_menu"></a>

  ** **

设置名字服务器网络地址。[RegisterNameServer](pages/069-JYJK-CTHOSTFTDCTRADERAPI-REGISTERNAMESERVER.html.md)优先于[RegisterFront](pages/034-HQJK-CTHOSTFTDCMDAPI-REGISTERFRONT.html.md)。

调用前需要先使用[RegisterFensUserInfo](pages/033-HQJK-CTHOSTFTDCMDAPI-REGISTERFENSUSERINFO.html.md)设置登录模式。

如果CTP系统启用了fens前置，则可以使用该接口连接fens前置地址。

fens的好处是fens地址对应的后端地址是一个前置地址池，前置地址的增删改都对用户透明，用户不需要调整自己的接入地址。当API使用fens地址接入时，fens前置会返回一个地址池，随后API择优选择一个地址进行接入。

详见[fens连接说明](pages/393-QTYWGZ-FENS.html.md)
<a id="21ece22c-5e27-4f5e-9879-5ec271f01bce"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [RegisterNameServer](pages/069-JYJK-CTHOSTFTDCTRADERAPI-REGISTERNAMESERVER.html.md)(char *pszNsAddress) = 0;

<a id="2c311869-4038-4e4b-899d-a0225206880c"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pszNsAddress:指向后台服务器地址的指针。

服务器地址的格式为：“protocol://ipaddress:port”。如：“tcp://127.0.0.1:17001”。“tcp”代表传输协议，“127.0.0.1”代表服务器地址。“17001”代表服务器端口号。

SSL前置格式：ssl://192.168.0.1:41205

TCP前置IPv4格式：tcp://192.168.0.1:41205

TCP前置IPv6格式：tcp6://fe80::20f8:aa9b:7d59:887d:35001

<a id="3c604dd0-cd7f-40fd-88a1-9416b2b9e54b"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="47089e12-765c-43d4-8151-1a8f1361ebdb"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
CThostFtdcFensUserInfoField pFensUserInfo = { 0 };
strcpy_s(pFensUserInfo.BrokerID, g_chBrokerID);
strcpy_s(pFensUserInfo.UserID, g_chUserID);
pFensUserInfo.LoginMode = THOST_FTDC_LM_Trade;
pUserMdApi->RegisterFensUserInfo(&pFensUserInfo, nRequestID++);
pUserMdApi->RegisterNameServer (“tcp://127.0.0.1:41205”);
pUserMdApi->Init();

```

<a id="96fdb7de-f747-4899-ac9b-cbd99f847ca2"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
