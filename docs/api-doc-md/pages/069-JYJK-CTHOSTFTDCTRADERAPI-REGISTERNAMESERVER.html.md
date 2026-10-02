# RegisterNameServer

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

RegisterNameServer<a id="content"></a>

<a id="left_menu"></a>

  ** **

设置名字服务器网络地址。RegisterNameServer优先于[RegisterFront](pages/034-HQJK-CTHOSTFTDCMDAPI-REGISTERFRONT.html.md)。

调用前需要先使用[RegisterFensUserInfo](pages/033-HQJK-CTHOSTFTDCMDAPI-REGISTERFENSUSERINFO.html.md)设置登录模式。

详见[fens连接说明](pages/393-QTYWGZ-FENS.html.md)
<a id="cec8a551-c0f0-4e21-a1f0-391afbda9fad"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void RegisterNameServer(char *pszNsAddress) = 0;

<a id="f1e6747d-8419-414e-b335-8b1cfc135b77"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pszNsAddress:指向后台服务器地址的指针。

服务器地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:17001”。“tcp”代表传输协议，“127.0.0.1”代表服务器地址。”17001”代表服务器端口号。

SSL前置格式：ssl://192.168.0.1:41205

TCP前置IPv4格式：tcp://192.168.0.1:41205

TCP前置IPv6格式：tcp6://fe80::20f8:aa9b:7d59:887d:35001

<a id="151b00f3-625f-42cd-853f-9ffafad2d9e8"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="57a3165e-de06-4295-9cc9-8fc298fea70e"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
printf(pUserApi->GetApiVersion());
pUserApi->SubscribePrivateTopic(THOST_TERT_QUICK);
pUserApi->SubscribePublicTopic(THOST_TERT_QUICK);
CThostFtdcFensUserInfoField pFensUserInfo = { 0 };
strcpy_s(pFensUserInfo.BrokerID, "9999");
strcpy_s(pFensUserInfo.UserID, "1000001");
pFensUserInfo.LoginMode = THOST_FTDC_LM_Trade;
pUserApi->RegisterFensUserInfo(&pFensUserInfo);
pUserApi-> RegisterNameServer (“tcp://127.0.0.1:41205”);
pUserApi->Init();

```

<a id="5c5b4375-e9cf-486c-ba2c-912a28ba596b"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
