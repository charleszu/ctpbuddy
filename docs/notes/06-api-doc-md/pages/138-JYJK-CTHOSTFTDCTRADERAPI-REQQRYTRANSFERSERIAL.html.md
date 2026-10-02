# ReqQryTransferSerial

ReqQryTransferSerial

请求查询转帐流水。

响应：[OnRspQryTransferSerial](../CTHOSTFTDCTRADERSPI/ONRSPQRYTRANSFERSERIAL.html)

◇ 1. 函数原型

virtual int ReqQryTransferSerial(CThostFtdcQryTransferSerialField *pQryTransferSerial, int nRequestID) = 0;

◇ 2. 参数

pQryTransferSerial：请求查询转帐流水

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 是 |
| TThostFtdcBankIDType | BankID | 银行代码 | 是 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryTransferSerialField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.AccountID, "1000001");
strcpy_s(a.BankID, “1”);
strcpy_s(a.CurrencyID, "CNY");
m_pUserApi->ReqQryTransferSerial(&a, nRequestID++);

```

◇ 5. FAQ

我在柜台上做了手工出入金，通过这个接口查不到？

| ReqQryTransferSerial不能查柜台手工出入金，只能查询到期货发起期货转银行或者银行转期货的内容。 |
|---|
