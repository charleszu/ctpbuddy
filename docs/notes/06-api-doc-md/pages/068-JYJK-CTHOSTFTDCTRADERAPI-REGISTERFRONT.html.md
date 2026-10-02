# RegisterFront

RegisterFront

设置交易托管系统的网络通讯地址，交易托管系统拥有多个通信地址，但用户只需要选择一个通信地址。

也可以通过多次调用的方式注册不同的前置地址实现冗余。当交易系统断开连接时会自动从注册的地址池中择优选择一个建立连接。择优方式为以最先建立TCP连接的地址为连接地址。

◇ 1. 函数原型

virtual void [RegisterFront](../../HQJK/CTHOSTFTDCMDAPI/REGISTERFRONT.html)(char *pszFrontAddress) = 0;

◇ 2. 参数

pszFrontAddress：指向后台服务器地址的指针。

服务器地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:17001”。“tcp”代表传输协议，“127.0.0.1”代表服务器地址。”17001”代表交易端口号。

SSL前置格式：ssl://192.168.0.1:41205

TCP前置IPv4格式：tcp://192.168.0.1:41205

TCP前置IPv6格式：tcp6://fe80::20f8:aa9b:7d59:887d:35001

◇ 3. 返回

无

◇ 4. 调用示例

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow \\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
pUserApi->SubscribePrivateTopic(THOST_TERT_RESUME);
pUserApi->SubscribePublicTopic(THOST_TERT_RESUME);
//注册多个前置地址
pUserApi->RegisterFront(“tcp://192.168.1.10:51205”);
pUserApi->RegisterFront(“tcp://192.168.1.11:51205”);
pUserApi->RegisterFront(“tcp://192.168.1.12:51205”);
pUserApi->Init();

```

◇ 5. FAQ

“CTP:无此功能”是什么错？

| 端口如果填写错误，例如交易和行情写反，则会报。 |
|---|
