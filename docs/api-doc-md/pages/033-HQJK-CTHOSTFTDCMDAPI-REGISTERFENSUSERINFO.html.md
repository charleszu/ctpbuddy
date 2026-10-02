# RegisterFensUserInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

RegisterFensUserInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

注册名字服务器用户信息，调用[RegisterNameServer](pages/069-JYJK-CTHOSTFTDCTRADERAPI-REGISTERNAMESERVER.html.md)前需要先使用RegisterFensUserInfo设置登录模式。

详见[fens连接说明](pages/393-QTYWGZ-FENS.html.md)
<a id="cabc78f4-c6a1-4107-b14f-436edc8844e5"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void RegisterFensUserInfo(CThostFtdcFensUserInfoField * pFensUserInfo) = 0;

<a id="5dec4c01-1aa2-4785-ae16-c2278e772003"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pFensUserInfo：Fens用户信息

```
struct CThostFtdcFensUserInfoField
{
///经纪公司代码
TThostFtdcBrokerIDType BrokerID;
///用户代码
TThostFtdcUserIDType UserID;
///登录模式
TThostFtdcLoginModeType LoginMode;
};

```

<a id="b0254711-e08d-4654-ac0d-38d07bd18ab2"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

无

<a id="042df6f4-786b-4195-84c3-5cfbd5700462"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcMdApi  *pUserMdApi = CThostFtdcMdApi::CreateFtdcMdApi();
CSimpleMdHandler ash(pUserMdApi);
pUserMdApi->RegisterSpi(&ash);
CThostFtdcFensUserInfoField pFensUserInfo = { 0 };
strcpy_s(pFensUserInfo.BrokerID, "9999");
strcpy_s(pFensUserInfo.UserID, "00001");
pFensUserInfo.LoginMode = THOST_FTDC_LM_Trade;
pUserMdApi->RegisterFensUserInfo(&pFensUserInfo);
pUserMdApi-> RegisterNameServer ("tcp://127.0.0.1:41213");
pUserMdApi->Init();

```

<a id="1916c317-62b7-4544-a035-990a06a59074"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
