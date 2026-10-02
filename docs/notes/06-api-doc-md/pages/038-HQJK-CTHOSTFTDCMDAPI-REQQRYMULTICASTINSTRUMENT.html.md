# ReqQryMulticastInstrument

ReqQryMulticastInstrument

请求查询组播合约，对应响应[OnRspQryMulticastInstrument](../CTHOSTFTDCMDSPI/ONRSPQRYMULTICASTINSTRUMENT.html)

目前只有上期所、原油交易所有组播行情

上期所topicid：1001(一档行情)，1000(五档行情)

原油交易所topicid：5001(一档行情)，5000(五档行情)

**注意：该函数只有连接支持交易所组播行情的mdfront才能获得完整功能，连接其他行情前置则不能使用该函数。**

详细说明见[二代行情接入](../../QTYWGZ/EDHQJR.html)

◇ 1. 函数原型

virtual int ReqQryMulticastInstrument(CThostFtdcQryMulticastInstrumentField *pQryMulticastInstrument, int nRequestID) = 0;

◇ 2. 参数

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

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryMulticastInstrumentField a = { 0 };
a.TopicID = 1001;//对应上期所的组播行情topic
//a.TopicID = 5001;//对应原油交易所的组播行情topic
strcpy_s(g_chInstrumentID,"cu1906");
m_pUserMdApi->ReqQryMulticastInstrument(&a, 1);

```

◇ 5. FAQ

无
