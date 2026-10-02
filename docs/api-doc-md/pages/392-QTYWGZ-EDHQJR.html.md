# 协议方式接收二代组播行情

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

协议方式接收二代组播行情<a id="content"></a>

<a id="left_menu"></a>

  ** **

本文介绍协议方式下如何通过mdfront接收二代组播行情。
<a id="c8b410ec-7564-44ac-bd0e-195c2674a318"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 术语介绍
<a id="panel1"></a>

-

协议方式：指不通过CTP封装的API，而是自行通过二代行情协议规范解码数据报文，从而获取实时行情的方式。

-

二代组播行情（下文简称二代行情）：交易所以组播方式提供的实时五档行情。因为是组播，所以接收端必须在内部网络并且要加入组播组。

-

mdfront：CTP交易系统中一个接收交易所二代组播行情的组件。该组件提供查询组播合约接口，可通过API函数[ReqQryMulticastInstrument](pages/038-HQJK-CTHOSTFTDCMDAPI-REQQRYMULTICASTINSTRUMENT.html.md)获取。

<a id="55e910db-647c-4858-bfe6-8742c0f62028"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 说明
<a id="panel2"></a>

交易所不允许投资者直接交易所报盘网去接收二代行情，但期货公司可以将二代行情转发出来给投资者使用。

二代行情包含了快照查询和增量行情实时推送功能，其中，增量行情仅包含了变动值，例如价格变动多少、成交量变动多少，只有通过快照+增量的方式才能得到一笔完整的行。而期货公司只能转发增量行情，不能转发快照查询。所以投资者除了接收增量行情外，还需要想办法获取行情快照。

行情快照建议通过mdfront获取，只需订阅行情即可。订阅方法参见[行情接口](pages/026-HQJK-_HQJK.html.md)。

最后，增量行情还有个问题，即报文里的合约是以数字编号命名，例如编号100。而CTP的行情都是合约名称，例如rb2002，为了解决合约编号到合约名称的映射关系，mdfront提供了[ReqQryMulticastInstrument](pages/038-HQJK-CTHOSTFTDCMDAPI-REQQRYMULTICASTINSTRUMENT.html.md)查询接口，用户可以通过mdapi的[ReqQryMulticastInstrument](pages/038-HQJK-CTHOSTFTDCMDAPI-REQQRYMULTICASTINSTRUMENT.html.md)查询到映射关系。

注意，普通front不提供[ReqQryMulticastInstrument](pages/038-HQJK-CTHOSTFTDCMDAPI-REQQRYMULTICASTINSTRUMENT.html.md)查询接口，调用不会返回有效数据。

综上所述，需要有三个部分才能拼装出完整的行情，分别为增量行情、行情快照和组播合约映射关系。

<a id="1c016e5b-803f-4eae-8cd1-44c6321ed06a"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 方法
<a id="panel3"></a>

下面是推荐方法，仅做参考：

1、订阅CTP行情。

可以连接mdfront组件接收行情，因为CTP端的行情是完整的行情，而非增量，所以可以作为基准快照，在后续拼装完整行情的时候使用。

2、订阅期货公司转发的二代行情。

具有解码能力的投资者，可以订阅该行情，并参考交易所二代行情协议规范做相应解码。解码后会得到增量行情，但是这里要注意，这个增量行情还不能直接拿来用，因为该行情里没有InstrumentID，只有合约编号InstrumentNo，而CTP mdfront的行情里是InstrumentID，两者无法关联。为了解决这个问题，新的mdapi增加了[ReqQryMulticastInstrument](pages/038-HQJK-CTHOSTFTDCMDAPI-REQQRYMULTICASTINSTRUMENT.html.md)接口，该接口会返回InstrumentID和InstrumentNo的对应关系，有了这个函数便可以拼出完整的行情。**注意，仅新的组播mdfront支持该函数，普通行情front和老mdfront不支持该函数！**完整行情拼装方法如下：

a)  盘前接入

开盘后，CTP推送每tick快照行情，转发行情则推送每tick增量行情。投资者程序判断同一合约两边的UpdateTime和UpdateMillisec。如果一致，则将mdfront的这笔tick快照行情作为该合约的基准快照，后续的增量行情都在此基础上做拼装，得到笔笔完整行情。

b)  盘中接入

盘中，如果客户接入，则方法同上述a，投资者程序实时判断两边的UpdateTime和UpdateMillisec，直到遇到一致，则作为基准快照。

c)  盘中丢行情

交易所的二代行情带行情序号，如果行情序号不连续了，说明行情丢包了。此时应该停止行情组装工作，重新寻找最新的行情快照，方法同a。

<a id="0a4ee0a0-b532-4a6c-aaab-391b56fb1f27"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 代码示例
<a id="panel4"></a>

下面代码演示了如何接入mdfront前置，查询组播合约，并订阅行情。

// mduserhandle.h

```
#include "ThostFtdcMdApi.h"
#include <stdio.h>
#include <Windows.h>
TThostFtdcInstrumentIDType  g_chInstrumentID;
class CMduserHandler : public CThostFtdcMdSpi
{
    private:
      CThostFtdcMdApi *m_mdApi;
    public:
    void connect()
    {
    //创建并初始化API
        m_mdApi = CThostFtdcMdApi::CreateFtdcMdApi("", true, true);
        m_mdApi->RegisterSpi(this);
        m_mdApi->RegisterFront("tcp://218.28.130.102:41413");
        m_mdApi->Init();
    }
    //登陆
    void login()
    {
        CThostFtdcReqUserLoginField t = {0};
        while (m_mdApi->ReqUserLogin(&t, 1)!=0) Sleep(1000);
    }
    //请求查询组播合约
     void ReqQryMulticastInstrument()
    {
        CThostFtdcQryMulticastInstrumentField a = { 0 };
        a.TopicID = 1001;
        strcpy_s(a.InstrumentID, "cu1905");
        while (m_mdApi->ReqQryMulticastInstrument(&a, 1)!=0) Sleep(1000);
    }
    void OnRspQryMulticastInstrument(CThostFtdcMulticastInstrumentField *pMulticastInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
    {
        if (pMulticastInstrument)
        {
            strcpy_s(g_chInstrumentID,pMulticastInstrument->InstrumentID);
        }
    }
    // 订阅行情
    void subscribe()
    {
        char **ppInstrument=new char * [50];
        ppInstrumentID[0] = g_chInstrumentID;
        while (m_mdApi->SubscribeMarketData(ppInstrument, 1)!=0) Sleep(1000);
    }
    //接收行情
    void OnRtnDepthMarketData(CThostFtdcDepthMarketDataField *pDepthMarketData)
    {
            printf("OnRtnDepthMarketData\n");
    }
};
// main.cpp
#include "mduserhandle.h"
int main(int argc, char* argv[])
{
    CMduserHandler *mduser = new CMduserHandler;
    mduser->connect();
    mduser->login();
    mduser->ReqQryMulticastInstrument();
    mduser->subscribe();
    Sleep(INFINITE);
}

```

<a id="author"></a>

<a id="theme_switcher"></a>
