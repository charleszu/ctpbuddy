# ReqParkedOrderInsert

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqParkedOrderInsert<a id="content"></a>

<a id="left_menu"></a>

  ** **

预埋单录入请求

注意：由于交易所不推送组合合约的开盘信号，而服务器预埋单依赖交易所合约开盘信号触发，所以服务器预埋单暂不支持下组合合约。

响应: [OnRspParkedOrderInsert](pages/239-JYJK-CTHOSTFTDCTRADERSPI-ONRSPPARKEDORDERINSERT.html.md)
<a id="58522d04-a93a-429c-b126-92674702a5d0"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqParkedOrderInsert(CThostFtdcParkedOrderField *pParkedOrder, int nRequestID) = 0;

<a id="19b1b9f2-b1d3-4762-8b6e-8253c9647514"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

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

<a id="776e06fd-8b34-4346-bd1d-bda8dbf71b3d"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="4bcd0899-ca15-47e2-82c3-9bba745cd15c"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

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

<a id="01bcd500-e597-4588-86d5-1e7708b4b94a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="anchor-id-01"></a>

<a id="region_header_1"></a>

“CTP:预埋单:不支持的触发类型。”，是什么原因？<a id="region_panel_1"></a>

| 后台版本自6.7.2开始不再支持报入预埋条件单、预埋预埋单。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
