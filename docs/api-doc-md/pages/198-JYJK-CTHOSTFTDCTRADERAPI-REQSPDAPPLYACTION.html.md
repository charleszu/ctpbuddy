# ReqSpdApplyAction

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqSpdApplyAction<a id="content"></a>

<a id="left_menu"></a>

  ** **

套利确认撤销请求

若CTP校验通过后返回[OnRtnSpdApply](pages/373-JYJK-CTHOSTFTDCTRADERSPI-ONRTNSPDAPPLY.html.md)撤单状态通知

如果CTP校验未通过返回[OnRspSpdApplyAction](pages/371-JYJK-CTHOSTFTDCTRADERSPI-ONRSPSPDAPPLYACTION.html.md)响应和[OnErrRtnSpdApplyAction](pages/375-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNSPDAPPLYACTION.html.md)错误回报

从交易所回来收到回报后给[OnRtnSpdApply](pages/373-JYJK-CTHOSTFTDCTRADERSPI-ONRTNSPDAPPLY.html.md)撤单状态通知（若交易所校验未通过会有[OnErrRtnSpdApplyAction](pages/375-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNSPDAPPLYACTION.html.md)的）。
<a id="fa61c053-e9a5-4637-a3da-d59f32ec2fe1"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqSpdApplyAction(CThostFtdcInputSpdApplyActionField *pInputSpdApplyAction, int nRequestID) = 0;

<a id="9b71b169-be6d-43f4-84fe-25bae69076ac"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputSpdApplyAction：套利申请撤销

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填*2 |
| TThostFtdcOrderSysIDType | OrderSysID | 合同编号 | 必填*2 |
| TThostFtdcOrderRefType | OrderRef | 报单引用 | 必填*1 |
| TThostFtdcFrontIDType | FrontID | 前置编号 | 必填*1 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 必填*1 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 否 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 否 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 否 |

**必填*1、必填*2**：两组选一组必填，能对应要撤的报单。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="0d8458db-a069-41d7-83c9-eb1d35fa4568"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="15b038b6-3258-4b27-96d3-5ecbd26561f1"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="e4407334-c8ce-46ea-a760-5c39b4cab526"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
