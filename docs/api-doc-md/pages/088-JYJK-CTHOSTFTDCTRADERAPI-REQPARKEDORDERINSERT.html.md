# ReqParkedOrderInsert

ReqParkedOrderInsert

预埋单录入请求

注意：由于交易所不推送组合合约的开盘信号，而服务器预埋单依赖交易所合约开盘信号触发，所以服务器预埋单暂不支持下组合合约。

响应: [OnRspParkedOrderInsert](../CTHOSTFTDCTRADERSPI/ONRSPPARKEDORDERINSERT.html)

◇ 1. 函数原型

virtual int ReqParkedOrderInsert(CThostFtdcParkedOrderField *pParkedOrder, int nRequestID) = 0;

◇ 2. 参数

pParkedOrder：预埋单

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填 |
| TThostFtdcOrderRefType | OrderRef | 报单引用 | 自定义或不填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcCombOffsetFlagType | CombOffsetFlag | 开平标志 | 必填 |
| TThostFtdcCombHedgeFlagType | CombHedgeFlag | 投机套保标志 | 必填 |
| TThostFtdcDateType | GTDDate | GTD日期 | 无 |
| TThostFtdcBusinessUnitType | BusinessUnit | 业务单元 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 无 |
| TThostFtdcParkedOrderIDType | ParkedOrderID | 预埋报单编号 | 无 |
| TThostFtdcErrorMsgType | ErrorMsg | 错误信息 | 无 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 无 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 无 |
| TThostFtdcClientIDType | ClientID | 客户代码 | 无 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcVolumeType | VolumeTotalOriginal | 数量 | 必填 |
| TThostFtdcVolumeType | MinVolume | 最小成交量 | 无 |
| TThostFtdcBoolType | IsAutoSuspend | 自动挂起标志 | 无 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcBoolType | UserForceClose | 用户强评标志 | 无 |
| TThostFtdcErrorIDType | ErrorID | 错误代码 | 无 |
| TThostFtdcBoolType | IsSwapOrder | 互换单标志 | 无 |
| TThostFtdcOrderPriceTypeType | OrderPriceType | 报单价格条件 | 限价 |
| TThostFtdcDirectionType | Direction | 买卖方向 | 必填 |
| TThostFtdcTimeConditionType | TimeCondition | 有效期类型 | 当日有效 |
| TThostFtdcVolumeConditionType | VolumeCondition | 成交量类型 | 任何数量 |
| TThostFtdcContingentConditionType | ContingentCondition | 触发条件 | 立即 |
| TThostFtdcForceCloseReasonType | ForceCloseReason | 强平原因 | 非强平 |
| TThostFtdcUserTypeType | UserType | 用户类型 | 无 |
| TThostFtdcParkedOrderStatusType | Status | 预埋单状态 | 无 |
| TThostFtdcPriceType | LimitPrice | 价格 | 必填 |
| TThostFtdcPriceType | StopPrice | 止损价 | 无 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 否 |

OrderRef：OrderRef是本地会话全局唯一编号，必须保持递增；可由用户维护，也可由系统自动填写。一定为数字。

OrderPriceType：确定输入的报单类型，比如限价单则填写THOST_FTDC_OPT_LimitPrice、市价单则填写THOST_FTDC_OPT_AnyPrice。

Direction：确定买卖方向

CombOffsetFlag：确定开平标志。注:上期所、能源交易所有平今指令，下平仓指令和平昨指令相同

CombHedgeFlag：确定投机套保标志 例:投机 THOST_FTDC_BHF_Speculation

TimeCondition：确定报单有效期类型 例:立即完成，否则撤销 THOST_FTDC_TC_IOC

VolumeCondition：确定成交量类型

ContingentCondition：确定触发条件

StopPrice：止损价，用于条件单的触发价格

ForceCloseReason：一般填写THOST_FTDC_FCC_NotForceClose 非强平

IsSwapOrder：用户互换单的标志，只有互换单需要填写，是互换单的则赋值为1

CurrencyID：不填写默认为CNY

IPAddress：手工填写本机IP地址，不自动获取。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcParkedOrderField a = { 0 };
strcpy(a.BrokerID, "9999");
strcpy(a.InvestorID, "1000001");
strcpy(a.InstrumentID, "rb1809");
strcpy(a.UserID, "1000001");
strcpy(a.ExchangeID, "SHFE");
a.OrderPriceType = THOST_FTDC_OPT_LimitPrice;
a.Direction = THOST_FTDC_D_Buy;
strcpy(a.CombOffsetFlag, "0");
strcpy(a.CombHedgeFlag, "1");
a.LimitPrice = 400;
a.VolumeTotalOriginal = 1;
a.TimeCondition = THOST_FTDC_TC_GFD;
strcpy(a.GTDDate, "");
a.VolumeCondition = THOST_FTDC_VC_CV;
a.MinVolume = 0;
a.ContingentCondition = THOST_FTDC_CC_Immediately;
a.StopPrice = 0;
a.ForceCloseReason = THOST_FTDC_FCC_NotForceClose;
a.IsAutoSuspend = 0;
m_pUserApi->ReqParkedOrderInsert(&a, nRequestID++);

```

◇ 5. FAQ

“CTP:预埋单:不支持的触发类型。”，是什么原因？

| 后台版本自6.7.2开始不再支持报入预埋条件单、预埋预埋单。 |
|---|
