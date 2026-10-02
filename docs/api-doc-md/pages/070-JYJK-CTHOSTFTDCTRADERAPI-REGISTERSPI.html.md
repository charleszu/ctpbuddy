# RegisterSpi

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

RegisterSpi<a id="content"></a>

<a id="left_menu"></a>

  ** **

注册一个派生自[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md) 接口类的实例，该实例将完成事件处理。
<a id="50afc748-9091-4941-a585-c3c10f79a5d6"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [RegisterSpi](pages/036-HQJK-CTHOSTFTDCMDAPI-REGISTERSPI.html.md)([CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md) *pSpi) = 0;

<a id="c60fcb20-4751-444a-b362-4341c470ddab"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pSpi：实现了[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md)接口的实例指针。

<a id="690eb772-7d02-4234-b694-9eac0926bd62"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="44d2fa02-3474-41b2-9f00-b9c57663c25b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\api_liu\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
pUserApi->SubscribePrivateTopic(THOST_TERT_QUICK);
pUserApi->SubscribePublicTopic(THOST_TERT_QUICK);
pUserApi->RegisterFront(“tcp://127.0.0.1:51205”);
pUserApi->Init();

```

<a id="274ac600-3ecc-4871-a9e9-b4020fee340a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
