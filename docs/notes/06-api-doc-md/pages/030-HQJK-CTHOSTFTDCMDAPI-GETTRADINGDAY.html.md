# GetTradingDay

GetTradingDay

获得当前交易日。只有当登陆成功后才会取到正确的值。

◇ 1. 函数原型

virtual const char *GetTradingDay() = 0;

◇ 2. 返回

返回一个指向日期信息字符串的常量指针。

◇ 3. 调用示例

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
pUserMdApi->RegisterFront(“tcp://127.0.0.1:41213”);
pUserMdApi->Init();
//登录成功后
printf("获取当前交易日期:%s\n", pUserMdApi->GetTradingDay());

```

◇ 4. FAQ

登录前为什么也可以获取到交易日？

| 登录前调用该函数会去上一次的API流水文件里去获取交易日。只有在登录后才能获取到正确的交易日。 |
|---|
