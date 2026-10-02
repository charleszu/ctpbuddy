# CreateFtdcTraderApi

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

CreateFtdcTraderApi<a id="content"></a>

<a id="left_menu"></a>

  ** **

创建TraderApi实例。如果创建多个api实例，则每个实例的flow目录都要区分开，否则可能会导致报单回报丢失。

接口由  “CreateFtdcTraderApi(const char *pszflowPath="");  ”改为 “CreateFtdcTraderApi(const char*pszflowPath ="",bool blsProductionMode=true);”。
<a id="7cbc9ce9-7321-4c8a-9e96-6eab5fcccc6e"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

static [CThostFtdcTraderApi](pages/060-JYJK-CTHOSTFTDCTRADERAPI-_CTHOSTFTDCTRADERAPI.html.md) *CreateFtdcTraderApi(const char *pszFlowPath = "", bool bIsProductionMode = true);

<a id="742b9c93-eac5-4dbc-8447-0db4d35473f0"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pszFlowPath：常量字符指针，用于指定一个文件目录来存贮交易托管系统发布消息的状态。默认值代表当前目录。

bIsProductionMode：定义连接的是生产还是评测前置，true:使用生产版本的API  false:使用测评版本的API

<a id="afe97566-365e-4d00-977e-2fb74536d197"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="de8e451a-1047-4e51-969d-2dcc2e8f5f81"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
//初始化api
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("flow\\01\\",true);//连接生产前置
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
pUserApi->SubscribePrivateTopic(THOST_TERT_QUICK);
pUserApi->SubscribePublicTopic(THOST_TERT_QUICK);
pUserApi->RegisterFront("tcp://127.0.0.1:41205");
pUserApi->Init();
//创建第二个api实例，要区分开flow目录
CThostFtdcTraderApi *pUserApi2 = CThostFtdcTraderApi::CreateFtdcTraderApi("flow\\02\\",true);

```

<a id="64d64a98-d0bb-48e5-8d5d-a0e1af22a755"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

“RuntimeError:can not open CFlow file in line 279 of file ....\source\userapi\ThostFtdcUserApiImplBase.cpp” 报错是什么意思？<a id="region_panel_1"></a>

| 程序运行之前，flow目录必须提前创建好，否则会报错。 |
|---|

<a id="region_tail_1"></a>

<a id="anchor-id-01"></a>

<a id="region_header_2"></a>

“RuntimeError:can not open CFlow file in line 338 of file ....\source\userapi\ThostFtdcUserApiImplBase.cpp” 报错是什么意思？<a id="region_panel_2"></a>

| 有core文件生成，调试后发现断点在CThostUserFlow::OpenFile中，可能是ulimit参数“open files”太小导致不能开启更多线程。 |
|---|

<a id="region_tail_2"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
