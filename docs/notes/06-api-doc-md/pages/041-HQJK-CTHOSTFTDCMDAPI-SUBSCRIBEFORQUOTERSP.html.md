# SubscribeForQuoteRsp

SubscribeForQuoteRsp

订阅询价，对应响应[OnRspSubForQuoteRsp](../CTHOSTFTDCMDSPI/ONRSPSUBFORQUOTERSP.html)；订阅成功后推送[OnRtnForQuoteRsp](../../JYJK/CTHOSTFTDCTRADERSPI/ONRTNFORQUOTERSP.html)。

询价相关业务请参考：[做市商询价和报价](../../QTYWGZ/BJHXJ.html)

◇ 1. 函数原型

virtual int SubscribeForQuoteRsp(char *ppInstrumentID[], int nCount) = 0;

◇ 2. 参数

ppInstrumentID：合约ID

nCount：要订阅/退订行情的合约个数

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
char **ppInstrumentID = new char*[50];
ppInstrumentID[0] = “sc1801”;
int result = m_pUserMdApi->SubscribeForQuoteRsp(ppInstrumentID, 1);

```

◇ 5. FAQ
