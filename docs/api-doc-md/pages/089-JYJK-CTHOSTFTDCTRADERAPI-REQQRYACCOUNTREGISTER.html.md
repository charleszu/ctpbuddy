# ReqQryAccountregister

ReqQryAccountregister

请求查询银期签约关系

响应: [OnRspQryAccountregister](../CTHOSTFTDCTRADERSPI/ONRSPQRYACCOUNTREGISTER.html)

◇ 1. 函数原型

virtual int ReqQryAccountregister(CThostFtdcQryAccountregisterField *pQryAccountregister, int nRequestID) = 0;

◇ 2. 参数

pQryAccountregister：请求查询银期签约关系

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 否 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 否 |
| TThostFtdcBankIDType | BankID | 银行代码 | 否 |
| TThostFtdcBankBrchIDType | BankBranchID | 银行分支机构代码 | 否 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryAccountregisterField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.AccountID, "1000001");
strcpy_s(a.BankID, "1");
strcpy_s(a.CurrencyID, "CNY");
m_pUserApi->ReqQryAccountregister(&a, 1);

```

◇ 5. FAQ

查询无结果是什么原因？

| 只有和银行签约成功后，并且银行的签约信息返回CTP柜台后才能查询到信息。对应柜台菜单【银期转账账户信息查询】 |
|---|
