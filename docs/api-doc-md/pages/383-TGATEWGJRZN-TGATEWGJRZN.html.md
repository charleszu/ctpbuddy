# TGate网关接入指南

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

TGate网关接入指南<a id="content"></a>

<a id="left_menu"></a>

  ** **<a id="0355812e-4b94-4a32-ba1a-2e9cc0b1719f"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. TGate简介
<a id="panel1"></a>

TGate(CTP交易网关)是CTP fens组件的全新换代，相较于fens有着更轻量、客户端适配成本低、易维护、易使用等优点。用户可以通过查询的方式实时获取其所在中心可接入的服务地址信息，包括交易、行情、银期转账等服务地址，灵活设计策略选择地址。即使盘中切换了交易中心，客户端可自动快速获取新的地址接入。

<a id="c3362c96-0414-46ff-9deb-05deb7dbe828"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. TGate接入场景
<a id="panel2"></a>

TGate连接CTP获取全量各中心前置地址列表以及用户与交易中心对应关系。配套提供的TGateAPi为终端实现统一接入管理提供完整的解决方案。用户终端通过TGateAPi连接TGate发起查询前置地址请求，无需登录，TGate根据客户请求返回所在交易中心全部可接入的服务地址。

无论客户是在CTP次中心，还是在异构分中心做交易，只需将其可连接的前置地址维护在CTP柜台，终端通过TGate获取地址列表，不必再做过滤即可接入所在中心的交易系统。即使盘中发生交易中心切换，对于终端也只需重新获取一次地址列表，重新连接即可，省略繁琐的站点切换。

<a id="dc406d27-255f-444c-9d70-0db8654e46a3"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 接口文件列表
<a id="panel3"></a>

| 文件名 | 详情 |
|---|---|
| TGateFtdcApi.h | 交易接口C++头文件包含交易相关的指令。 |
| TGateFtdcApiStruct.h | 包含了所有用到的数据结构的头文件。 |
| TGateFtdcApiDataType.h | 包含了所有用到的数据类型的头文件。 |
| tgateapi.dll | tgate的动态链接库和静态链接库。 |
| tgateapi.lib |  |
| error.dtd | 包含所有可能的错误信息。 |
| error.xml |  |
| FTD_tgateapi.xml | FTD协议中用到的各种包和域 |

<a id="1bae909a-bf6e-4a1c-9101-125082d03f33"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. TGateApi使用介绍
<a id="panel4"></a>

新增TGateAPI实现终端与交易网关的接入与通讯。

TGateAPI提供了两个接口，分别为CTGateFtdcApi和CTGateFtdcSpi。这两个接口对FTD协议进行了封装，方便客户端应用程序的开发。客户端应用程序可以通过CTGateFtdcApi发出操作请求，通过继承CTGateFtdcSpi并重载回调函数来处理后台服务的响应。与TradeApi的使用方法基本相同。

客户端调用网关API至少需要两个线程，一个是应用程序主线程，一个是网关API工作线程。应用程序与网关通讯是由 API工作线程驱动的。CTGateFtdcApi提供的接口是线程安全的，可以有多个应用程序线程同时发出请求。CTGateFtdcSpi提供的接口回调是由API工作线程驱动，通过实现SPI中的接口方法，可以从网关服务端收取所需数据。

如果重载的某个回调函数阻塞，则等于阻塞了API工作线程，API与网关的通讯会停止。因此，在CTGateFtdcSpi派生类的回调函数中，应迅速返回，此处不建议在回调中发送请求或处理其他代码逻辑，不要阻塞。

注意以下事项：

最大支持5000个终端连接

在途查询流控为1笔

[Release](pages/072-JYJK-CTHOSTFTDCTRADERAPI-RELEASE.html.md)不是线程安全的，多线程使用需要加锁

API请求的输入参数不能为 NULL。

API请求的返回参数，0表示正确，其他表示错误。

<a id="40d4d6ea-5639-4e10-9271-78ad81701988"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. TGateApi示例
<a id="panel5"></a>

本次API新增一个查询接口ReqQryTGIpAddrParam，入参BrokerID、UserID要求必填。

下面通过一个简单的代码示例，带大家快速了解下网关API的使用方法。该示例演示了API如何接入网关并查询服务地址的过程。

```c++

```
1   #include "TGateFtdcApi.h"
2   #include "TGateFtdcApiDataType.h"
3   #include "TGateFtdcApiStruct.h"
4   #include <iostream>
5   #include <string>
6   #include <Windows.h>
7
8   class TGUser : public CTGateFtdcSpi
9   {
10  public:
11      TGUser(){};
12      ~TGUser(){};
13
14      ///服务地址参数查询
15      void ReqQryTGIpAddrParam()
16      {
17          string m_sTGFrontAddr = "tcp://127.0.0.1";
18          string m_sBrokerid = "0001";
19          string m_sUserid = "123456789";
20          ///创建TGateApi实例
21          CTGateFtdcApi *m_pTGApi = CTGateFtdcApi::CreateTGateFtdcApi();
22          m_pTGApi->RegisterSpi(this);
23          m_pTGApi->RegisterTGAddr(const_cast<char *>(m_sTGFrontAddr.c_str()));
24          ///调用TGateApi查询服务地址请求
25          CTGateFtdcQryTGIpAddrParamField* pQryTGIpAddrParamField = new CTGateFtdcQryTGIpAddrParamField;
26          memset(pQryTGIpAddrParamField, 0, sizeof(pQryTGIpAddrParamField));
27          ///该查询接口的重要传参
28          strcpy(pQryTGIpAddrParamField->BrokerID, m_sBrokerid.c_str());
29          strcpy(pQryTGIpAddrParamField->UserID, m_sUserid.c_str());
30          int RequestID = 0;
31          int rst = m_pTGApi->ReqQryTGIpAddrParam(pQryTGIpAddrParamField, RequestID);
32          Sleep(1000);
33          delete pQryTGIpAddrParamField;
34      }
35
36      ///服务地址参数查询响应
37      void OnRspQryTGIpAddrParam(CTGateFtdcTGIpAddrParamField* pTGIpAddrParam, CTGateFtdcRspInfoField* pRspInfo, int nRequestID, bool bIsLast)
38      {
39          string m_sFrontAddr = pTGIpAddrParam->Address;
40          printf("<OnRspQryTGIpAddrParam>\n");
41      }
42  }

```

```

  注意：连到非tgate地址，会返回-4 "API验证失败"的报错

<a id="author"></a>

<a id="theme_switcher"></a>
