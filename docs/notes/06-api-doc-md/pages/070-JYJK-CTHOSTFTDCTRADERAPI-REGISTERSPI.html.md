# RegisterSpi

RegisterSpi

注册一个派生自[CThostFtdcTraderSpi](../CTHOSTFTDCTRADERSPI/_CTHOSTFTDCTRADERSPI.html) 接口类的实例，该实例将完成事件处理。

◇ 1. 函数原型

virtual void [RegisterSpi](../../HQJK/CTHOSTFTDCMDAPI/REGISTERSPI.html)([CThostFtdcTraderSpi](../CTHOSTFTDCTRADERSPI/_CTHOSTFTDCTRADERSPI.html) *pSpi) = 0;

◇ 2. 参数

pSpi：实现了[CThostFtdcTraderSpi](../CTHOSTFTDCTRADERSPI/_CTHOSTFTDCTRADERSPI.html)接口的实例指针。

◇ 3. 返回

无

◇ 4. 调用示例

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\api_liu\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
pUserApi->SubscribePrivateTopic(THOST_TERT_QUICK);
pUserApi->SubscribePublicTopic(THOST_TERT_QUICK);
pUserApi->RegisterFront(“tcp://127.0.0.1:51205”);
pUserApi->Init();

```

◇ 5. FAQ

无
