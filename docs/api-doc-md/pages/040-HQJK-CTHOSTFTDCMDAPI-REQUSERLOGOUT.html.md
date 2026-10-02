# ReqUserLogout

ReqUserLogout

登出请求，对应响应[OnRspUserLogout](../../JYJK/CTHOSTFTDCTRADERSPI/ONRSPUSERLOGOUT.html)。暂不支持。

◇ 1. 函数原型

virtual int ReqUserLogout(CThostFtdcUserLogoutField *pUserLogout, int nRequestID) = 0;

◇ 2. 参数

pUserLogout：用户登出请求

```
struct CThostFtdcUserLogoutField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///用户代码
    TThostFtdcUserIDType UserID;
};

```

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcUserLogoutField a = { 0 };
m_pUserApi->ReqUserLogout(&a, nRequestID++);

```

◇ 5. FAQ

无
