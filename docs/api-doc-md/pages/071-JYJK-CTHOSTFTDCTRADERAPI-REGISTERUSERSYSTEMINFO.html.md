# RegisterUserSystemInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

RegisterUserSystemInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

注册用户终端信息，用于中继服务器多连接模式

需要在终端认证成功后，用户登录前调用该接口
<a id="a0d70fa9-3995-466f-92e1-9d26e678bac2"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int RegisterUserSystemInfo(CThostFtdcUserSystemInfoField *pUserSystemInfo) = 0;

<a id="9d7faa30-91f8-42ba-ad51-0ed3d8ad145d"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

CThostFtdcUserSystemInfoField ：用户系统信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcClientSystemInfoType | ClientSystemInfo | 用户端系统内部信息 | 必填 |
| TThostFtdcIPAddressType | ClientPublicIP | 终端IP端口 | 必填 |
| TThostFtdcTimeType | ClientLoginTime | 登录成功时间 | 必填 |
| TThostFtdcAppIDType | ClientAppID | App代码 | 必填 |
| TThostFtdcSystemInfoLenType | ClientSystemInfoLen | 用户端系统内部信息长度 | 必填 |
| TThostFtdcIPPortType | ClientIPPort | 终端IP端口 | 必填 |
| TThostFtdcOldIPAddressType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcClientLoginRemarkType | ClientLoginRemark | 客户登录备注2 | 否 |
| TThostFtdcDeviceTagType | MAC | 客户终端的MAC等标识 | 必填 |

ClientSystemInfoLen：存储的为加密后的用户终端系统内部信息的长度

ClientSystemInfo：存储的为加密后的用户终端系统内部信息。

ClientPublicIP：存储的为用户终端IP，由中继服务器采集和填写

ClientLoginTime：存储的为用户登录中继时间，由中继服务器采集和填写

ClientAppid：存储的为用户终端的appid，由中继服务器采集和填写

<a id="5ead3ac7-719f-4ea7-b709-65bac966b444"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0 正确

-1 字段长度不对

-2 非CTP采集的终端信息

-3 当前终端类型非多对多中继

-5 字段中存在非法字符或者长度超限

-6 采集结果字段错误

<a id="anchor-id-01"></a>

-7 采集库的版本类型和生产库的不一致

<a id="3412abeb-51e9-4333-88cd-c6253b5723bf"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

//多对多中继终端使用流程

Step 1 终端侧采集信息 向中继发起登录 并将终端信息发送给中继

```
char pSystemInfo[344];
int len;
CTP_GetSystemInfo(pSystemInfo, len);

```

Step 2 中继收到终端的登录请求，发起终端认证

```
ReqAuthenticate();

```

Step 3 终端认证成功后，中继上报用户信息

```
RegisterUserSystemInfo()
{
    char pSystemInfo[344];
    int len;
    ////将从终端得到的信息赋值给下面结构体
    CThostFtdcUserSystemInfoField field;
    memset(&field, 0, sizeof(field));
    strcpy(field.BrokerID, "9999");
    strcpy(field.UserID, "00001");
    memcpy(field.ClientSystemInfo, pSystemInfo, len);
    field.ClientSystemInfoLen = len;
    strcpy(field.ClientPublicIP, "127.0.0.1");
    field.ClientIPPort = 65535;
    strcpy(field.ClientLoginTime, "11:28:28");
    strcpy(field.ClientAppID, "aaa");
    m_pUserApi->RegisterUserSystemInfo(&field);
}

```

Step 4 中继上报成功后，发起登录

```
ReqUserLogin();

```

Step 5 中继登录成功

<a id="43dabd23-6330-4fa3-bded-edb580026048"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

采集信息上报时候总是提示operation not permitted，这是为什么？<a id="region_panel_1"></a>

| 如果提示operation not permitted，可能是AppID类型错误。例如，直连模式的AppID，却错误调用了SubmitUserSystemInfo。
此错误不会通过特定接口返回，只在标准输出中提示，例如直接在屏幕上打印出来。 |
|---|

<a id="region_tail_1"></a>

<a id="region_header_2"></a>

不上报采集信息，会影响登录吗？<a id="region_panel_2"></a>

| 不影响登录，CTP不做控制，但这样不符合监管要求。 |
|---|

<a id="region_tail_2"></a>

<a id="region_header_3"></a>

多对多中继代理模式的AppID怎么填？是填RelayAppID还是ClientAppID？<a id="region_panel_3"></a>

| 认证时用relayappid，上报时填客户端的appid。 |
|---|

<a id="region_tail_3"></a>

<a id="region_header_4"></a>

使用中继模式，已经认证成功了。用工具查看不到采集获得的信息，但是在代码里看到的pSystemInfo不为空的。这可能是什么原因？<a id="region_panel_4"></a>

| 采集不落库，基本上都是采集的数据在转储时被截断了。
一定要使用memcpy，不要把采集的数据当成字符串，他不是字符串。
还有一种常见错误，ClientSystemInfoLen不按照采集库的返回值填写，而是自己用strlen()之类的函数去获取。 |
|---|

<a id="region_tail_4"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
