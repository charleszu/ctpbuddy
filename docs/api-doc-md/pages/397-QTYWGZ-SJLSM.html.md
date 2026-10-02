# 通讯模式

通讯模式

交易员API使用建立在TCP协议之上FTD协议与交易托管系统进行通讯，交易托管系统负责投资者的交易业务处理。

◇ 1. 通讯模式

FTD 协议中的所有通讯都基于某个通讯模式。通讯模式实际上就是通讯双方协同工作的方式。

FTD涉及的通讯模式共有三种：

- 对话通讯模式

对话通讯模式是指由会员端主动发起的通讯请求。该请求被交易所端接收和处理，并给予响应。例如报单、查询等。这种通讯模式与普通的客户/服务器模式相同。

- 私有通讯模式

私有通讯模式是指交易所端主动，向某个特定的会员发出的信息。例如成交回报等。

- 广播通讯模式（公有流）

广播通讯模式又称公有流，是指交易所端主动，向市场中的所有会员都发出相同的信息。例如公告、市场公共信息等。

通讯模式和网络的连接不一定存在简单的一对一的关系。也就是说，一个网络连接中可能传送多种不同通讯模式的报文，一种通讯模式的报文也可以在多个不同的连接中传送。

无论哪种通讯模式，其通讯过程都如下图所示

◇ 2. 数据流

交易托管系统支持对话通讯模式、私有通讯模式、广播通讯模式：

- 对话通讯模式下支持对话数据流和查询数据流：

对话数据流是一个双向数据流，交易托管系统发送交易请求，交易系统反馈应答。交易系统不维护对话流的状态。系统故障时，对话数据流会重置，通讯途中的数据可能会丢失。

查询数据流是一个双向数据流，交易托管系统发送查询请求，交易系统反馈应答。交易系统不维护查询流的状态。系统故障时，查询数据流会重置，通讯途中的数据可能会丢失。

- 私有通讯模式下支持私有数据流：

私有流是一个单向数据流，由交易系统发向交易托管系统，用于传送交易员私有的通知和回报信息。私有流是一个可靠的数据流，交易系统维护每个交易托管系统的私有流，在一个交易日内，交易托管系统断线后恢复连接时，可以请求交易系统发送指定序号之后的私有流数据。私有数据流向交易托管系统提供报单状态报告、成交回报更等信息。

- 广播通讯模式下支持公共数据流：

公共数据流是一个单向数据流，由交易系统发向交易托管系统，用于发送市场公共信息；公共数据流也是一个可靠的数据流，交易系统维护整个系统的公共数据流，在一个交易日内，交易托管系统断线恢复连接时，可以请求交易系统发送指定序号之后的公共数据流数据。

◇ 3. 业务与接口对照

| 业务类型 | 业务 | 请求接口 | 响应接口 | 数据流 |
|---|---|---|---|---|
| 登录 | 登录 | CThostFtdcTraderApi::ReqUserLogin | CThostFtdcTraderSpi::OnRspUserLogin | 对话流 |
| 登出 | CThostFtdcTraderApi::ReqUserLogout | CThostFtdcTraderSpi::OnRspUserLogout | 对话流 |  |
| 修改用户口令 | CThostFtdcTraderApi::ReqUserPasswordUpdate | CThostFtdcTraderSpi::OnRspUserPasswordUpdate | 对话流 |  |
| 交易 | 报单录入 | CThostFtdcTraderApi::ReqOrderInsert | CThostFtdcTraderSpi::OnRspOrderInsert | 对话流 |
| 报单操作 | CThostFtdcTraderApi::ReqOrderAction | CThostFtdcTraderSpi::OnRspOrderAction | 对话流 |  |
| 报价录入 | CThostFtdcTraderApi::ReqQuoteInsert | CThostFtdcTraderSpi::OnRspQuoteInsert | 对话流 |  |
| 报价操作 | CThostFtdcTraderApi::ReqQuoteAction | CThostFtdcTraderSpi::OnRspQuoteAction | 对话流 |  |
| 私有回报 | 成交回报 | N/A | CThostFtdcTraderSpi::OnRtnTrade | 私有流 |
| 报单回报 | N/A | CThostFtdcTraderSpi::OnRtnOrder | 私有流 |  |
| 报单录入错误回报 | N/A | CThostFtdcTraderSpi::OnErrRtnOrderInsert | 私有流 |  |
| 报单操作错误回报 | N/A | CThostFtdcTraderSpi::OnErrRtnOrderAction | 私有流 |  |
| 查询 | 报单查询 | CThostFtdcTraderApi::ReqQryOrder | CThostFtdcTraderSpi::OnRspQryOrder | 查询流 |
| 成交查询 | CThostFtdcTraderApi::ReqQryTrade | CThostFtdcTraderSpi::OnRspQryTrade | 查询流 |  |
| 投资者查询 | CThostFtdcTraderApi::ReqQryInvestor | CThostFtdcTraderSpi::OnRspQryInvestor | 查询流 |  |
| 投资者持仓查询 | CThostFtdcTraderApi::ReqQryInvestorPosition | CThostFtdcTraderSpi::OnRspQryInvestorPosition | 查询流 |  |
| 合约查询 | CThostFtdcTraderApi::ReqQryInstrument | CThostFtdcTraderSpi::OnRspQryInstrument | 查询流 |  |
| 通知 | 合约状态 | N/A | CThostFtdcTraderSpi::OnRtnInstrumentStatus | 公有流 |

[交易接口](../JYJK/_JYJK.html)和私有流接口会有相互关联，如用户报单录入[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)，马上会收到报单响应[OnRspOrderInsert](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPORDERINSERT.html)，说明交易系统已经收到报单。报单进入交易系统后，如果报单的交易状态发生变化，就会收到报单回报[OnRtnOrder](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNORDER.html)。如果报单被撮合(部分)成交，就会收到成交回报[OnRtnTrade](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNTRADE.html)。其中，一个用户的报单回报和成交回报也会被所属会员下其他的用户接受到。
