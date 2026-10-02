# Init

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

Init<a id="content"></a>

<a id="left_menu"></a>

  ** **

初始化运行环境,只有调用后,接口才开始发起前置的连接请求。
<a id="99524225-9136-4611-be55-61ec0a81155d"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void Init() = 0;

<a id="d14fb4f0-be44-4ee6-91f4-61016cad79e8"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

无

<a id="2b759fb1-ef58-4213-bf92-f08954f66787"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="92598053-e826-4f0c-98e1-5329cb885515"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
pUserMdApi->RegisterFront(“tcp://127.0.0.1:41205”);
pUserMdApi->Init();

```

<a id="599825c1-13c0-46c3-aa13-97fc4aad06e4"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
