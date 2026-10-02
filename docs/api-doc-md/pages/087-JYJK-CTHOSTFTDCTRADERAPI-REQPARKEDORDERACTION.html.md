# ReqParkedOrderAction

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqParkedOrderAction<a id="content"></a>

<a id="left_menu"></a>

  ** **

预埋撤单录入请求

响应: [OnRspParkedOrderAction](pages/238-JYJK-CTHOSTFTDCTRADERSPI-ONRSPPARKEDORDERACTION.html.md)
<a id="93b6fca1-633b-4f23-be21-3f7051c12247"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqParkedOrderAction(CThostFtdcParkedOrderActionField *pParkedOrderAction, int nRequestID) = 0;

<a id="3a343f2b-7e9e-4e9f-911d-56d726c83989"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pParkedOrderAction：输入预埋单操作

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcOrderRefType | OrderRef | 报单引用 | 必填*1 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 无 |
| TThostFtdcOrderSysIDType | OrderSysID | 报单编号 | 必填*2 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填 |
| TThostFtdcParkedOrderActionIDType | ParkedOrderActionID | 预埋撤单单编号 | 无 |
| TThostFtdcErrorMsgType | ErrorMsg | 错误信息 | 无 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcOrderActionRefType | OrderActionRef | 报单操作引用 | 无 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcFrontIDType | FrontID | 前置编号 | 必填*1 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 必填*1 |
| TThostFtdcVolumeType | VolumeChange | 数量变化 | 无 |
| TThostFtdcErrorIDType | ErrorID | 错误代码 | 无 |
| TThostFtdcActionFlagType | ActionFlag | 操作标志 | 必填 |
| TThostFtdcUserTypeType | UserType | 用户类型 | 无 |
| TThostFtdcParkedOrderStatusType | Status | 预埋单状态 | 无 |
| TThostFtdcPriceType | LimitPrice | 价格 | 无 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 否 |

**必填*1、必填*2**：两组选一组必填，能对应要撤的报单。

OrderRef：对应要撤销的那笔报单的报单引用

FrontID: 对应要撤销的那笔报单的前置编号

SessionID: 对应要撤销的那笔报单的会话编号

ExchangeID: 对应要撤销的那笔报单的交易所ID

OrderSysID: 对应要撤销的那笔报单的报单编号

ActionFlag：只支持删除，不支持修改

IPAddress：手工填写本机IP地址，不自动获取。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="39cd07d9-2a20-415f-b590-1e0f5e292243"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="2b0536ba-bf24-4b74-aa2c-c3b09ccf6040"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcParkedOrderActionField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.ExchangeID, "SHFE");
strcpy_s(a.OrderSysID, "    10061782");
strcpy_s(a.UserID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
ActionFlag = THOST_FTDC_AF_Delete;
m_pUserApi->ReqParkedOrderAction(&a, nRequestID++);

```

<a id="b67d9f78-5845-4dad-87fb-47ce33f8edf9"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
