# Init

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

Init<a id="content"></a>

<a id="left_menu"></a>

  ** **

使客户端开始与交易托管系统建立连接，连接成功后可以进行登陆。

非线程安全，多线程使用请加锁。
<a id="47c93027-3a8d-4fec-9f34-24a67156d8a2"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [Init](pages/031-HQJK-CTHOSTFTDCMDAPI-INIT.html.md)() = 0;

<a id="823209a5-f395-4ba0-8531-03b37cf7e4e5"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

无

<a id="b21c1531-71c2-4ffc-9da2-6b67f0e70045"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="4ae3090b-66a2-4dbd-be28-192c26017185"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
pUserApi->SubscribePrivateTopic(THOST_TERT_QUICK);
pUserApi->SubscribePublicTopic(THOST_TERT_QUICK);
pUserApi->RegisterFront(“tcp://127.0.0.1:51205”);
pUserApi->Init();

```

<a id="0521a994-596b-4999-b523-b2b6e23624e1"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
