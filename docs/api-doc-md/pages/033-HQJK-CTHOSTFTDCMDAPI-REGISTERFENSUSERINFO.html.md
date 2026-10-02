# RegisterFensUserInfo

RegisterFensUserInfo

注册名字服务器用户信息，调用[RegisterNameServer](../../JYJK/CTHOSTFTDCTRADERAPI/REGISTERNAMESERVER.html)前需要先使用RegisterFensUserInfo设置登录模式。

详见[fens连接说明](../../QTYWGZ/FENS.html)

◇ 1. 函数原型

virtual void RegisterFensUserInfo(CThostFtdcFensUserInfoField * pFensUserInfo) = 0;

◇ 2. 参数

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

◇ 3. 返回

无

◇ 4. 调用示例

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

◇ 5. FAQ

无
