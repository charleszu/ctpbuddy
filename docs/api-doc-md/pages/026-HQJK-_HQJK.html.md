# 行情接口

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

行情接口<a id="content"></a>

<a id="left_menu"></a>

　[CThostFtdcMdSpi](pages/045-HQJK-CTHOSTFTDCMDSPI-_CTHOSTFTDCMDSPI.html.md)类

  ** **

|  | ■ 6.7.13_API接口说明
└◆ 行情接口 |  |
|---|---|---|

<a id="bb38dbe2-129c-4c45-aa1a-51dbbd828e31"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 说明
<a id="panel1"></a>

行情API提供了两个接口，分别为[CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md)和[CThostFtdcMdSpi](pages/045-HQJK-CTHOSTFTDCMDSPI-_CTHOSTFTDCMDSPI.html.md)。这两个接口对FTD协议进行了封装，方便客户端应用程序的开发。客户端应用程序可以通过[CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md)发出操作请求，通过继承[CThostFtdcMdSpi](pages/045-HQJK-CTHOSTFTDCMDSPI-_CTHOSTFTDCMDSPI.html.md)并重载回调函数来处理后台服务的响应。

特别注意：CTP系统在早盘系统启动时，会重演夜盘流水，此时有可能重复推送整个夜盘的行情。如果用户此时连入CTP就有可能收到这些重复行情，因此建议用户在处理行情时过滤掉重复行情，以免影响程序逻辑。

自6.6.2P8柜台版本开始front*md*se行情前置带流重启后，不再推送夜盘行情数据。

<a id="af080230-9c74-4a42-a3d4-64313e814d7e"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 代码示例
<a id="panel2"></a>

<a id="adc8982c-563a-4959-b78f-34b0df5b1250"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 2.1. 源代码
<a id="panel3"></a>

下面通过一个简单的代码示例，带大家快速了解下行情API的使用方法。该示例演示了API的初始化、连接前置、登录行情系统和订阅行情的过程。

```
01. //以下是mduserhandle.h文件
02.
03. #include "ThostFtdcMdApi.h"
04.
05. #include <stdio.h>
06.
07. #include <Windows.h>
08.
09.
10.
11. class CMduserHandler : public CThostFtdcMdSpi
12.
13. {
14.
15. private:
16.
17.   CThostFtdcMdApi *m_mdApi;
18.
19.
20.
21. public:
22.
23.   void connect()
24.
25.   {
26.
27. //创建并初始化API
28.
29.       m_mdApi = CThostFtdcMdApi::CreateFtdcMdApi(".//flow_md/", true, true);
30.
31.       m_mdApi->RegisterSpi(this);
32.
33.       m_mdApi->RegisterFront("tcp://218.28.130.102:41413");
34.
35.       m_mdApi->Init();
36.
37.   }
38.
39.
40.
41. //登陆
42.
43.   void login()
44.
45.   {
46.
47.       CThostFtdcReqUserLoginField t = {0};
48.
49.       while (m_mdApi->ReqUserLogin(&t, 1)!=0) Sleep(1000);
50.
51.   }
52.
53.
54.
55. // 订阅行情
56.
57.   void subscribe()
58.
59.   {
60.
61.       char **ppInstrument=new char * [50];
62.
63.       ppInstrument[0] = "IF1809";
64.
65.       while (m_mdApi->SubscribeMarketData(ppInstrument, 1)!=0) Sleep(1000);
66.
67.   }
68.
69.
70.
71.   //接收行情
72.
73.   void OnRtnDepthMarketData(CThostFtdcDepthMarketDataField *pDepthMarketData)
74.
75.   {
76.
77.           printf("OnRtnDepthMarketData\n");
78.
79.   }
80.
81. };
82.
83.
84.
85. // 以下是main.cpp文件
86.
87. #include "mduserhandle.h"
88.
89. int main(int argc, char* argv[])
90.
91. {
92.
93.   CMduserHandler *mduser = new CMduserHandler;
94.
95.   mduser->connect();
96.
97.   mduser->login();
98.
99.   mduser->subscribe();
100.
101.  Sleep(INFINITE);
102.
103. }

```

<a id="c4dbb39d-8c67-4466-a6d7-a885e9b63049"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 2.2. 代码说明
<a id="panel4"></a>

<a id="3c35c2c8-d8ab-4c05-ac7c-93f2559d690a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 2.2.1. 继承[CThostFtdcMdSpi](pages/045-HQJK-CTHOSTFTDCMDSPI-_CTHOSTFTDCMDSPI.html.md)类
<a id="panel5"></a>

代码一开始通过#include "ThostFtdcMdApi.h"，将thostmduserapi.lib中声明的类和数据类型包括进来，该头文件中有两个重要的基类：[CThostFtdcMdSpi](pages/045-HQJK-CTHOSTFTDCMDSPI-_CTHOSTFTDCMDSPI.html.md)和[CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md)。

[CThostFtdcMdSpi](pages/045-HQJK-CTHOSTFTDCMDSPI-_CTHOSTFTDCMDSPI.html.md)类提供了行情相关的回调接口，用户需要继承该类并重载这些接口，以获取响应数据。

[CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md)类则是提供了行情相关的请求接口，例如订阅行情请求、订阅询价请求。

第11行我们声明了一个CMduserHandler类，该类正是继承了[CThostFtdcMdSpi](pages/045-HQJK-CTHOSTFTDCMDSPI-_CTHOSTFTDCMDSPI.html.md)类，并且重载了[OnRtnDepthMarketData](pages/057-HQJK-CTHOSTFTDCMDSPI-ONRTNDEPTHMARKETDATA.html.md)等接口。通过这种方式，例如当我们API和前置建立连接成功时，便会触发[OnFrontConnected](pages/220-JYJK-CTHOSTFTDCTRADERSPI-ONFRONTCONNECTED.html.md)；当我们发起订阅行情请求成功后，便会触发[OnRspSubMarketData](pages/052-HQJK-CTHOSTFTDCMDSPI-ONRSPSUBMARKETDATA.html.md)。通过这些回调返回的数据，我们可以得知订阅是否成功，之后交易系统会通过[OnRtnDepthMarketData](pages/057-HQJK-CTHOSTFTDCMDSPI-ONRTNDEPTHMARKETDATA.html.md)实时推送行情。

第17行，声明了一个[CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md)类型的变量m_mdApi，后续我们会实例化它，创建实例后便可以调用mdapi提供的登录，订阅行情等接口功能。
<a id="15634669-32f9-4f0f-9784-8ba8d18826f1"></a><a id="title6"></a>

<a id="header_span6"></a>◇ 2.2.2. 初始化
<a id="panel6"></a>

第23行到37行connect()函数里，实现了线程的初始化，步骤为：

Step 1 创建Api实例（[CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md)）并为其注册对应的回调接口类的实例（[RegisterSpi](pages/036-HQJK-CTHOSTFTDCMDAPI-REGISTERSPI.html.md)）。

Step 2 注册名称服务器网络地址（[RegisterNameServer](pages/069-JYJK-CTHOSTFTDCTRADERAPI-REGISTERNAMESERVER.html.md)）或注册前置机网络地址（[RegisterFront](pages/034-HQJK-CTHOSTFTDCMDAPI-REGISTERFRONT.html.md)）。

Step 3 初始化Api（[Init](pages/031-HQJK-CTHOSTFTDCMDAPI-INIT.html.md)），使API建立与前置的连接，连接成功后回调[OnFrontConnected](pages/220-JYJK-CTHOSTFTDCTRADERSPI-ONFRONTCONNECTED.html.md)。

第29行创建了一个api实例，并将其赋值给m_mdApi，同时参数".//flow_md/"指明我们api流水文件存放的目录为flow目录，因此在运行程序前，我们需要手工创建好这个目录！

流水文件以.con文件类型方式存放流水序号和交易日信息，这些信息对api正常工作有重要意义，不可随意修改和删除！并且多个api实例不可共享一个流水文件，必须通过不同文件夹或者不同流水文件名区分开。虽然行情没有私有流，但是也建议遵循以上规则。

第31行将自己定义的继承了Spi类的CMduserHandler注册给[CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md)，这样API就能将收到的各种数据通过Spi类的接口回调给用户。

第33行注册了前置地址，如果有多个地址，可以多次调用[RegisterFront](pages/034-HQJK-CTHOSTFTDCMDAPI-REGISTERFRONT.html.md)函数传入不同的地址来实现。API会择优挑选一个地址进行连接。

第35行调用[Init](pages/031-HQJK-CTHOSTFTDCMDAPI-INIT.html.md)()函数开始正式初始化api，也就是说前面的工作只是准备工作，到了这里api才真正开始工作。此时api会向之前注册的地址发起与CTP前置的连接。

完成以上步骤后，客户端就已经和ctp的行情前置建立了连接，后续我们就可以调用各种API接口完成业务需求。
<a id="d2ff12b2-d7e8-4ee2-a2fe-3ace26ffb825"></a><a id="title7"></a>

<a id="header_span7"></a>◇ 2.2.3. 登录
<a id="panel7"></a>

第43到51行，前置连接成功后能够开始登陆交易系统了，先初始化登陆结构体，再赋值相应字段。对于行情的登录来说只需要调用登录接口即可，目前CTP暂时不对行情做密码校验。之后发送登陆（[ReqUserLogin](pages/039-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGIN.html.md)）指令。通过返回值可以判断是否发送成功，0表示成功，其他则表示失败，具体可以参考接口[ReqUserLogin](pages/039-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGIN.html.md)。发送不成功可以尝试等待一小会重发。

注意这里返回0不表示登录成功，而是仅仅表示api指令发出去了。该规则同样适用于其他请求接口，建议在实际应用中做好超时重发机制，以便在网络丢包的情况下能够及时重发指令。

<a id="e12ca700-6f0c-4fe4-bc4c-907098be9bc3"></a><a id="title8"></a>

<a id="header_span8"></a>◇ 2.2.4. 订阅行情
<a id="panel8"></a>

第57到67行，调用订阅行情的接口，我们这里订阅了合约"IF1809"。若需要批量订阅多个合约，则需要循环把合约输入到ppInstrumentID中去，同时别忘了更改合约数量（第二个参数）。订阅发出后通过[OnRspSubMarketData](pages/052-HQJK-CTHOSTFTDCMDSPI-ONRSPSUBMARKETDATA.html.md)响应判断是否订阅成功。

第73到81行，订阅行情后，通过[OnRtnDepthMarketData](pages/057-HQJK-CTHOSTFTDCMDSPI-ONRTNDEPTHMARKETDATA.html.md)回调推送实时行情信息。可以在此时实现自身业务逻辑。

但是如果业务逻辑比较耗时，应该在另外一个线程处理，而不应该卡在此回调里，否则会导致后续的行情堵塞在API内部，严重情况下会导致断线。

<a id="e13a3e20-9aaf-46c5-91c1-e4779a7af42b"></a><a id="title9"></a>

<a id="header_span9"></a>◇ 2.2.5. 程序运行流程
<a id="panel9"></a>

第89到103行，是我们的主函数，该函数是业务实现主体。首先初始化CMduserHandler类。调用connect函数开始连接ctp前置，然后依次执行登录和订阅行情操作。

请参阅：

 >> [CThostFtdcMdApi](pages/027-HQJK-CTHOSTFTDCMDAPI-_CTHOSTFTDCMDAPI.html.md)

 >> [CThostFtdcMdSpi](pages/045-HQJK-CTHOSTFTDCMDSPI-_CTHOSTFTDCMDSPI.html.md)

<a id="author"></a>

<a id="theme_switcher"></a>
