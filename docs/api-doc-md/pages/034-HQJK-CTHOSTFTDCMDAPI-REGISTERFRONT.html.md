# RegisterFront

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

RegisterFront<a id="content"></a>

<a id="left_menu"></a>

  ** **

设置交易托管系统的网络通讯地址，交易托管系统拥有多个通信地址，用户可以注册一个或多个地址。如果注册多个地址则使用最先建立TCP连接的地址。
<a id="07201518-c481-4f15-9b67-f4cfa6c59122"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void RegisterFront(char *pszFrontAddress) = 0;

<a id="725cbf77-ae99-4bb5-a678-0ed403f26822"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pszFrontAddress：指向后台服务器地址的指针。

服务器地址的格式为：“protocol://ipaddress:port”如：”tcp://127.0.0.1:17001”。“tcp”代表传输协议，“127.0.0.1”代表服务器地址。”17001”代表行情端口号。

SSL前置格式：ssl://192.168.0.1:41205

TCP前置IPv4格式：tcp://192.168.0.1:41205

TCP前置IPv6格式：tcp6://fe80::20f8:aa9b:7d59:887d:35001

<a id="1d8c69cf-d835-451c-90de-69eb13ca2010"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="e03743db-9a8c-4801-b8eb-0ac99d535c75"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
//此处注册多个前置
pUserMdApi->RegisterFront(“tcp://192.168.0.1:41213”);
pUserMdApi->RegisterFront(“tcp://192.168.0.2:41213”);
pUserMdApi->Init();

```

<a id="f27c6a4b-739b-4cbf-b430-6cc4ab1a2dc6"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

如果我注册了多个前置，会选择一个最优的前置进行连接吗？<a id="region_panel_1"></a>

| 会以最先建立TCP连接的地址作为当前连接地址进行连接。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
