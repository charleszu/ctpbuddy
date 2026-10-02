# ReqQryOptionSelfClose

ReqQryOptionSelfClose

请求查询期权自对冲

响应: [OnRspQryOptionSelfClose](../CTHOSTFTDCTRADERSPI/ONRSPQRYOPTIONSELFCLOSE.html)

◇ 1. 函数原型

virtual int ReqQryOptionSelfClose(CThostFtdcQryOptionSelfCloseField *pQryOptionSelfClose, int nRequestID) = 0;

◇ 2. 参数

pQryOptionSelfClose：期权自对冲查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcOrderSysIDType | OptionSelfCloseSysID | 期权自对冲编号 | 是 |
| TThostFtdcTimeType | InsertTimeStart | 开始时间 | 是 |
| TThostFtdcTimeType | InsertTimeEnd | 结束时间 | 是 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

[OnRspQryOptionSelfClose](../CTHOSTFTDCTRADERSPI/ONRSPQRYOPTIONSELFCLOSE.html)：由此能定位一笔期权自对冲的报单

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcQryOptionSelfCloseField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
strcpy_s(a.ExchangeID, "SHFE");
m_pUserApi->ReqQryOptionSelfClose(&a, nRequestID++);

```

◇ 5. FAQ

无
