# ReqQryUserSession

ReqQryUserSession

请求查询用户会话，查询响应[OnRspQryUserSession](../CTHOSTFTDCTRADERSPI/ONRSPQRYUSERSESSION.html)

◇ 1. 函数原型

virtual int ReqQryUserSession(CThostFtdcQryUserSessionField *pQryUserSession, int nRequestID) = 0;

◇ 2. 参数

pQryUserSession：查询用户会话

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcFrontIDType | FrontID | 前置编号 | 必填 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 否 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |

ProductID：填写产品后，返回该产品下的所有设置。

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
