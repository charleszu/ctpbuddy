# CreateFtdcMdApi

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

CreateFtdcMdApi<a id="content"></a>

<a id="left_menu"></a>

  ** **

创建[CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md)实例。

接口由 “CreateFtdcMdApi(const char *pszFlowPath = "", const bool blsUsingUdp=false, const bool blsMulticast=false);”改为“CreateFtdcMdApi(const char *pszFlowPath = "", const bool blsUsingUdp=false, const bool blsMulticast=false, bool blsProductionMode=true);”  。
<a id="016337e1-e896-4300-91f8-fd75d768b498"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

static [CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md) *CreateFtdcMdApi(const char *pszFlowPath = "", const bool bIsUsingUdp=false, const bool bIsMulticast=false, bool bIsProductionMode=true);

<a id="f78c813b-41c8-41c6-ad81-c461c18ace41"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pszFlowPath：常量字符指针，用于指定一个文件目录来存贮交易托管系统发布消息的状态。默认值代表当前目录。

bIsUsingUdp：是否使用UDP行情

bIsMulticast：是否使用组播行情

组播行情只能在内网中使用，需要咨询所连接的系统是否支持组播行情。

bIsProductionMode:选在连接的是生产还是评测前置，true:使用生产版本的API  false:使用测评版本API

各类型行情字段组合如下：

|  | bIsUsingUdp | bIsMulticast |
|---|---|---|
| TCP行情前置 | false | false |
| UDP行情前置 | true | false |
| 组播行情前置 | true | true |

连接mdfront时，在ini中有ThostChannelModel、ThostUsingMulticast两项用于配置对客户端的连接模式，分三种情况：

|  | ThostChannelModel | ThostUsingMulticast | 终端配置的模式 |
|---|---|---|---|
| TCP模式 | tcp |  | TCP |
| UDP模式 | udp | no | TCP、UDP |
| 组播模式 | udp | yes | 组播 |

<a id="64ba32c9-5004-4e5b-a8c2-1141d290bed2"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

返回一个[CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md)实例。

<a id="e381532d-54d7-4454-b714-b0a26cea3803"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
pUserMdApi->RegisterFront(“tcp://127.0.0.1:41205”);
pUserMdApi->Init();

```

<a id="96c3ad8f-592b-49a3-9c20-d7713653e381"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

“RuntimeError:can not open CFlow file in line 279 of file ....\source\userapi\ThostFtdcUserApiImplBase.cpp”程序一运行就报这个错是为什么？<a id="region_panel_1"></a>

| 程序运行之前，flow目录必须提前创建好，否则会报错。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
