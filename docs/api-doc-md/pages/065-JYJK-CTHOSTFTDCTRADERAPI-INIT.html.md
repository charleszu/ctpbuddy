# Init

Init

使客户端开始与交易托管系统建立连接，连接成功后可以进行登陆。

非线程安全，多线程使用请加锁。

◇ 1. 函数原型

virtual void [Init](../../HQJK/CTHOSTFTDCMDAPI/INIT.html)() = 0;

◇ 2. 参数

无

◇ 3. 返回

无

◇ 4. 调用示例

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
pUserApi->SubscribePrivateTopic(THOST_TERT_QUICK);
pUserApi->SubscribePublicTopic(THOST_TERT_QUICK);
pUserApi->RegisterFront(“tcp://127.0.0.1:51205”);
pUserApi->Init();

```

◇ 5. FAQ

无
