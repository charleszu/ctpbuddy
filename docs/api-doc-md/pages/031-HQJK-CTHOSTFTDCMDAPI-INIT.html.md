# Init

Init

初始化运行环境,只有调用后,接口才开始发起前置的连接请求。

◇ 1. 函数原型

virtual void Init() = 0;

◇ 2. 参数

无

◇ 3. 返回

无

◇ 4. 调用示例

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
pUserMdApi->RegisterFront(“tcp://127.0.0.1:41205”);
pUserMdApi->Init();

```

◇ 5. FAQ

无
