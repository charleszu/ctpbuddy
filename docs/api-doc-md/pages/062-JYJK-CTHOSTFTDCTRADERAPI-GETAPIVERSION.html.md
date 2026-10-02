# GetApiVersion

GetApiVersion

获取API的版本信息

◇ 1. 函数原型

virtual const char *[GetApiVersion](../../HQJK/CTHOSTFTDCMDAPI/GETAPIVERSION.html)() = 0;

◇ 2. 参数

无

◇ 3. 返回

返回具体的版本号，如（v6.3.11_20180109 14:59:39）

◇ 4. 调用示例

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
LOG(pUserApi->GetApiVersion());
pUserApi->SubscribePrivateTopic(THOST_TERT_QUICK);
pUserApi->SubscribePublicTopic(THOST_TERT_QUICK);
pUserApi->RegisterFront(“tcp://127.0.0.1:51205”);
pUserApi->Init();

```

◇ 5. FAQ

无
