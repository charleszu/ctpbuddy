# UnSubscribeForQuoteRsp

UnSubscribeForQuoteRsp

退订询价，对应响应[OnRspUnSubForQuoteRsp](../CTHOSTFTDCMDSPI/ONRSPUNSUBFORQUOTERSP.html)

◇ 1. 函数原型

virtual int UnSubscribeForQuoteRsp(char *ppInstrumentID[], int nCount) = 0;

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
m_pUserMdApi->SubscribeForQuoteRsp(ppInstrumentID, 1);

```

◇ 5. FAQ

无
