# ReqHedgeCfm

ReqHedgeCfm

套保确认请求

若CTP校验通过后返回[OnRtnHedgeCfm](../CTHOSTFTDCTRADERSPI/ONRTNHEDGECFM.html)通知

如果CTP校验未通过返回[OnRspHedgeCfm](../CTHOSTFTDCTRADERSPI/ONRSPHEDGECFM.html)响应和[OnErrRtnHedgeCfm](../CTHOSTFTDCTRADERSPI/ONERRRTNHEDGECFM.html)错误回报

从交易所回来收到回报后给[OnRtnHedgeCfm](../CTHOSTFTDCTRADERSPI/ONRTNHEDGECFM.html)通知（若交易所校验未通过会有[OnErrRtnHedgeCfm](../CTHOSTFTDCTRADERSPI/ONERRRTNHEDGECFM.html)的）。

◇ 1. 函数原型

virtual int ReqHedgeCfm(CThostFtdcInputHedgeCfmField *pInputHedgeCfm, int nRequestID) = 0;

◇ 2. 参数

pInputHedgeCfm：套保确认输入基本信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填 |
| TThostFtdcVolumeType | Volume | 数量 | 必填 |
| TThostFtdcDirectionType | Direction | 买卖方向 | 必填 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 否 |
| TThostFtdcOrderRefType | OrderRef | 报单引用 | 否 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 否 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 否 |

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
