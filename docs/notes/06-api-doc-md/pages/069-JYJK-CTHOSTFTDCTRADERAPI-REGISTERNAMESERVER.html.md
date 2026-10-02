# RegisterNameServer

RegisterNameServer

设置名字服务器网络地址。RegisterNameServer优先于[RegisterFront](../../HQJK/CTHOSTFTDCMDAPI/REGISTERFRONT.html)。

调用前需要先使用[RegisterFensUserInfo](../../HQJK/CTHOSTFTDCMDAPI/REGISTERFENSUSERINFO.html)设置登录模式。

详见[fens连接说明](../../QTYWGZ/FENS.html)

◇ 1. 函数原型

virtual void RegisterNameServer(char *pszNsAddress) = 0;

◇ 2. 参数

pszNsAddress:指向后台服务器地址的指针。

服务器地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:17001”。“tcp”代表传输协议，“127.0.0.1”代表服务器地址。”17001”代表服务器端口号。

SSL前置格式：ssl://192.168.0.1:41205

TCP前置IPv4格式：tcp://192.168.0.1:41205

TCP前置IPv6格式：tcp6://fe80::20f8:aa9b:7d59:887d:35001

◇ 3. 返回

无

◇ 4. 调用示例

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

◇ 5. FAQ

无
