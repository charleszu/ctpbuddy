# UnSubscribeMarketData

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

UnSubscribeMarketData<a id="content"></a>

<a id="left_menu"></a>

  ** **

退订行情，对应响应[OnRspUnSubMarketData](pages/054-HQJK-CTHOSTFTDCMDSPI-ONRSPUNSUBMARKETDATA.html.md)。
<a id="d0a45bae-e403-49bb-bd3d-ad53c63e5b85"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int UnSubscribeMarketData(char *ppInstrumentID[], int nCount) = 0;

<a id="6af88d54-0738-4d54-b8d6-4d9395a5f9b7"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

ppInstrumentID：合约ID

nCount：要订阅/退订行情的合约个数

<a id="0c770a44-df93-4698-a533-6a86f2a2dab9"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="93889ccb-5a8c-4cfb-826f-970b9b06ee54"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
char **ppInstrumentID = new char*[50];
ppInstrumentID[0] = "T1712";
m_pUserMdApi->UnSubscribeMarketData(ppInstrumentID, 1);

```

<a id="fe52ec2e-7b89-4f10-a4d4-0d26bfdeba5f"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
