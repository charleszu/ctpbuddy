# GetTradingDay

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

GetTradingDay<a id="content"></a>

<a id="left_menu"></a>

  ** **

获得当前交易日。只有当登陆成功后才会取到正确的值。
<a id="61890410-b9ee-4f6b-85bd-76d06b71d4a0"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual const char *[GetTradingDay](pages/030-HQJK-CTHOSTFTDCMDAPI-GETTRADINGDAY.html.md)() = 0;

<a id="61c74e69-5d53-48cf-86ef-a940ffaa306c"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

无

<a id="a886ce6b-08d5-4db2-9b6f-733bc7218142"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

返回一个指向日期信息字符串的常量指针。

<a id="879c4a4d-11ea-4fd3-aedc-8559a9bfda76"></a><a id="title4"></a>

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
WaitForSingleObject(g_LoginSig, INFINITE); //等待登陆成功后
printf(pUserApi->GetTradingDay()); //获取交易日

```

<a id="ae9e33c6-33c5-467d-9f6e-3d430f46cb03"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
