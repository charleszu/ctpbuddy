# RegisterFront

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

RegisterFront<a id="content"></a>

<a id="left_menu"></a>

  ** **

设置交易托管系统的网络通讯地址，交易托管系统拥有多个通信地址，但用户只需要选择一个通信地址。

也可以通过多次调用的方式注册不同的前置地址实现冗余。当交易系统断开连接时会自动从注册的地址池中择优选择一个建立连接。择优方式为以最先建立TCP连接的地址为连接地址。
<a id="debe61af-9021-4cc3-9d9a-b18dfd499142"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [RegisterFront](pages/034-HQJK-CTHOSTFTDCMDAPI-REGISTERFRONT.html.md)(char *pszFrontAddress) = 0;

<a id="0617c576-e325-4023-821b-7ae78454ae9b"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pszFrontAddress：指向后台服务器地址的指针。

服务器地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:17001”。“tcp”代表传输协议，“127.0.0.1”代表服务器地址。”17001”代表交易端口号。

SSL前置格式：ssl://192.168.0.1:41205

TCP前置IPv4格式：tcp://192.168.0.1:41205

TCP前置IPv6格式：tcp6://fe80::20f8:aa9b:7d59:887d:35001

<a id="496f6e66-5fa5-4ee5-b960-395bc4110cd7"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="863ebd27-e735-47e6-89ff-d539d9a3910f"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

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

<a id="64e10139-2778-40be-b947-657e4194a99f"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

“CTP:无此功能”是什么错？<a id="region_panel_1"></a>

| 端口如果填写错误，例如交易和行情写反，则会报。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
