# GetTradingDay

GetTradingDay

获得当前交易日。只有当登陆成功后才会取到正确的值。

◇ 1. 函数原型

virtual const char *[GetTradingDay](../../HQJK/CTHOSTFTDCMDAPI/GETTRADINGDAY.html)() = 0;

◇ 2. 参数

无

◇ 3. 返回

返回一个指向日期信息字符串的常量指针。

◇ 4. 调用示例

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

◇ 5. FAQ

无
