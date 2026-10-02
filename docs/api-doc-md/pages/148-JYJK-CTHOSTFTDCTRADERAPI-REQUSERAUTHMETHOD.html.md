# ReqUserAuthMethod

ReqUserAuthMethod

查询用户当前支持的认证模式，暂不支持

响应: [OnRspUserAuthMethod](../CTHOSTFTDCTRADERSPI/ONRSPUSERAUTHMETHOD.html)

◇ 1. 函数原型

virtual int ReqUserAuthMethod(CThostFtdcReqUserAuthMethodField *pReqUserAuthMethod, int nRequestID) = 0;

◇ 2. 参数

pReqUserAuthMethod：用户发出获取安全安全登陆方法请求

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcDateType | TradingDay | 交易日 | 无 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 无 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

无

◇ 5. FAQ

无
