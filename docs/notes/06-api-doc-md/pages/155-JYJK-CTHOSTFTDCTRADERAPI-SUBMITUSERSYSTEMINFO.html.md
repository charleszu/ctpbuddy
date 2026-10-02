# SubmitUserSystemInfo

SubmitUserSystemInfo

上报用户终端信息，用于中继服务器操作员登录模式，操作员登录后，可以多次调用该接口上报不同客户信息。

◇ 1. 函数原型

virtual int SubmitUserSystemInfo(CThostFtdcUserSystemInfoField *pUserSystemInfo) = 0;

◇ 2. 参数

pUserSystemInfo：用户系统信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcClientSystemInfoType | ClientSystemInfo | 用户端系统内部信息 | 必填 |
| TThostFtdcIPAddressType | ClientPublicIP | 终端IP地址 | 必填 |
| TThostFtdcTimeType | ClientLoginTime | 登录成功时间 | 必填 |
| TThostFtdcAppIDType | ClientAppID | App代码 | 必填 |
| TThostFtdcSystemInfoLenType | ClientSystemInfoLen | 用户端系统内部信息长度 | 必填 |
| TThostFtdcIPPortType | ClientIPPort | 终端IP端口 | 必填 |
| TThostFtdcOldIPAddressType | reserve1 | 保留的无效字段 | 无 |
| TThostFtdcClientLoginRemarkType | ClientLoginRemark | 客户登录备注2 | 无 |
| TThostFtdcDeviceTagType | MAC | 客户终端的MAC等标识 | 必填 |

UserID：此处应该填终端的投资者账号，即InvestorID，而非操作员账号。

ClientSystemInfoLen：加密后的用户终端系统内部信息的长度

ClientSystemInfo：加密后的用户终端系统内部信息。

ClientPublicIP：用户终端IP，由中继服务器采集和填写

ClientLoginTime：用户登录中继时间，由中继服务器采集和填写

ClientAppid：用户终端的appid，由中继服务器采集和填写

◇ 3. 返回

0 正确

-1 字段长度不对

-2 非CTP采集的终端信息

-4 当前用户非一对多操作员

-5 字段中存在非法字符或者长度超限

-6 采集结果字段错误

-7 采集库的版本类型和生产库的不一致

◇ 4. 调用示例

//一对多中继终端使用方法

Step 1 中继在启动后，在API连接后发起认证

```
Authenticate();

```

Step 2 认证成功后发起登录

```
UserLogin();

```

Step 3 终端登录中继时，中继上报用户终端信息

```
char pSystemInfo[344];
  int len;
  CTP_GetSystemInfo(pSystemInfo, len);
  CThostFtdcUserSystemInfoField field1;
  memset(&field1, 0, sizeof(field));
  strcpy(field1.BrokerID, "9999");
  strcpy(field1.UserID, "00001");
  memcpy(field1.ClientSystemInfo, pSystemInfo, len);
  field1.ClientSystemInfoLen = len;
  strcpy(field1.ClientPublicIP, "127.0.0.1");
  field1.ClientIPPort = 65535;
  strcpy(field1.ClientLoginTime, "11:28:28");
  strcpy(field1.ClientAppID, "Q7");
  m_pUserApi->SubmitUserSystemInfo(&field1);

```

◇ 5. FAQ

采集信息上报时候总是提示operation not permitted，这是为什么？

| 如果提示operation not permitted，可能是AppID类型错误。例如，直连模式的AppID，却错误调用了SubmitUserSystemInfo。
此错误不会通过特定接口返回，只在标准输出中提示，例如直接在屏幕上打印出来。 |
|---|

不上报采集信息，会影响登录吗？

| 不影响登录，CTP不做控制，但这样不符合监管要求。 |
|---|

一对多模式下，如果认证成功后，密码输错导致登录失败了，是否要退出登录，重新认证和登录？

| 要退出登录，重新认证和登录。否则继续登录的话会导致submitusersysinfo调用失败！ |
|---|
