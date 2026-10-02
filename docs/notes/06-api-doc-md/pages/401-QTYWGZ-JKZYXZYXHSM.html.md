# 接口中一些重要序号说明

接口中一些重要序号说明

◇ 1. 报单

◇ 1.1. FrontID + SessionID + OrderRef

当报单报入CTP后，这组序号可以唯一定位一笔报单。可以用于跟踪报单回报的整个生命周期，但无法跟踪成交回报，因为成交回报里没有这组序号。

注意：CTP清流重启后报单中的该组序号会重新分配，即对于同一笔报单，清流重启前后的这组序号会不一致，使用上需注意。

◇ 1.2. ExchangeID + TraderID + OrderLocalID

当报单被CTP接受后，系统会分配OrderLocalID并发往交易所，这组序号可以用于跟踪此后报单的生命周期，包括报单回报和成交回报。

当报单被CTP拒绝后，不会分配OrderLocalID，此时仍要使用第1组序号跟踪报单。

注意：CTP清流重启后报单回报中的该组序号保持不变。

◇ 1.3. ExchangeID + OrderSysID

当报单被交易所接受后，交易所系统会分配OrderSysID并给CTP推送报单回报，这组序号可以用于跟踪此后报单的生命周期，包括报单回报和成交回报。

当报单被交易所拒绝后，交易所不会分配OrderSysID，此时仍要使用第1和第2组序号跟踪报单。

注意：CTP清流重启后报单回报中的该组序号保持不变。

◇ 1.4. RequestID

[ReqOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)中的RequestID与[OnRtnOrder](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNORDER.html)中的相对应，可以用于跟踪报单回报的整个生命周期，但无法跟踪成交回报，因为成交回报里没有这个字段。

注意：CTP清流重启后该序号置零。

◇ 1.5. RelativeOrderSysID

用于关联条件报单和触发后的报单回报。当报入条件单后，会生成OrderSysID=TJBD_XX的报单，此时状态为未触发；当行情满足条件并触发条件单后，会生成一笔新的报单，其RelativeOrderSysID=TJBD_XX。

注意：CTP清流重启后RelativeOrderSysID会清空，无法再关联条件单。

◇ 2. 报价

◇ 2.1. FrontID + SessionID + QuoteRef

当报价报入CTP后，这组序号可以唯一定位一笔报价。可以用于跟踪报价回报的整个生命周期，但无法跟踪报价衍生单，因为报价衍生单回报（[OnRtnOrder](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNORDER.html)和[OnRtnTrade](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNTRADE.html)）里没有这组序号。

注意：CTP清流重启后报价中的该组序号会重新分配，即对于同一笔报价，清流重启前后的这组序号会不一致，使用上需注意。

◇ 2.2. FrontID + SessionID + AskOrderRef/BidOrderRef

当报价报入CTP后，可以通过报价回报中的这组序号跟踪报价衍生单，AskOrderRef对应报价衍生卖单的OrderRef，BidOrderRef对应报价衍生买单的OrderRef。规则同上面报单接口的序号1。

注意：CTP清流重启后报单中的该组序号会重新分配。

◇ 2.3. ExchangeID + TraderID + QuoteLocalID

当报价被CTP接受后，系统会分配QuoteLocalID并发往交易所，这组序号可以用于跟踪此后报价的生命周期，但无法跟踪报价衍生单，因为报价衍生单回报里没有这组序号。衍生单的序号规则同上面报价接口序号2。

当报价被CTP拒绝后，不会分配QuoteLocalID，此时仍要使用第1组序号跟踪报价。

注意：CTP清流重启后报价回报中的该组序号保持不变。

◇ 2.4. ExchangeID + QuoteSysID

当报价被交易所接受后，交易所系统会分配QuoteSysID并给CTP推送报价回报，这组序号可以用于跟踪此后报价的生命周期，但无法跟踪报价衍生单，因为报价衍生单回报里没有这组序号。

当报价被交易所拒绝后，交易所不会分配QuoteSysID，此时仍要使用第1和第3组序号跟踪报价。

注意：CTP清流重启后报价回报中的该组序号保持不变。

◇ 2.5. ExchangeID + AskOrderSysID/BidOrderSysID

当报价被交易所接受后，上期所、能源中心、中金所可以通过报价回报中的这组序号跟踪报价衍生单，AskOrderSysID对应报价衍生卖单的OrderSysID，BidOrderSysID对应报价衍生买单的OrderSysID。衍生单的序号规则同上面报单接口。

大商所和郑商所的报价回报中，AskOrderSysID和BidOrderSysID为空。

大商所报价回报中QuoteSysID对应买衍生单的的OrderSysID，QuoteSysID+1的值对应卖衍生单的OrderSysID，比如报价回报中QuoteSysID为110，那么衍生单的OrderSysID分别是110和111。

郑商所报价回报中QuoteSysID的前缀加上'a'对应卖衍生单的OrderSysID，QuoteSysID的前缀加上'b'对应买衍生单的OrderSysID，比如报价回报的QuoteSysID为110，那么衍生单的OrderSysID分别是a110、b110。

注意：CTP清流重启后报价回报中的该组序号保持不变。

◇ 2.6. RequestID

[ReqQuoteInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQQUOTEINSERT.html)中的RequestID与[OnRtnQuote](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNQUOTE.html)和[OnRtnOrder](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNORDER.html)中的相对应，用于跟踪报价回报和报价衍生单回报的整个生命周期。例如[ReqQuoteInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQQUOTEINSERT.html)中RequestID=888，则[OnRtnQuote](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNQUOTE.html)的RequestID=888，买衍生单的[OnRtnOrder](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNORDER.html)中的RequestID=888，卖衍生单[OnRtnOrder](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNORDER.html)中的RequestID=888。

注意：CTP清流重启后该序号置零。

◇ 3. 询价

◇ 3.1. ForQuoteRef

当询价请求报入CTP后，该字段用来唯一定位一笔询价。

注意：CTP清流重启后报单中的该组序号会重新分配。

◇ 3.2. ForQuoteSysID

当询价被交易所接受后，交易所系统分配ForQuoteSysID，可以用于跟踪此后询价的的生命周期。

注意：CTP清流重启后报价回报中的该组序号保持不变。

◇ 4. 行权

◇ 4.1. FrontID + SessionID + ExecOrderRef

当行权请求报入CTP后，该组序号用来唯一定位一笔行权。

注意：CTP清流重启后报单中的该组序号会重新分配。

◇ 4.2. ExchangeID + TraderID + ExecOrderLocalID

当行权被CTP接受后，可以通过行权回报中的这组序号跟踪此后行权的生命周期。

注意：CTP清流重启后报价回报中的该组序号保持不变。

◇ 4.3. ExchangeID + ExecOrderSysID

当行权被交易所接受后，可以通过行权回报中的这组序号跟踪此后行权的生命周期。

注意：CTP清流重启后报价回报中的该组序号保持不变。

◇ 5. 自对冲

◇ 5.1. FrontID + SessionID + OptionSelfCloseRef

当自对冲报入CTP后，这组序号可以唯一定位一笔自对冲。可以用于跟踪自对冲回报的整个生命周期。

注意：CTP清流重启后自对冲中的该组序号会重新分配，即对于同一笔自对冲，清流重启前后的这组序号会不一致，使用上需注意。

◇ 5.2. ExchangeID + TraderID + OptionSelfCloseLocalID

当自对冲被CTP接受后，系统会分配OptionSelfCloseLocalID并发往交易所，这组序号可以用于跟踪此后自对冲的生命周期。

当自对冲被CTP拒绝后，不会分配OptionSelfCloseLocalID，此时仍要使用第1组序号跟踪自对冲。

注意：CTP清流重启后自对冲回报中的该组序号保持不变。

◇ 5.3. ExchangeID + OptionSelfCloseSysID

当自对冲被交易所接受后，交易所系统会分配OptionSelfCloseSysID并给CTP推送自对冲回报，这组序号可以用于跟踪此后自对冲的生命周期。

当自对冲被交易所拒绝后，交易所不会分配OptionSelfCloseSysID，此时仍要使用第1和第2组序号跟踪自对冲。

注意：CTP清流重启后自对冲回报中的该组序号保持不变。

大商所不适用此序号，因为大商所不分配OptionSelfCloseSysID值。

◇ 5.4. RequestID

[ReqOptionSelfCloseInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQOPTIONSELFCLOSEINSERT.html)中的RequestID与[OnRtnOptionSelfClose](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNOPTIONSELFCLOSE.html)中的相对应，用于跟踪自对冲回报的整个生命周期。

注意：CTP清流重启后该序号置零。
