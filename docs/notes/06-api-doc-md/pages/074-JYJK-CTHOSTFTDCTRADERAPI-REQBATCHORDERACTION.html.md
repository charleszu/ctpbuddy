# ReqBatchOrderAction

ReqBatchOrderAction

批量报单操作请求，暂不可用

错误回报: [OnErrRtnBatchOrderAction](../CTHOSTFTDCTRADERSPI/ONERRRTNBATCHORDERACTION.html)，[OnRspBatchOrderAction](../CTHOSTFTDCTRADERSPI/ONRSPBATCHORDERACTION.html)

正确回报：[OnRtnOrder](../CTHOSTFTDCTRADERSPI/ONRTNORDER.html)

◇ 1. 函数原型

virtual int ReqBatchOrderAction(CThostFtdcInputBatchOrderActionField *pInputBatchOrderAction, int nRequestID) = 0;

◇ 2. 参数

pInputBatchOrderAction：输入批量报单操作

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 无 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 无 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcOrderActionRefType | OrderActionRef | 报单操作引用 | 无 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcFrontIDType | FrontID | 前置编号 | 无 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 无 |
| TThostFtdcOldIPAddressType | reserve1 | 保留的无效字段 | 否 |

IPAddress：手工填写本机IP地址，不自动获取。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

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
