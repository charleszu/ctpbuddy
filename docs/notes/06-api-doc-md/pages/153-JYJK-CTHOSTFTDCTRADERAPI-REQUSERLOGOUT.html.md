# ReqUserLogout

ReqUserLogout

登出请求，对应响应[OnRspUserLogout](../CTHOSTFTDCTRADERSPI/ONRSPUSERLOGOUT.html)。

◇ 1. 函数原型

virtual int [ReqUserLogout](../../HQJK/CTHOSTFTDCMDAPI/REQUSERLOGOUT.html)(CThostFtdcUserLogoutField *pUserLogout, int nRequestID) = 0;

◇ 2. 参数

pUserLogout：用户登出请求

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcUserLogoutField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.UserID, "1000001");
m_pUserApi->ReqUserLogout(&a, nRequestID++);

```

◇ 5. FAQ

用户调用[ReqUserLogout](../../HQJK/CTHOSTFTDCMDAPI/REQUSERLOGOUT.html)后是否会自动重连？

| 会，Logout后，触发OnFrontDisconnected，能自动OnFrontConnected。 |
|---|
