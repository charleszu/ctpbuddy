# ReqSettlementInfoConfirm

ReqSettlementInfoConfirm

投资者结算结果确认，在开始每日交易前都需要先确认上一日结算单，只需要确认一次。对应响应[OnRspSettlementInfoConfirm](../CTHOSTFTDCTRADERSPI/ONRSPSETTLEMENTINFOCONFIRM.html)。

◇ 1. 函数原型

virtual int ReqSettlementInfoConfirm(CThostFtdcSettlementInfoConfirmField *pSettlementInfoConfirm, int nRequestID) = 0;

◇ 2. 参数

pSettlementInfoConfirm：投资者结算结果确认信息

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcDateType | ConfirmDate | 确认日期 | 否 |
| TThostFtdcTimeType | ConfirmTime | 确认时间 | 否 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 否 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 否 |
| TThostFtdcSettlementIDType | SettlementID | 结算编号 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcSettlementInfoConfirmField Confirm = { 0 };
strcpy_s(Confirm.BrokerID, "9999");
strcpy_s(Confirm.InvestorID, "1000001");
m_pUserApi->ReqSettlementInfoConfirm(&Confirm, nRequestID++);

```

◇ 5. FAQ

无
