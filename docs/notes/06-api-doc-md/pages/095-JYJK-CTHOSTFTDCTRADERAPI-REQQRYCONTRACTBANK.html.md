# ReqQryContractBank

ReqQryContractBank

请求查询签约银行

响应: [OnRspQryContractBank](../CTHOSTFTDCTRADERSPI/ONRSPQRYCONTRACTBANK.html)

◇ 1. 函数原型

virtual int ReqQryContractBank(CThostFtdcQryContractBankField *pQryContractBank, int nRequestID) = 0;

◇ 2. 参数

pQryContractBank：查询签约银行请求

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcBankIDType | BankID | 银行代码 | 是 |
| TThostFtdcBankBrchIDType | BankBrchID | 银行分中心代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryContractBankField a = { 0 };
strcpy_s(a.BrokerID, "9999");
m_pUserApi->ReqQryContractBank(&a, nRequestID++);

```

◇ 5. FAQ

无
