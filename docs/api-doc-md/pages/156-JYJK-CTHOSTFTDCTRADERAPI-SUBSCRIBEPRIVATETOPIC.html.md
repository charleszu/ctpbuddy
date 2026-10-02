# SubscribePrivateTopic

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

SubscribePrivateTopic<a id="content"></a>

<a id="left_menu"></a>

  ** **

订阅私有流。该方法要在[Init](pages/031-HQJK-CTHOSTFTDCMDAPI-INIT.html.md) 方法前调用。若不调用则默认按照restart模式订阅。推荐使用THOST_TERT_RESTART方式订阅私有流。
<a id="4dcdc556-66d3-4fc0-a751-fd0da2a338ab"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void SubscribePrivateTopic(THOST_TE_RESUME_TYPE nResumeType, int nSeqNo=1) = 0;

<a id="fbfe180a-3388-4ba0-9968-7c8bbd35eed5"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

nResumeType： 私有流重传方式

THOST_TERT_RESTART:从本交易日开始重传（推荐）

THOST_TERT_RESUME:从上次收到的续传

序号保存在本地流水文件里，若遇到CTP交易系统清流重启，如果此时不删除原流水文件并继续用RESUME模式接入，则可能收不到私有流回报

THOST_TERT_QUICK:只传送登录后私有流的内容

THOST_TERT_RESUME_FROM_SEQ_NO:从指定序号开始重传，序号从1开始

nSeqNo：私有流序号，只在THOST_TERT_RESUME_FROM_SEQ_NO模式下有效

<a id="92086476-92b8-40a3-b5ab-9b7f5e2b2c9d"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="82999049-245e-45e4-8528-1ccee9ed4d8d"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\",true);
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
pUserApi->SubscribePrivateTopic(THOST_TERT_RESTART,1);
pUserApi->SubscribePublicTopic(THOST_TERT_RESTART);
pUserApi->RegisterFront("tcp://127.0.0.1:51205");
pUserApi->Init();

```

<a id="626f064b-e884-4af8-8e72-341258efc768"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
