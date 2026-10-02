# CreateFtdcMdApi

CreateFtdcMdApi

创建[CThostFtdcMdApi](_CTHOSTFTDCMDAPI.html)实例。

接口由 “CreateFtdcMdApi(const char *pszFlowPath = "", const bool blsUsingUdp=false, const bool blsMulticast=false);”改为“CreateFtdcMdApi(const char *pszFlowPath = "", const bool blsUsingUdp=false, const bool blsMulticast=false, bool blsProductionMode=true);”  。

◇ 1. 函数原型

static [CThostFtdcMdApi](_CTHOSTFTDCMDAPI.html) *CreateFtdcMdApi(const char *pszFlowPath = "", const bool bIsUsingUdp=false, const bool bIsMulticast=false, bool bIsProductionMode=true);

◇ 2. 参数

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

◇ 3. 返回

返回一个[CThostFtdcMdApi](_CTHOSTFTDCMDAPI.html)实例。

◇ 4. 调用示例

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
pUserMdApi->RegisterFront(“tcp://127.0.0.1:41205”);
pUserMdApi->Init();

```

◇ 5. FAQ

“RuntimeError:can not open CFlow file in line 279 of file ....\source\userapi\ThostFtdcUserApiImplBase.cpp”程序一运行就报这个错是为什么？

| 程序运行之前，flow目录必须提前创建好，否则会报错。 |
|---|
