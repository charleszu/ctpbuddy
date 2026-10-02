# RegisterFensUserInfo

RegisterFensUserInfo

注册名字服务器用户信息，调用[RegisterNameServer](REGISTERNAMESERVER.html)前需要先使用[RegisterFensUserInfo](../../HQJK/CTHOSTFTDCMDAPI/REGISTERFENSUSERINFO.html)设置登录模式。

详见[fens连接说明](../../QTYWGZ/FENS.html)

◇ 1. 函数原型

virtual void [RegisterFensUserInfo](../../HQJK/CTHOSTFTDCMDAPI/REGISTERFENSUSERINFO.html)(CThostFtdcFensUserInfoField * pFensUserInfo) = 0;

◇ 2. 参数

CThostFtdcFensUserInfoField：Fens用户信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcLoginModeType | LoginMode | 登录模式 | 必填 |

LoginMode：填写THOST_FTDC_LM_Trade

◇ 3. 返回

无

◇ 4. 调用示例

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

◇ 5. FAQ

无
