# ReqBatchOrderAction

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqBatchOrderAction<a id="content"></a>

<a id="left_menu"></a>

  ** **

批量报单操作请求，暂不可用

错误回报: [OnErrRtnBatchOrderAction](pages/205-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNBATCHORDERACTION.html.md)，[OnRspBatchOrderAction](pages/224-JYJK-CTHOSTFTDCTRADERSPI-ONRSPBATCHORDERACTION.html.md)

正确回报：[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)
<a id="6f81f309-b559-4e49-afad-98142093cfc7"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqBatchOrderAction(CThostFtdcInputBatchOrderActionField *pInputBatchOrderAction, int nRequestID) = 0;

<a id="25c5cf95-f98d-4b14-9a83-03b7c5c63c4b"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

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

<a id="eec9783f-3e1d-43e1-9faa-6a5071304bc2"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="8e619d65-f71d-49bd-8420-aedef07e5e49"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="9112a215-36ae-4df5-ab97-6f9c28c8a29d"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
