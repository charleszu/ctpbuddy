# ReqUserPasswordUpdate

ReqUserPasswordUpdate

用户口令更新请求，对应响应[OnRspUserPasswordUpdate](../CTHOSTFTDCTRADERSPI/ONRSPUSERPASSWORDUPDATE.html)。

◇ 1. 函数原型

virtual int ReqUserPasswordUpdate(CThostFtdcUserPasswordUpdateField *pUserPasswordUpdate, int nRequestID) = 0;

◇ 2. 参数

pUserPasswordUpdate：用户口令变更

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcPasswordType | OldPassword | 原来的口令 | 必填 |
| TThostFtdcPasswordType | NewPassword | 新的口令 | 必填 |

NewPassword：新密码需要符合复杂度要求，不然修改失败。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcUserPasswordUpdateField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.UserID, "1000001");
strcpy_s(a.OldPassword, "123456");
strcpy_s(a.NewPassword, "666666");
m_pUserApi->ReqUserPasswordUpdate(&a, nRequestID++);

```

◇ 5. FAQ

无
