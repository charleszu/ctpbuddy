# ReqQuoteAction

ReqQuoteAction

报价操作请求，用于撤销报价

错误响应:[OnRspQuoteAction](../CTHOSTFTDCTRADERSPI/ONRSPQUOTEACTION.html)   [OnErrRtnQuoteAction](../CTHOSTFTDCTRADERSPI/ONERRRTNQUOTEACTION.html)

正确响应:[OnRtnQuote](../CTHOSTFTDCTRADERSPI/ONRTNQUOTE.html)  [OnRtnOrder](../CTHOSTFTDCTRADERSPI/ONRTNORDER.html)

详见[做市商询价和报价](../../QTYWGZ/BJHXJ.html)

关于接口中的重要序号说明详见[接口中一些重要序号说明](../../QTYWGZ/JKZYXZYXHSM.html)

◇ 1. 函数原型

virtual int ReqQuoteAction(CThostFtdcInputQuoteActionField *pInputQuoteAction, int nRequestID) = 0;

◇ 2. 参数

pInputQuoteAction：输入报价操作

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填*2 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填*2 |
| TThostFtdcOrderRefType | QuoteRef | 报价引用 | 必填*1 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填*2 |
| TThostFtdcOrderSysIDType | QuoteSysID | 报价编号 | 必填*2 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填*1 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcClientIDType | ClientID | 客户代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcOrderActionRefType | QuoteActionRef | 报价操作引用 | 无 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcFrontIDType | FrontID | 前置编号 | 必填*1 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 必填*1 |
| TThostFtdcActionFlagType | ActionFlag | 操作标志 | 必填 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 否 |
| TThostFtdcOrderMemoType | OrderMemo | 报单回显字段 | 无 |
| TThostFtdcSequenceNo12Type | SessionReqSeq | session上请求计数 api自动维护 | 无 |

**必填*1、必填*2**：两组选一组必填，能对应要撤的报价单。

FrontID：对应要撤销的报价的前置编号

SessionID：对应要撤销的报价的会话编号

QuoteSysID: 对应要撤销的报价的编号

ActionFlag：操作标志，支持删除，目前不支持改单。想实现改单操作可以先撤销之后再重新报单。

IPAddress：手工填写本机IP地址，不自动获取。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

OrderMemo:报单回显字段，OrderMemo字段可供终端厂商标记订单使用，CTP不做处理，即终端填写什么CTP就返回什么

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

SessionReqSeq：一个session中，一笔业务流程周期中唯一序号，此序号由API自动填写，客户维护无效。

OrderMemo：该字段CTP不做任何处理，即终端填写什么CTP就返回什么。可以用来给多账户系统做标记用。

◇ 3. 返回

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

◇ 4. 调用示例

```
CThostFtdcInputQuoteActionField t = { 0 };
strcpy_s(t.BrokerID, "9999");
strcpy_s(t.InvestorID, "1000001");
strcpy_s(t.UserID, "1000001");
strcpy_s(t.ExchangeID, "SHFE");
strcpy_s(t.QuoteRef, "           1");
t.FrontID = 1;
t.SessionID = 6442531;
t.ActionFlag = THOST_FTDC_AF_Delete;
strcpy_s(t.InstrumentID, "rb1809");
m_pUserApi->ReqQuoteAction(&t, nRequestID++);

```

◇ 5. FAQ

无
