# ReqExecOrderAction

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqExecOrderAction<a id="content"></a>

<a id="left_menu"></a>

  ** **

执行宣告操作请求、详见[期货期权的行权、自对冲](pages/402-QTYWGZ-QHQQDHQ-ZDCGZ.html.md)

关于接口中的重要序号说明详见[接口中一些重要序号说明](pages/401-QTYWGZ-JKZYXZYXHSM.html.md)

错误响应: [OnErrRtnExecOrderAction](pages/207-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNEXECORDERACTION.html.md)，[OnRspExecOrderAction](pages/227-JYJK-CTHOSTFTDCTRADERSPI-ONRSPEXECORDERACTION.html.md)

正确响应: [OnRtnExecOrder](pages/308-JYJK-CTHOSTFTDCTRADERSPI-ONRTNEXECORDER.html.md)
<a id="9dbe8e1b-4efe-451a-b8d3-1eac2f20b2f3"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqExecOrderAction(CThostFtdcInputExecOrderActionField *pInputExecOrderAction, int nRequestID) = 0;

<a id="22690f83-879e-4f7a-b751-8beb30652a88"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputExecOrderAction：输入执行宣告操作

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcOrderRefType | ExecOrderRef | 报单引用 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填 |
| TThostFtdcExecOrderSysIDType | ExecOrderSysID | 执行宣告操作编号 | 与执行宣告记录该编号一致 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 无 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcOrderActionRefType | ExecOrderActionRef | 报单操作引用 | 无 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcFrontIDType | FrontID | 前置编号 | 无 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 无 |
| TThostFtdcActionFlagType | ActionFlag | 操作标志 | 删除 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 否 |

ActionFlag：只支持删除，不支持修改

IPAddress：手工填写本机IP地址，不自动获取。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="1e026e45-db93-4ef6-aea1-82757c12c5b3"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="4b17624c-73d1-4de4-a6dd-bc4624b4bfc3"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcInputExecOrderActionField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
a.ExecOrderActionRef = 1;
strcpy_s(a.ExecOrderRef, "00000003");
a.FrontID = 1;
a.SessionID = -7844256;
strcpy_s(a.ExchangeID, "SHFE");
strcpy_s(a.ExecOrderSysID, "         285");
a.ActionFlag = THOST_FTDC_AF_Delete;//删除
strcpy_s(a.UserID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
m_pUserApi->ReqExecOrderAction(&a, nRequestID++);

```

<a id="6d51e4d6-42b6-4395-a54d-2384bbbda27a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
