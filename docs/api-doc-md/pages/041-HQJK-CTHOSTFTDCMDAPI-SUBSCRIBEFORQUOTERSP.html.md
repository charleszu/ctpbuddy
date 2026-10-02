# SubscribeForQuoteRsp

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

SubscribeForQuoteRsp<a id="content"></a>

<a id="left_menu"></a>

  ** **

订阅询价，对应响应[OnRspSubForQuoteRsp](pages/051-HQJK-CTHOSTFTDCMDSPI-ONRSPSUBFORQUOTERSP.html.md)；订阅成功后推送[OnRtnForQuoteRsp](pages/309-JYJK-CTHOSTFTDCTRADERSPI-ONRTNFORQUOTERSP.html.md)。

询价相关业务请参考：[做市商询价和报价](pages/388-QTYWGZ-BJHXJ.html.md)
<a id="ca2b8cd0-1087-4cae-8e99-e5afb36137db"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int SubscribeForQuoteRsp(char *ppInstrumentID[], int nCount) = 0;

<a id="25b4fe51-7941-470d-864e-8eab21f2f425"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

ppInstrumentID：合约ID

nCount：要订阅/退订行情的合约个数

<a id="3da0e04d-a613-487f-90ff-e41ed7d5289f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="dfdae8a1-44d5-4641-8a56-8fd43855aad0"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
char **ppInstrumentID = new char*[50];
ppInstrumentID[0] = “sc1801”;
int result = m_pUserMdApi->SubscribeForQuoteRsp(ppInstrumentID, 1);

```

<a id="9dc0768b-5e4b-42d2-87d9-aedf4fddb38e"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
