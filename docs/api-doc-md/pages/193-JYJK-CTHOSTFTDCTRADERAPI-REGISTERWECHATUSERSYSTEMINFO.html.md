# RegisterWechatUserSystemInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

RegisterWechatUserSystemInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

注册用户终端信息，用于中继服务器多连接模式.用于微信小程序等应用上报信息.
<a id="c232051d-0f18-4d7b-9558-077f6fc1b651"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int RegisterWechatUserSystemInfo(CThostFtdcWechatUserSystemInfoField *pUserSystemInfo) = 0;

<a id="003b9ca1-691c-49fb-9420-91591cb0a67f"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pUserSystemInfo：微信小程序等用户系统信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcSystemInfoLenType | WechatCltSysInfoLen | 微信小程序等用户端系统内部信息长度 | 必填 |
| TThostFtdcClientSystemInfoType | WechatCltSysInfo | 微信小程序等用户端系统内部信息 | 必填 |
| TThostFtdcIPPortType | ClientIPPort | 终端IP端口 | 必填 |
| TThostFtdcTimeType | ClientLoginTime | 登录成功时间 | 必填 |
| TThostFtdcAppIDType | ClientAppID | App代码 | 必填 |
| TThostFtdcIPAddressType | ClientPublicIP | 用户公网IP | 必填 |
| TThostFtdcClientLoginRemarkType | ClientLoginRemark | 客户登录备注2 | 无 |

<a id="a4d6f048-41b5-434e-9239-66b346cdcda6"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0 正确

-1 字段长度不对

-2 非CTP采集的终端信息

-3 当前终端类型非多对多中继

-5 字段中存在非法字符或者长度超限

-6 采集结果字段错误

-7 采集库的版本类型和生产库的不一致

<a id="25bf6ad1-8638-4ab8-abb8-f022d2a472f5"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="2066fe0f-476c-40b7-b227-f07fa7f65bfb"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
