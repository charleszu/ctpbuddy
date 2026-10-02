# ReqAuthenticate

ReqAuthenticate

客户端认证请求，对应响应[OnRspAuthenticate](../CTHOSTFTDCTRADERSPI/ONRSPAUTHENTICATE.html)。如果交易系统开启了强制终端认证，则必须认证通过后才能发起登陆；如果未开启，则不需要认证即可登陆，此时如果主动去认证，不管成功或失败，也不影响后续登陆。

◇ 1. 函数原型

virtual int ReqAuthenticate(CThostFtdcReqAuthenticateField *pReqAuthenticateField, int nRequestID) = 0;

◇ 2. 参数

pReqAuthenticateField：客户端认证请求

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcProductInfoType | UserProductInfo | 用户端产品信息 | 无 |
| TThostFtdcAuthCodeType | AuthCode | 认证码 | 必填 |
| TThostFtdcAppIDType | AppID | App代码 | 必填 |

UserProductInfo：客户端的产品信息，如软件开发商、版本号等。

CTP后台用户事件中的用户登录事件所显示的用户端产品信息取自ReqAuthentication接口里的UserProductInfo，而非[ReqUserLogin](../../HQJK/CTHOSTFTDCMDAPI/REQUSERLOGIN.html)里的。

AuthCode：认证码需要向期货公司申请得到

AppID：**必填项**，不然认证失败，如果没有则要向期货公司申请，申请的AppID必须遵循监控中心规范格式

UserID：**必填项**，用户代码。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcReqAuthenticateField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.UserID, "1000001");
strcpy_s(a.UserProductInfo, "mytest");
strcpy_s(a.AuthCode, "MLX0LEA4L4UPUCBF");
strcpy_s(a.AppID, "mytest");
m_pUserApi->ReqAuthenticate(&a, nRequestID++);

```

◇ 5. FAQ

每次重连都需要做一遍客户端认证吗？

| 重新连接都需要重新认证一下。 |
|---|

非穿透式版本，开通了强制认证，需要申请一个认证码的。穿透式版本，也需要一个穿透式的认证码。如果想同时开通非穿透式强制认证和穿透式认证，这样就要两个认证码；但API里是只需要一个认证码，那请问这种情况怎么解决？

| 这种场景是不存在的，因为穿透式和非穿透式版本的api互不兼容，所以不存在用一个api去同时做新旧前置的认证。 |
|---|

在使用“看穿式监管信息采集评测工具”时候，发现获取到的记录数和数据库中信息的条目数不匹配。这是为什么？

| 数据库中的记录为乱码的话就不显示。 |
|---|

采集信息库是否是线程安全的？能多个线程同时调用吗？

| 不是线程安全的，不能同时调用。 |
|---|

ErrorID=4043, ErrorMsg-CTP:用户与客户端授权验证失败 请问这个报错的原因是什么？

| 认证使用的appid没有授权给当前投资者账号使用。需要柜台重新指定下。 |
|---|
