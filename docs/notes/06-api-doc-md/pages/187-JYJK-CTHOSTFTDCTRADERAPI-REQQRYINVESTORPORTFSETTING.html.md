# ReqQryInvestorPortfSetting

ReqQryInvestorPortfSetting

投资者新型组合保证金开关查询，对应响应请求[OnRspQryInvestorPortfSetting](../CTHOSTFTDCTRADERSPI/ONRSPQRYINVESTORPORTFSETTING.html)

◇ 1. 函数原型

virtual int ReqQryInvestorPortfSetting(CThostFtdcQryInvestorPortfSettingField *pQryInvestorPortfSetting, int nRequestID) = 0;

◇ 2. 参数

pQryInvestorPortfSetting：投资者新组保设置查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |

注：返回记录是交易编码级的，除中金所可能存在多条记录外，其他交易所均是一条记录。

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
