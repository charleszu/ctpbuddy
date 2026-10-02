# SubmitWechatUserSystemInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

SubmitWechatUserSystemInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

上报用户终端信息，用于中继服务器操作员登录模式.用于微信小程序等应用上报信息.
<a id="1de91252-b7cd-46ae-b0fa-947f345e424c"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int SubmitWechatUserSystemInfo(CThostFtdcWechatUserSystemInfoField *pUserSystemInfo) = 0;

<a id="78e9a8c7-cf59-44be-a723-68e60323f386"></a><a id="title2"></a>

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

<a id="8ec3b400-aae8-47bf-8d7b-3fea53663506"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0 正确

-1 字段长度不对

-2 非CTP采集的终端信息

-4 当前用户非一对多操作员

-5 字段中存在非法字符或者长度超限

-6 采集结果字段错误

-7 采集库的版本类型和生产库的不一致

<a id="70f6953b-5ef1-41fb-8ded-e4de352f6eef"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="a13380fe-d68a-4da9-b087-a384a9711498"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
