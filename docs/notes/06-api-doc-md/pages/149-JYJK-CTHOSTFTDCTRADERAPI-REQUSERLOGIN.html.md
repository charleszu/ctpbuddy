# ReqUserLogin

ReqUserLogin

用户登录请求，对应响应[OnRspUserLogin](../CTHOSTFTDCTRADERSPI/ONRSPUSERLOGIN.html)。

接口包含采集函数不是线程安全的，不同线程同步调用建议加锁。

◇ 1. 函数原型

virtual int [ReqUserLogin](../../HQJK/CTHOSTFTDCMDAPI/REQUSERLOGIN.html)(CThostFtdc[ReqUserLogin](../../HQJK/CTHOSTFTDCMDAPI/REQUSERLOGIN.html)Field *p[ReqUserLogin](../../HQJK/CTHOSTFTDCMDAPI/REQUSERLOGIN.html)Field, int nRequestID) = 0;

◇ 2. 参数

p[ReqUserLogin](../../HQJK/CTHOSTFTDCMDAPI/REQUSERLOGIN.html)Field：用户登录请求

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcDateType | TradingDay | 交易日 | 无 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcPasswordType | Password | 密码 | 必填 |
| TThostFtdcProductInfoType | UserProductInfo | 用户端产品信息 | 无 |
| TThostFtdcProductInfoType | InterfaceProductInfo | 接口端产品信息 | 无 |
| TThostFtdcProtocolInfoType | ProtocolInfo | 协议信息 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcPasswordType | OneTimePassword | 动态密码 | 无 |
| TThostFtdcOldIPAddressType | reserve1 | 保留的无效字段 | 无 |
| TThostFtdcIPAddressType | ClientIPAddress | 终端IP地址 | 无 |
| TThostFtdcLoginRemarkType | LoginRemark | 登录备注 | 无 |
| TThostFtdcIPPortType | ClientIPPort | 终端IP端口 | 无 |
| TThostFtdcSMSCodeType | SMSCode | 短信验证码 | 无 |

UserID：操作员代码，之后的investorid需要属于该操作员管理下。

UserProductInfo：客户端的产品信息，如软件开发商、版本号等。

用户事件中的UserProductInfo取决于认证填写的值，而非登录。

例如：SFITTraderV100。

LoginRemark：可以写登录备注，能够被交易系统的日志查询到。对于中继客户，建议终端按如下规范的“登录备注”的格式填写，格式为：

IP|MAC|UUID|APPID|手机号|…

OneTimePassword：如果期货公司启用了动态口令，客户会有一个电子口令牌，把生成的值，输入到登录报文里的OneTimePassword(动态密码)这个字段中去即可

ClientIPAddress：CTP前置主动获取，填写无效。

MacAddress：API采集，填写无效。

LoginRemark：在注备中：逗号，引号，“\r”，“\n”。四种符号在风控用户事件中会被过滤。

IPAddress：系统自动获取，填写无效。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcReqUserLoginField reqUserLogin = { 0 };
strcpy_s(reqUserLogin.BrokerID, “0000”);
strcpy_s(reqUserLogin.UserID, “00001”);
strcpy_s(reqUserLogin.Password, “123456”);
m_pUserApi->ReqUserLogin(&reqUserLogin, nRequestID++);

```

◇ 5. FAQ

现在投资者登录的时候，有时候会被锁定，请问会有什么原因？

| CTP有单独锁IP的功能。被锁定的情况下继续登录会报“CTP：登录失败次数超限，IP被禁止”。目前这个阈值一般设置为5000，单个交易日内累积计算。
CTP有锁IP+账号的功能。被锁定的情况下继续登录会报“CTP：连续登录失败次数超限，登录被禁止”。目前这个阈值一般设置为6-10，注意这里的登录错误次数CTP是连续计算的。
具体的阈值以期货公司设置的为准。 |
|---|

首次登陆如果提示修改密码，如何修改？

| 首次登陆如果提示修改密码，则必须在当前会话调用ReqUserPasswordUpdate来修改密码。 |
|---|

“CTP:用户在线会话超出上限”是什么意思？

| 表示该UserID的同时在线会话数超出了期货公司设置的阈值，具体阈值咨询期货公司柜台设置。 |
|---|

登录报“CTP：重复的登录”，是什么原因？

| 正常情况下，收到登录成功的响应后再发起登录，是没有响应的，这是因为如果前置收到登陆成功的响应，则会过滤后续的相同Session的登陆请求。
极端情况下，当前置还没收到核心返回的登录成功响应，而此时又再次收到来自客户端的登录请求，就不会过滤，核心收到重复的登录请求后就会返回“CTP：重复的登录”的错误响应。 |
|---|

登录时候报错：“CTP：连续登录失败数超限，登录被禁止”，为什么？

| 此报错说明用户连续输错密码被禁止登录了，ip被锁定。此时用户换ip或者联系期货公司解锁即可。 |
|---|
