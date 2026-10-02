# RegisterFensUserInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

RegisterFensUserInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

注册名字服务器用户信息，调用[RegisterNameServer](pages/069-JYJK-CTHOSTFTDCTRADERAPI-REGISTERNAMESERVER.html.md)前需要先使用[RegisterFensUserInfo](pages/033-HQJK-CTHOSTFTDCMDAPI-REGISTERFENSUSERINFO.html.md)设置登录模式。

详见[fens连接说明](pages/393-QTYWGZ-FENS.html.md)
<a id="96085fba-4354-4116-aebe-a712a43a5987"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void [RegisterFensUserInfo](pages/033-HQJK-CTHOSTFTDCMDAPI-REGISTERFENSUSERINFO.html.md)(CThostFtdcFensUserInfoField * pFensUserInfo) = 0;

<a id="de7f4013-54ba-412a-8a67-eafbfef3dec0"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

CThostFtdcFensUserInfoField：Fens用户信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcLoginModeType | LoginMode | 登录模式 | 必填 |

LoginMode：填写THOST_FTDC_LM_Trade

<a id="b8a4d66b-a4ce-4e55-844d-8d2d1fb6c45d"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="ac4af966-038a-4d63-acfa-429c1bf28ac4"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcTraderApi *pUserApi = CThostFtdcTraderApi::CreateFtdcTraderApi("F:\\flow\\");
CSimpleHandler sh(pUserApi);
pUserApi->RegisterSpi(&sh);
printf(pUserApi->GetApiVersion());
pUserApi->SubscribePrivateTopic(THOST_TERT_QUICK);
pUserApi->SubscribePublicTopic(THOST_TERT_QUICK);
CThostFtdcFensUserInfoField pFensUserInfo = { 0 };
strcpy_s(pFensUserInfo.BrokerID, "9999");
strcpy_s(pFensUserInfo.UserID, "1000001");
pFensUserInfo.LoginMode = THOST_FTDC_LM_Trade;
pUserApi->RegisterFensUserInfo(&pFensUserInfo);
pUserApi-> RegisterNameServer("tcp://127.0.0.1:41205");
pUserApi->Init();

```

<a id="151807c2-9518-483d-946d-bb0292ff6abf"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
