# UnSubscribeForQuoteRsp

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

UnSubscribeForQuoteRsp<a id="content"></a>

<a id="left_menu"></a>

  ** **

退订询价，对应响应[OnRspUnSubForQuoteRsp](pages/053-HQJK-CTHOSTFTDCMDSPI-ONRSPUNSUBFORQUOTERSP.html.md)
<a id="ac0b4613-8b34-4edb-b20d-e056b835e8c3"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int UnSubscribeForQuoteRsp(char *ppInstrumentID[], int nCount) = 0;

<a id="bac3bd76-f5eb-45b1-b30e-b66671bd6ca7"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

ppInstrumentID：合约ID

nCount：要订阅/退订行情的合约个数

<a id="ee4dcbb2-6ef8-40f7-99de-68eb84203a2a"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="53370cd4-27bb-4c0e-92bc-3054449d7814"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
char **ppInstrumentID = new char*[50];
ppInstrumentID[0] = “sc1801”;
m_pUserMdApi->SubscribeForQuoteRsp(ppInstrumentID, 1);

```

<a id="020898c0-3810-4ade-a2f5-1479d34b51da"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
