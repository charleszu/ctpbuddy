# GetApiVersion

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

GetApiVersion<a id="content"></a>

<a id="left_menu"></a>

  ** **

获取API的版本信息
<a id="c01673d0-1af4-49df-adc2-2a134f5970ea"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual const char * GetApiVersion () = 0;

<a id="cfaa175d-5f37-4412-ab3c-57e8c9cc74b6"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 返回
<a id="panel2"></a>

const char * 返回一个指向版本字符串的指针

<a id="8c6c3046-83e9-481a-8ebb-fec41d55a7ad"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 调用示例
<a id="panel3"></a>

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
printf("版本号为:%s\n", pUserMdApi->GetApiVersion());
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
pUserMdApi->RegisterFront(“tcp://127.0.0.1:41205”);
pUserMdApi->Init();

```

<a id="193c065f-a702-4229-81e8-944c39303c51"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
