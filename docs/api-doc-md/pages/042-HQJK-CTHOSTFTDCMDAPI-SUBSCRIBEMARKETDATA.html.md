# SubscribeMarketData

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

SubscribeMarketData<a id="content"></a>

<a id="left_menu"></a>

  ** **

订阅行情，对应响应[OnRspSubMarketData](pages/052-HQJK-CTHOSTFTDCMDSPI-ONRSPSUBMARKETDATA.html.md)；订阅成功后，通过[OnRtnDepthMarketData](pages/057-HQJK-CTHOSTFTDCMDSPI-ONRTNDEPTHMARKETDATA.html.md)推送行情信息。

订阅全市场合约需要把全市场合约代码都赋值给ppInstrumentID，填空不能订阅全市场合约。

目前[OnRtnDepthMarketData](pages/057-HQJK-CTHOSTFTDCMDSPI-ONRTNDEPTHMARKETDATA.html.md)响应的数量会比请求合约的数量少，且[OnRtnDepthMarketData](pages/057-HQJK-CTHOSTFTDCMDSPI-ONRTNDEPTHMARKETDATA.html.md)响应中会有多次bIsLast=true，此为已知问题，但不影响实际订阅的合约数量。

<a id="caa9d970-c69a-4663-a6f2-f33073fdf30b"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int SubscribeMarketData(char *ppInstrumentID[], int nCount) = 0;

<a id="4a017a9d-8ae6-446f-9ab3-6c7f9743a08d"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

ppInstrumentID：合约数组

nCount：合约数组的数量

<a id="5c357136-7cef-4459-b312-18febdbb9ee5"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="431deedd-2137-4f20-8822-bfc87c06c289"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
char **ppInstrumentID = new char*[50];
ppInstrumentID[0] = "T1712";
m_pUserMdApi->SubscribeMarketData(ppInstrumentID, 1);

```

<a id="801ccda8-a5f0-4f38-b441-410a5627a56a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

能否订阅重收全天的行情？<a id="region_panel_1"></a>

| 不行，只推送最新的行情。 |
|---|

<a id="region_tail_1"></a>

<a id="anchor-id-01"></a>

<a id="region_header_2"></a>

订阅全部合约包含期货和期权所有合约后，发生OnSessionDisconnected（4097）的报错，是什么原因？<a id="region_panel_2"></a>

| 行情前置有个缓冲区限制，一瞬间发送太多超出缓冲区后就会有触发自我保护机制把session断开，可以尝试分批订阅，比如每订阅1000个延迟1秒。 |
|---|

<a id="region_tail_2"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
