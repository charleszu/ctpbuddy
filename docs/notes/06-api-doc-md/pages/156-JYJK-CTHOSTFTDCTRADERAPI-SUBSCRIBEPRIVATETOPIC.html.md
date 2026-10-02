# SubscribePrivateTopic

SubscribePrivateTopic

订阅私有流。该方法要在[Init](../../HQJK/CTHOSTFTDCMDAPI/INIT.html) 方法前调用。若不调用则默认按照restart模式订阅。推荐使用THOST_TERT_RESTART方式订阅私有流。

◇ 1. 函数原型

virtual void SubscribePrivateTopic(THOST_TE_RESUME_TYPE nResumeType, int nSeqNo=1) = 0;

◇ 2. 参数

nResumeType： 私有流重传方式

THOST_TERT_RESTART:从本交易日开始重传（推荐）

THOST_TERT_RESUME:从上次收到的续传

序号保存在本地流水文件里，若遇到CTP交易系统清流重启，如果此时不删除原流水文件并继续用RESUME模式接入，则可能收不到私有流回报

THOST_TERT_QUICK:只传送登录后私有流的内容

THOST_TERT_RESUME_FROM_SEQ_NO:从指定序号开始重传，序号从1开始

nSeqNo：私有流序号，只在THOST_TERT_RESUME_FROM_SEQ_NO模式下有效

◇ 3. 返回

无

◇ 4. 调用示例

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\",true);
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
pUserApi->SubscribePrivateTopic(THOST_TERT_RESTART,1);
pUserApi->SubscribePublicTopic(THOST_TERT_RESTART);
pUserApi->RegisterFront("tcp://127.0.0.1:51205");
pUserApi->Init();

```

◇ 5. FAQ

无
