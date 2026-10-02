# ReqQryInstrument

ReqQryInstrument

请求查询合约，填空可以查询到所有合约。

响应:[OnRspQryInstrument](../CTHOSTFTDCTRADERSPI/ONRSPQRYINSTRUMENT.html)

◇ 1. 函数原型

virtual int ReqQryInstrument(CThostFtdcQryInstrumentField *pQryInstrument, int nRequestID) = 0;

◇ 2. 参数

pQryInstrument：查询合约

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcExchangeInstIDType | ExchangeInstID | 合约在交易所的代码 | 是 |
| TThostFtdcInstrumentIDType | ProductID | 产品代码 | 是 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldExchangeInstIDType | reserve2 | 保留的无效字段 | 否 |
| TThostFtdcOldInstrumentIDType | reserve3 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryInstrumentField a = { 0 };
strcpy_s(a.InstrumentID, "rb1809");
strcpy_s(a.ExchangeID, "SHFE");
m_pUserApi->ReqQryInstrument(&a, nRequestID++);

```

◇ 5. FAQ

为何查询不到郑商所跨式和宽跨式套利合约？

| 郑商所没有推跨式和宽跨式套利合约，所以查询不到此类合约。 |
|---|

为何查询中金所的EFP（期转现）合约，InstrumentName为空？

| 交易所就没有推送instrumentname。 |
|---|
