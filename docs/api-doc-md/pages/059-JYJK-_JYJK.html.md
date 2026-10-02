# 交易接口

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

交易接口<a id="content"></a>

<a id="left_menu"></a>

　[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md)类

  ** **

|  | ■ 6.7.13_API接口说明
└◆ 交易接口 |  |
|---|---|---|

<a id="fab9b6df-383b-4edd-843b-32d40c41f0e7"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 说明
<a id="panel1"></a>

交易员API提供了两个接口，分别为[CThostFtdcTraderApi](pages/060-JYJK-CTHOSTFTDCTRADERAPI-_CTHOSTFTDCTRADERAPI.html.md)和[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md)。这两个接口对FTD协议进行了封装，方便客户端应用程序的开发。客户端应用程序可以通过[CThostFtdcTraderApi](pages/060-JYJK-CTHOSTFTDCTRADERAPI-_CTHOSTFTDCTRADERAPI.html.md)发出操作请求，通过继承[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md)并重载回调函数来处理后台服务的响应。

交易员客户端应用程序至少由两个线程组成，一个是应用程序主线程，一个是交易员API工作线程。应用程序与交易系统的通讯是由 API工作线程驱动的。[CThostFtdcTraderApi](pages/060-JYJK-CTHOSTFTDCTRADERAPI-_CTHOSTFTDCTRADERAPI.html.md)提供的接口是线程安全的，可以有多个应用程序线程同时发出请求。[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md)提供的接口回调是由API工作线程驱动，通过实现SPI中的接口方法，可以从交易托管系统收取所需数据。

如果重载的某个回调函数阻塞，则等于阻塞了API工作线程，API与交易系统的通讯会停止。因此，在 [CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md) 派生类的回调函数中，通常应迅速返回，可以利用将数据放入缓冲区或通过Windows的消息机制来实现。

交易员 API 在运行过程中，会将一些数据写入本地文件中。调用[CreateFtdcTraderApi](pages/061-JYJK-CTHOSTFTDCTRADERAPI-CREATEFTDCTRADERAPI.html.md)函数，可以传递一个参数，指明存贮本地文件的路径。该路径必须在运行前已创建好。本地文件的扩展名都是”.con”。

注意以下事项：

init和release不是线程安全的，多线程使用需要加锁

在[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md) 派生类的回调函数中应及时返回，不要阻塞。

API请求的输入参数不能为 NULL。

API请求的返回参数，0表示正确，其他表示错误，详细错误编码请查error.xml。

API一个进程中linux最多开180-200左右的连接数，windows最多开400左右连接数，想建立更多连接请多开进程。

CTP系统支持的[通讯模式](pages/397-QTYWGZ-SJLSM.html.md)有三种：对话[通讯模式](pages/397-QTYWGZ-SJLSM.html.md)，私有[通讯模式](pages/397-QTYWGZ-SJLSM.html.md)，广播[通讯模式](pages/397-QTYWGZ-SJLSM.html.md)。详细说明见[[通讯模式]](pages/397-QTYWGZ-SJLSM.html.md)

<a id="c059aca8-3429-404b-80b6-83f9b112e7fa"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 代码示例
<a id="panel2"></a>

下面通过一个简单的代码示例，带大家快速了解下交易API的使用方法。该示例演示了API的初始化、订阅私有流、连接前置、登录交易系统、报单和查询等过程。
<a id="5a0c45ef-cdd6-4f5b-b032-41d9a2d1e635"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 2.1. 源代码
<a id="panel3"></a>

```
01.     //以下是tradehandler.h文件
02.
03.     #include "ThostFtdcTraderApi.h"
04.
05.     #include "traderApi.h"
06.
07.     #include <string.h>
08.
09.     #include <stdio.h>
10.
11.     #include <Windows.h>
12.
13.
14.
15.     class CTraderHandler : public CThostFtdcTraderSpi
16.
17.     {
18.
19.     private:
20.
21.       CThostFtdcTraderApi *m_ptraderapi;
22.
23.
24.
25.     public:
26.
27.       void connect()
28.
29.       {
30.
31.           //创建API实例
32.
33.           m_ptraderapi = CThostFtdcTraderApi::CreateFtdcTraderApi(".//flow/"); //必须提前创建好flow目录，流文件*.con就会在该文件夹下面创建，如果想要区别不同session创建的流水文件名称可见如下示例“.//flow/a_”
34.
35.           m_ptraderapi->RegisterSpi(this);
36.
37.           m_ptraderapi->SubscribePublicTopic(THOST_TERT_QUICK);
38.
39.           m_ptraderapi->SubscribePrivateTopic(THOST_TERT_QUICK); //设置私有流订阅模式
40.
41.           m_ptraderapi->RegisterFront("tcp://127.0.0.1:41205");
42.
43.           m_ptraderapi->Init();
44.
45.           //输出API版本信息
46.
47.           printf("%s\n", m_ptraderapi->GetApiVersion());
48.
49.       }
50.
51.
52.
53.       //释放
54.
55.       void release()
56.
57.       {
58.
59.           m_ptraderapi->Release();
60.
61.       }
62.
63.
64.
65.       //登陆
66.
67.       void login()
68.
69.       {
70.
71.           CThostFtdcReqUserLoginField t = {0};
72.
73.           strcpy_s(t.BrokerID, "1701");
74.
75.           strcpy_s(t.UserID, "1701_admin");
76.
77.           strcpy_s(t.Password, "1701_admin");
78.
79.           while (m_ptraderapi->ReqUserLogin(&t, 1)!=0)   Sleep(1000);
80.
81.       }
82.
83.
84.
85.       //结算单确认
86.
87.       void settlementinfoConfirm()
88.
89.       {
90.
91.           CThostFtdcSettlementInfoConfirmField t = {0};
92.
93.           strcpy_s(t.BrokerID, "1701");
94.
95.           strcpy_s(t.InvestorID, "00001");
96.
97.           while (m_ptraderapi->ReqSettlementInfoConfirm(&t, 2)!=0)  Sleep(1000);
98.
99.       }
100.
101.
102.
103.          //报单
104.
105.          void orderinsert()
106.
107.          {
108.
109.              CThostFtdcInputOrderField t = { 0 };
110.
111.              strcpy_s(t.BrokerID, "9999");
112.
113.              strcpy_s(t.InvestorID, "00001");
114.
115.              strcpy_s(t.UserID, "00001");
116.
117.              t.Direction = THOST_FTDC_D_Buy;
118.
119.              t.CombOffsetFlag[0] = THOST_FTDC_OF_Open;
120.
121.              t.CombHedgeFlag[0] = THOST_FTDC_HF_Speculation;
122.
123.              t.ContingentCondition = THOST_FTDC_CC_Immediately;
124.
125.              strcpy_s(t.InstrumentID, "sc1807");
126.
127.              t.ForceCloseReason = THOST_FTDC_FCC_NotForceClose;
128.
129.              t.LimitPrice = 490;
130.
131.              t.OrderPriceType = THOST_FTDC_OPT_LimitPrice;
132.
133.              t.VolumeCondition = THOST_FTDC_VC_AV;
134.
135.              t.TimeCondition = THOST_FTDC_TC_GFD;
136.
137.              t.VolumeTotalOriginal = 1;
138.
139.              strcpy_s(t.OrderRef, "0000001");
140.
141.              strcpy_s(t.ExchangeID, "INE");
142.
143.              while (m_ptraderapi->ReqOrderInsert(&t, 3) != 0) Sleep(1000);
144.
145.          }
146.
147.          //查询合约
148.
149.          void qryInstrument()
150.
151.          {
152.
153.              CThostFtdcQryInstrumentField t = { 0 };
154.
155.              strcpy_s(t.ExchangeID, "SHFE");
156.
157.              strcpy_s(t.InstrumentID, "zn1803");
158.
159.              while (m_ptraderapi->ReqQryInstrument(&t, 4) != 0) Sleep(1000);
160.
161.          }
162.
163.
164.
165.        //前置连接响应
166.
167.          void OnFrontConnected()
168.
169.          {
170.
171.              printf("OnFrontConnected\n");
172.
173.          }
174.
175.
176.
177.          //登陆请求响应
178.
179.          void OnRspUserLogin(CThostFtdcRspUserLoginField *pRspUserLogin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
180.
181.          {
182.
183.              printf("OnRspUserLogin\n");
184.
185.          }
186.
187.
188.
189.          //结算单确认响应
190.
191.          void OnRspSettlementInfoConfirm(CThostFtdcSettlementInfoConfirmField *pSettlementInfoConfirm, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
192.
193.          {
194.
195.              printf("OnRspSettlementInfoConfirm\n");
196.
197.          }
198.
199.
200.
201.          //查询合约响应
202.
203.          void OnRspQryInstrument(CThostFtdcInstrumentField *pInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
204.
205.          {
206.
207.              printf("OnRspQryInstrument\n");
208.
209.          }
210.
211.
212.          //报单通知
213.
214.          void OnRtnOrder(CThostFtdcOrderField *pOrder)
215.
216.          {
217.
218.              printf("OnRtnOrder\n");
219.
220.          }
221.
222.
223.
224.          //成交通知
225.
226.          void OnRtnTrade(CThostFtdcTradeField *pTrade)
227.
228.          {
229.
230.              printf("OnRtnTrade\n");
231.
232.          }
233.
234.
235.
236.          //报单录入请求响应
237.
238.          void OnRspOrderInsert(CThostFtdcInputOrderField *pInputOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
239.
240.          {
241.
242.              printf("OnRspOrderInsert\n");
243.
244.          }
245.
246.
247.
248.          //错误应答
249.
250.          void OnRspError(CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
251.
252.          {
253.
254.              printf("OnRspError\n");
255.
256.          }
257.
258.        };
259.
260.        //以下是main.cpp文件
261.
262.        #include "tradehandler.h"
263.
264.        int main(int argc, char *argv[])
265.
266.        {
267.
268.          CTraderHandler *trader = new CTraderHandler();
269.
270.          trader->connect();
271.
272.          Sleep(2000);
273.
274.          trader->login();
275.
276.          Sleep(2000);
277.
278.          trader->settlementinfoConfirm();
279.
280.          Sleep(2000);
281.
282.          trader->qryInstrument();
283.
284.          Sleep(2000);
285.
286.          trader->orderinsert();
287.
288.          Sleep(INFINITE);
289.
290.          trader->release();
291.
292.          return 1;
293.
294.        }

```

<a id="5b3c2873-528c-43c0-8c2f-f3be147a5de9"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 2.2. 代码说明
<a id="panel4"></a>

<a id="edc5b7a6-0112-4a46-bc91-4f140bff0680"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 2.2.1. 继承[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md)类
<a id="panel5"></a>

代码一开始通过#include "ThostFtdcTraderApi.h"，将thosttraderapi.lib中声明的类和数据类型包括进来，该头文件中有两个重要的基类：[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md)和[CThostFtdcTraderApi](pages/060-JYJK-CTHOSTFTDCTRADERAPI-_CTHOSTFTDCTRADERAPI.html.md)。

[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md)类提供了交易相关的回调接口，用户需要继承该类并重载这些接口，以获取响应数据。

[CThostFtdcTraderApi](pages/060-JYJK-CTHOSTFTDCTRADERAPI-_CTHOSTFTDCTRADERAPI.html.md)类则是提供了交易相关功能的请求接口，例如查询请求、报单请求、报价请求等。

第15行我们声明了一个CTraderHandler类，该类正是继承了[CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md)类，并且重载了[OnFrontConnected](pages/220-JYJK-CTHOSTFTDCTRADERSPI-ONFRONTCONNECTED.html.md)、[OnRspUserLogin](pages/299-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERLOGIN.html.md)、[OnRspError](pages/049-HQJK-CTHOSTFTDCMDSPI-ONRSPERROR.html.md)、[OnRspOrderInsert](pages/237-JYJK-CTHOSTFTDCTRADERSPI-ONRSPORDERINSERT.html.md)等接口。通过这种方式，例如当我们API和前置建立连接时，便会触发[OnFrontConnected](pages/220-JYJK-CTHOSTFTDCTRADERSPI-ONFRONTCONNECTED.html.md)；当我们发起登录请求后，便会触发[OnRspUserLogin](pages/299-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERLOGIN.html.md)。通过这些回调返回的数据，我们可以得知当前业务处理的结果或者获取我们想要的业务数据，进一步实现自己的业务逻辑。

第21行，声明了一个[CThostFtdcTraderApi](pages/060-JYJK-CTHOSTFTDCTRADERAPI-_CTHOSTFTDCTRADERAPI.html.md)类型的变量m_ptraderapi，后续我们会实例化它，以便我们使用traderapi提供的登录、报单和查询等功能。
<a id="e20a97ac-57ff-462d-9e66-7f7422859f2a"></a><a id="title6"></a>

<a id="header_span6"></a>◇ 2.2.2. 初始化
<a id="panel6"></a>

第27行到43行connect()函数里，实现了线程的初始化，步骤为：

Step 1 创建Api实例（[CreateFtdcTraderApi](pages/061-JYJK-CTHOSTFTDCTRADERAPI-CREATEFTDCTRADERAPI.html.md)）并为其注册对应的回调接口类的实例（[RegisterSpi](pages/036-HQJK-CTHOSTFTDCMDAPI-REGISTERSPI.html.md)）。

Step 2 订阅私有流（[SubscribePrivateTopic](pages/156-JYJK-CTHOSTFTDCTRADERAPI-SUBSCRIBEPRIVATETOPIC.html.md)）。

Step 3 订阅公有流（[SubscribePublicTopic](pages/157-JYJK-CTHOSTFTDCTRADERAPI-SUBSCRIBEPUBLICTOPIC.html.md)）。

Step 4 注册名称服务器网络地址（[RegisterNameServer](pages/069-JYJK-CTHOSTFTDCTRADERAPI-REGISTERNAMESERVER.html.md)）或注册前置机网络地址（[RegisterFront](pages/034-HQJK-CTHOSTFTDCMDAPI-REGISTERFRONT.html.md)）。

Step 5 初始化Api（[Init](pages/031-HQJK-CTHOSTFTDCMDAPI-INIT.html.md)），使API建立与前置的连接，连接成功后回调[OnFrontConnected](pages/220-JYJK-CTHOSTFTDCTRADERSPI-ONFRONTCONNECTED.html.md)。

第33行创建了一个api实例，并将其赋值给m_ptraderapi，同时参数".//flow/"指明我们api流水文件存放的目录为flow目录，因此在运行程序前，我们需要手工创建好这个目录！

流水文件以.con文件类型方式存放流水序号和交易日信息，这些信息对api正常工作有重要意义，不可随意修改和删除！并且多个api实例不可共享一个流水文件，必须通过不同文件夹或者不同流水文件名区分开。

第35行将自己定义的继承了Spi类的CTraderHandler注册给[CThostFtdcTraderApi](pages/060-JYJK-CTHOSTFTDCTRADERAPI-_CTHOSTFTDCTRADERAPI.html.md)，这样API就能将收到的各种数据通过Spi类的接口回调给用户。

第37行使用QUICK模式订阅了私有流。订阅私有流的方式有很多种，可以参见[SubscribePrivateTopic](CTHOSTFTDCTRADERSPI/SUBSCRIBEPRIVATETOPIC.html)的接口说明。

第41行注册了前置地址，如果有多个地址，可以多次调用[RegisterFront](pages/034-HQJK-CTHOSTFTDCMDAPI-REGISTERFRONT.html.md)函数传入不同的地址来实现。

第43行调用[Init](pages/031-HQJK-CTHOSTFTDCMDAPI-INIT.html.md)()函数开始正式初始化api，也就是说前面的工作只是准备工作，到了这里api才真正开始工作。此时api会按照之前设置的私有流订阅方式和注册的地址发起与CTP前置的连接。

init和release不是线程安全的，多线程使用需要加锁。

完成以上步骤后，客户端就已经和ctp的交易前置建立了连接。

第167行到173行，如果成功建立连接，[OnFrontConnected](pages/220-JYJK-CTHOSTFTDCTRADERSPI-ONFRONTCONNECTED.html.md)就会被调用。后续我们就可以调用各种API接口完成业务需求。
<a id="de6a4a41-dc87-47e4-9863-b15bccc1d099"></a><a id="title7"></a>

<a id="header_span7"></a>◇ 2.2.3. 显示api版本号
<a id="panel7"></a>

第47行，打印当前api的版本号。
<a id="f3eae166-f269-4d0b-a101-64f295d8c51e"></a><a id="title8"></a>

<a id="header_span8"></a>◇ 2.2.4. 释放api线程
<a id="panel8"></a>

第55行到61行，释放了api线程，释放后api资源会被回收，并且与前置断开连接。
<a id="91dc2085-26bf-46ed-8e49-a70eeefcd643"></a><a id="title9"></a>

<a id="header_span9"></a>◇ 2.2.5. 登录
<a id="panel9"></a>

第67到81行，前置连接成功后能够开始登陆交易系统了，先初始化登陆结构体，再赋值相应字段。brokerid、userid、password都是必要字段。之后发送登陆（[ReqUserLogin](pages/039-HQJK-CTHOSTFTDCMDAPI-REQUSERLOGIN.html.md)）指令。通过返回值可以判断是否发送成功，0表示成功，其他则表示失败，具体可以参考接口[ReqUserLogin](CTHOSTFTDCTRADERSPI/REQUSERLOGIN.html)。发送不成功可以尝试等待一小会重发。

注意这里返回0不表示登录成功，而是仅仅表示api指令发出去了。该规则同样适用于其他请求接口，建议在实际应用中做好超时重发机制，以便在网络丢包的情况下能够及时重发指令。

在响应里，FrontID和SessionID是非常重要的两个ID，他标识了本次登录的当天唯一编号。对于api用户来说，如果同一用户有多个同时在线的会话，鉴于api的私有流机制，他们彼此间可以互收私有流（例如0001投资者，session1报单，session2也能收到回报），此时就可以用这组ID来过滤非本会话的私有流数据，因为对于不同的会话，这组ID是不同的；而对于交易系统来说，则可以用来定位客户的各种交易行为。

第179到185行，为请求登录后的响应。
<a id="5d20e84c-b831-4f85-af5d-bbf74ad15aac"></a><a id="title10"></a>

<a id="header_span10"></a>◇ 2.2.6. 结算单确认
<a id="panel10"></a>

第87到99行，每一个交易日第一次登陆都需要先确认结算，不然只能做查询，不能执行报单等交易请求。

第191到197行，为请求确认结算单后的响应。
<a id="9714b841-2cf3-484e-a095-dc428176f2c2"></a><a id="title11"></a>

<a id="header_span11"></a>◇ 2.2.7. 报单
<a id="panel11"></a>

第105到143行，报入一笔限价立即单，报单属性为，多头，开仓，投机，立即单，合约为sc1807，非强平单，限价单价格为490，任意数量，当天有效，手数为1，报单引用OrderRef为"0000001",交易所为原油交易所。

注意，报单引用（OrderRef）是同一个sessionid下必须唯一且递增的一个值，不要求连续递增，但是必须要递增。而且，必须是数字，不能有英文字母！

第238到244行，如果报单填错了字段，或者客户资金不足，则报单指令发出后会被CTP拒绝，触发[OnRspOrderInsert](pages/237-JYJK-CTHOSTFTDCTRADERSPI-ONRSPORDERINSERT.html.md)回调，返回"CTP：..."这类的报错。

第214到220行，报单请求通过交易核心的合法性验证后，通过[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)返回。一开始会返回未知单，之后会实时推送更新的报单状态。

第226到232行，报单成交后响应，通过[OnRtnTrade](pages/326-JYJK-CTHOSTFTDCTRADERSPI-ONRTNTRADE.html.md)返回。
<a id="bc90a238-7ea5-4638-a59e-a458a18d0fea"></a><a id="title12"></a>

<a id="header_span12"></a>◇ 2.2.8. 查询合约
<a id="panel12"></a>

第149到161行，请求查询合约信息。对于大部分查询接口，如果请求信息填空，则会返回所有的记录；但是也有例外，比如保证金率和手续费率，C必须指定合约，可以参考对应的接口说明。

第203到209行，请求查询合约后的响应。
<a id="8cc964b8-adfb-4d55-9f22-5ce6694eb5a2"></a><a id="title13"></a>

<a id="header_span13"></a>◇ 2.2.9. 错误应答
<a id="panel13"></a>

第250到256，错误应答返回。一些特殊的报错会通过该接口返回，例如错将行情端口当成交易端口，则会报无此功能，或者为没有权限的投资者报单会报无此权限，等等。
<a id="17cc6035-a603-4fc4-a8dc-38894f3a8de1"></a><a id="title14"></a>

<a id="header_span14"></a>◇ 2.2.10. 程序运行流程
<a id="panel14"></a>

第264到292行，首先初始化CTraderHandler类。调用connect函数开始连接ctp前置，依次执行登录，结算单确认，查询合约，报单操作。

请参阅：

 >> [CThostFtdcTraderApi](pages/060-JYJK-CTHOSTFTDCTRADERAPI-_CTHOSTFTDCTRADERAPI.html.md)

 >> [CThostFtdcTraderSpi](pages/203-JYJK-CTHOSTFTDCTRADERSPI-_CTHOSTFTDCTRADERSPI.html.md)

<a id="author"></a>

<a id="theme_switcher"></a>
