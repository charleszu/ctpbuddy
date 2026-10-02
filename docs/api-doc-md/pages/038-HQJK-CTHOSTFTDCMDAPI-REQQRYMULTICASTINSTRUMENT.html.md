# ReqQryMulticastInstrument

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryMulticastInstrument<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询组播合约，对应响应[OnRspQryMulticastInstrument](pages/050-HQJK-CTHOSTFTDCMDSPI-ONRSPQRYMULTICASTINSTRUMENT.html.md)

目前只有上期所、原油交易所有组播行情

上期所topicid：1001(一档行情)，1000(五档行情)

原油交易所topicid：5001(一档行情)，5000(五档行情)

**注意：该函数只有连接支持交易所组播行情的mdfront才能获得完整功能，连接其他行情前置则不能使用该函数。**

详细说明见[二代行情接入](pages/392-QTYWGZ-EDHQJR.html.md)
<a id="13bbb78b-ba45-4707-92bf-761102929bf4"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryMulticastInstrument(CThostFtdcQryMulticastInstrumentField *pQryMulticastInstrument, int nRequestID) = 0;

<a id="bf4a3a44-a61f-40fb-acee-d3de4d1d4302"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

QryMulticastInstrument：请求查询组播合约

```
    struct CThostFtdcQryMulticastInstrumentField
    {
        ///主题号
        TThostFtdcInstallIDType TopicID;
        ///合约代码
        TThostFtdcInstrumentIDType  InstrumentID;
    };

```

TopicID：对应交易所组播行情主题号。

<a id="4ebc2b03-fbb9-4e98-84f1-4bce422e1f54"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="526f942f-0e82-4413-8784-0cb458c9d63f"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryMulticastInstrumentField a = { 0 };
a.TopicID = 1001;//对应上期所的组播行情topic
//a.TopicID = 5001;//对应原油交易所的组播行情topic
strcpy_s(g_chInstrumentID,"cu1906");
m_pUserMdApi->ReqQryMulticastInstrument(&a, 1);

```

<a id="a45f996d-93f5-4a74-858c-d8c8247d2bdf"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
