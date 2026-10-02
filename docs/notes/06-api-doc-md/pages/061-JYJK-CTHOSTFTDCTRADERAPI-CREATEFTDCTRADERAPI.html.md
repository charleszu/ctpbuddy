# CreateFtdcTraderApi

CreateFtdcTraderApi

创建TraderApi实例。如果创建多个api实例，则每个实例的flow目录都要区分开，否则可能会导致报单回报丢失。

接口由  “CreateFtdcTraderApi(const char *pszflowPath="");  ”改为 “CreateFtdcTraderApi(const char*pszflowPath ="",bool blsProductionMode=true);”。

◇ 1. 函数原型

static [CThostFtdcTraderApi](_CTHOSTFTDCTRADERAPI.html) *CreateFtdcTraderApi(const char *pszFlowPath = "", bool bIsProductionMode = true);

◇ 2. 参数

pszFlowPath：常量字符指针，用于指定一个文件目录来存贮交易托管系统发布消息的状态。默认值代表当前目录。

bIsProductionMode：定义连接的是生产还是评测前置，true:使用生产版本的API  false:使用测评版本的API

◇ 3. 返回

无

◇ 4. 调用示例

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

◇ 5. FAQ

“RuntimeError:can not open CFlow file in line 279 of file ....\source\userapi\ThostFtdcUserApiImplBase.cpp” 报错是什么意思？

| 程序运行之前，flow目录必须提前创建好，否则会报错。 |
|---|

“RuntimeError:can not open CFlow file in line 338 of file ....\source\userapi\ThostFtdcUserApiImplBase.cpp” 报错是什么意思？

| 有core文件生成，调试后发现断点在CThostUserFlow::OpenFile中，可能是ulimit参数“open files”太小导致不能开启更多线程。 |
|---|
