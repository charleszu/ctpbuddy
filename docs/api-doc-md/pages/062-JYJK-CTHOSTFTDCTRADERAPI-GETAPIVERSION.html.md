# GetApiVersion

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

GetApiVersion<a id="content"></a>

<a id="left_menu"></a>

  ** **

获取API的版本信息
<a id="c35fe5c8-9602-44a0-a9de-4fc555081ea4"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual const char *[GetApiVersion](pages/029-HQJK-CTHOSTFTDCMDAPI-GETAPIVERSION.html.md)() = 0;

<a id="c168728e-0236-4a6e-852d-761d526c720a"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

无

<a id="8ec4be3e-5216-42fb-a126-4bdd424ce179"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

返回具体的版本号，如（v6.3.11_20180109 14:59:39）

<a id="0b62f15b-a5d3-4c8a-9745-7a12c949ffee"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
LOG(pUserApi->GetApiVersion());
pUserApi->SubscribePrivateTopic(THOST_TERT_QUICK);
pUserApi->SubscribePublicTopic(THOST_TERT_QUICK);
pUserApi->RegisterFront(“tcp://127.0.0.1:51205”);
pUserApi->Init();

```

<a id="f32e16f5-0b81-49d3-bb0f-5e8dc31d475f"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
