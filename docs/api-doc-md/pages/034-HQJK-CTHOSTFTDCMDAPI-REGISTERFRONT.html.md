# RegisterFront

RegisterFront

设置交易托管系统的网络通讯地址，交易托管系统拥有多个通信地址，用户可以注册一个或多个地址。如果注册多个地址则使用最先建立TCP连接的地址。

◇ 1. 函数原型

virtual void RegisterFront(char *pszFrontAddress) = 0;

◇ 2. 参数

pszFrontAddress：指向后台服务器地址的指针。

服务器地址的格式为：“protocol://ipaddress:port”如：”tcp://127.0.0.1:17001”。“tcp”代表传输协议，“127.0.0.1”代表服务器地址。”17001”代表行情端口号。

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
//此处注册多个前置
pUserMdApi->RegisterFront(“tcp://192.168.0.1:41213”);
pUserMdApi->RegisterFront(“tcp://192.168.0.2:41213”);
pUserMdApi->Init();

```

◇ 5. FAQ

如果我注册了多个前置，会选择一个最优的前置进行连接吗？

| 会以最先建立TCP连接的地址作为当前连接地址进行连接。 |
|---|
