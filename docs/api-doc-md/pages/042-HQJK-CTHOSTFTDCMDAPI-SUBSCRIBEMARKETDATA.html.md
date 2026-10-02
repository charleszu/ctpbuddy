# SubscribeMarketData

SubscribeMarketData

订阅行情，对应响应[OnRspSubMarketData](../CTHOSTFTDCMDSPI/ONRSPSUBMARKETDATA.html)；订阅成功后，通过[OnRtnDepthMarketData](../CTHOSTFTDCMDSPI/ONRTNDEPTHMARKETDATA.html)推送行情信息。

订阅全市场合约需要把全市场合约代码都赋值给ppInstrumentID，填空不能订阅全市场合约。

目前[OnRtnDepthMarketData](../CTHOSTFTDCMDSPI/ONRTNDEPTHMARKETDATA.html)响应的数量会比请求合约的数量少，且[OnRtnDepthMarketData](../CTHOSTFTDCMDSPI/ONRTNDEPTHMARKETDATA.html)响应中会有多次bIsLast=true，此为已知问题，但不影响实际订阅的合约数量。

◇ 1. 函数原型

virtual int SubscribeMarketData(char *ppInstrumentID[], int nCount) = 0;

◇ 2. 参数

ppInstrumentID：合约数组

nCount：合约数组的数量

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
char **ppInstrumentID = new char*[50];
ppInstrumentID[0] = "T1712";
m_pUserMdApi->SubscribeMarketData(ppInstrumentID, 1);

```

◇ 5. FAQ

能否订阅重收全天的行情？

| 不行，只推送最新的行情。 |
|---|

订阅全部合约包含期货和期权所有合约后，发生OnSessionDisconnected（4097）的报错，是什么原因？

| 行情前置有个缓冲区限制，一瞬间发送太多超出缓冲区后就会有触发自我保护机制把session断开，可以尝试分批订阅，比如每订阅1000个延迟1秒。 |
|---|
