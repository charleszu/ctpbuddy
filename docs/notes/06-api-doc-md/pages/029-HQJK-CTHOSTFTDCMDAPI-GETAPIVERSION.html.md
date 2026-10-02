# GetApiVersion

GetApiVersion

获取API的版本信息

◇ 1. 函数原型

virtual const char * GetApiVersion () = 0;

◇ 2. 返回

const char * 返回一个指向版本字符串的指针

◇ 3. 调用示例

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
printf("版本号为:%s\n", pUserMdApi->GetApiVersion());
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
pUserMdApi->RegisterFront(“tcp://127.0.0.1:41205”);
pUserMdApi->Init();

```

◇ 4. FAQ

无
