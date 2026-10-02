# SubscribePublicTopic

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

SubscribePublicTopic<a id="content"></a>

<a id="left_menu"></a>

  ** **

订阅公共流。该方法要在[Init](pages/031-HQJK-CTHOSTFTDCMDAPI-INIT.html.md) 方法前调用。若不调用，默认RESTART模式订阅。
<a id="99ff2e4f-a2ab-49ec-b22e-0b5c4de39dd6"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void SubscribePublicTopic(THOST_TE_RESUME_TYPE nResumeType) = 0;

<a id="0b92073d-1547-412a-8709-536bc0b39c85"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

nResumeType：私有流重传方式。

THOST_TERT_RESTART：从本交易日开始重传

THOST_TERT_RESUME：从上次收到的续传

THOST_TERT_QUICK：只传送登录后私有流的内容

THOST_TERT_NONE：取消订阅公有流

<a id="73cd56f6-8bbd-48ad-95cb-fb9da1580c07"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="4c5dde0d-ed6e-482e-ac90-f32ce28f2620"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
pUserApi->SubscribePrivateTopic(THOST_TERT_RESUME);
pUserApi->SubscribePublicTopic(THOST_TERT_RESUME);
pUserApi->RegisterFront("tcp://127.0.0.1:51205");
pUserApi->Init();

```

<a id="4a83bb44-5113-4d09-b24c-e0e121b0693b"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
