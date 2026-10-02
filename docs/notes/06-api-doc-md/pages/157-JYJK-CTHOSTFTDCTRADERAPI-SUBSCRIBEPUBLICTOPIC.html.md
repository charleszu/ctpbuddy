# SubscribePublicTopic

SubscribePublicTopic

订阅公共流。该方法要在[Init](../../HQJK/CTHOSTFTDCMDAPI/INIT.html) 方法前调用。若不调用，默认RESTART模式订阅。

◇ 1. 函数原型

virtual void SubscribePublicTopic(THOST_TE_RESUME_TYPE nResumeType) = 0;

◇ 2. 参数

nResumeType：私有流重传方式。

THOST_TERT_RESTART：从本交易日开始重传

THOST_TERT_RESUME：从上次收到的续传

THOST_TERT_QUICK：只传送登录后私有流的内容

THOST_TERT_NONE：取消订阅公有流

◇ 3. 返回

无

◇ 4. 调用示例

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
pUserApi->SubscribePrivateTopic(THOST_TERT_RESUME);
pUserApi->SubscribePublicTopic(THOST_TERT_RESUME);
pUserApi->RegisterFront("tcp://127.0.0.1:51205");
pUserApi->Init();

```

◇ 5. FAQ

无
