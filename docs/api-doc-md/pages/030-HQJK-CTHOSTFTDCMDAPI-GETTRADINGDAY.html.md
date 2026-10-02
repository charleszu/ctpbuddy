# GetTradingDay

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

GetTradingDay<a id="content"></a>

<a id="left_menu"></a>

  ** **

获得当前交易日。只有当登陆成功后才会取到正确的值。
<a id="8bf9f7f7-7d1a-4993-b04b-b3755f9a096a"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual const char *GetTradingDay() = 0;

<a id="6092c9da-6da4-4e99-98d6-ea6cc7348b9e"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 返回
<a id="panel2"></a>

返回一个指向日期信息字符串的常量指针。

<a id="8c31b2eb-f0f4-4f44-91ba-7ca15bfb49bb"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 调用示例
<a id="panel3"></a>

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
pUserMdApi->RegisterFront(“tcp://127.0.0.1:41213”);
pUserMdApi->Init();
//登录成功后
printf("获取当前交易日期:%s\n", pUserMdApi->GetTradingDay());

```

<a id="4f4d46ec-00dc-45b8-900e-1ad615b3f64a"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

<a id="region_header_1"></a>

登录前为什么也可以获取到交易日？<a id="region_panel_1"></a>

| 登录前调用该函数会去上一次的API流水文件里去获取交易日。只有在登录后才能获取到正确的交易日。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
