# ReqCancelOffsetSetting

ReqCancelOffsetSetting

对冲设置撤销请求，对应响应：对冲设置撤销请求响应[OnRspCancelOffsetSetting](../CTHOSTFTDCTRADERSPI/ONRSPCANCELOFFSETSETTING.html),对冲设置通知[OnRtnOffsetSetting](../CTHOSTFTDCTRADERSPI/ONRTNOFFSETSETTING.html),对冲设置撤销错误回报[OnErrRtnCancelOffsetSetting](../CTHOSTFTDCTRADERSPI/ONERRRTNCANCELOFFSETSETTING.html)

大商所二阶段行权优化详见[大商所行权优化二阶段业务](../../QTYWGZ/DSSHQYHEJDYW.html)

**注意：该接口仅适用大商所。**

◇ 1. 函数原型

virtual int ReqCancelOffsetSetting(CThostFtdcInputOffsetSettingField *pInputOffsetSetting, int nRequestID) = 0;

◇ 2. 参数

pInputOffsetSetting：输入的对冲设置

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 按报入填 |
| TThostFtdcInstrumentIDType | UnderlyingInstrID | 标的期货合约代码 | 按报入填 |
| TThostFtdcProductIDType | ProductID | 产品代码 | 按报入填 |
| TThostFtdcOffsetTypeType | OffsetType | 对冲类型 | 必填 |
| TThostFtdcVolumeType | Volume | 申请对冲的合约数量 | 无 |
| TThostFtdcBoolType | IsOffset | 是否对冲 | 无 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填"DCE" |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcInputOffsetSettingField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "00001");
strcpy_s(a.InstrumentID, "a2407-C-3850");
a.OffsetType = THOST_FTDC_OT_OPT_OFFSET;
strcpy_s(a.ExchangeID, "DCE);
m_pUserApi->ReqCancelOffsetSetting(&a, nRequestID++);

```

◇ 5. FAQ

无
