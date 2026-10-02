# RegisterNameServer

RegisterNameServer

设置名字服务器网络地址。[RegisterNameServer](../../JYJK/CTHOSTFTDCTRADERAPI/REGISTERNAMESERVER.html)优先于[RegisterFront](REGISTERFRONT.html)。

调用前需要先使用[RegisterFensUserInfo](REGISTERFENSUSERINFO.html)设置登录模式。

如果CTP系统启用了fens前置，则可以使用该接口连接fens前置地址。

fens的好处是fens地址对应的后端地址是一个前置地址池，前置地址的增删改都对用户透明，用户不需要调整自己的接入地址。当API使用fens地址接入时，fens前置会返回一个地址池，随后API择优选择一个地址进行接入。

详见[fens连接说明](../../QTYWGZ/FENS.html)

◇ 1. 函数原型

virtual void [RegisterNameServer](../../JYJK/CTHOSTFTDCTRADERAPI/REGISTERNAMESERVER.html)(char *pszNsAddress) = 0;

◇ 2. 参数

pszNsAddress:指向后台服务器地址的指针。

服务器地址的格式为：“protocol://ipaddress:port”。如：“tcp://127.0.0.1:17001”。“tcp”代表传输协议，“127.0.0.1”代表服务器地址。“17001”代表服务器端口号。

SSL前置格式：ssl://192.168.0.1:41205

TCP前置IPv4格式：tcp://192.168.0.1:41205

TCP前置IPv6格式：tcp6://fe80::20f8:aa9b:7d59:887d:35001

◇ 3. 返回

无

◇ 4. 调用示例

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

◇ 5. FAQ

无
