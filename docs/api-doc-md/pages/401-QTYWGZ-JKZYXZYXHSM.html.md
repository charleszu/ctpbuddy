# 接口中一些重要序号说明

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

接口中一些重要序号说明<a id="content"></a>

<a id="left_menu"></a>

  ** **<a id="2e2149ca-993d-437c-97c3-6502fd488451"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 报单
<a id="panel1"></a>

<a id="83e020e5-6ca3-44bc-938e-de9c8e5ac6d1"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 1.1. FrontID + SessionID + OrderRef
<a id="panel2"></a>

当报单报入CTP后，这组序号可以唯一定位一笔报单。可以用于跟踪报单回报的整个生命周期，但无法跟踪成交回报，因为成交回报里没有这组序号。

注意：CTP清流重启后报单中的该组序号会重新分配，即对于同一笔报单，清流重启前后的这组序号会不一致，使用上需注意。
<a id="35b61b55-2f06-4433-b375-4be514b6ad6f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 1.2. ExchangeID + TraderID + OrderLocalID
<a id="panel3"></a>

当报单被CTP接受后，系统会分配OrderLocalID并发往交易所，这组序号可以用于跟踪此后报单的生命周期，包括报单回报和成交回报。

当报单被CTP拒绝后，不会分配OrderLocalID，此时仍要使用第1组序号跟踪报单。

注意：CTP清流重启后报单回报中的该组序号保持不变。
<a id="4f607bfe-593e-43af-a64b-18ab03341a11"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 1.3. ExchangeID + OrderSysID
<a id="panel4"></a>

当报单被交易所接受后，交易所系统会分配OrderSysID并给CTP推送报单回报，这组序号可以用于跟踪此后报单的生命周期，包括报单回报和成交回报。

当报单被交易所拒绝后，交易所不会分配OrderSysID，此时仍要使用第1和第2组序号跟踪报单。

注意：CTP清流重启后报单回报中的该组序号保持不变。
<a id="86c91584-3cf6-4bd7-8e24-66e11fce4a6e"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 1.4. RequestID
<a id="panel5"></a>

[ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)中的RequestID与[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)中的相对应，可以用于跟踪报单回报的整个生命周期，但无法跟踪成交回报，因为成交回报里没有这个字段。

注意：CTP清流重启后该序号置零。
<a id="dca740ac-8237-4f98-9533-ebede808f7e0"></a><a id="title6"></a>

<a id="header_span6"></a>◇ 1.5. RelativeOrderSysID
<a id="panel6"></a>

用于关联条件报单和触发后的报单回报。当报入条件单后，会生成OrderSysID=TJBD_XX的报单，此时状态为未触发；当行情满足条件并触发条件单后，会生成一笔新的报单，其RelativeOrderSysID=TJBD_XX。

注意：CTP清流重启后RelativeOrderSysID会清空，无法再关联条件单。

<a id="d5c40b18-d0b2-4d8f-be1b-6d18a819efed"></a><a id="title7"></a>

<a id="header_span7"></a>◇ 2. 报价
<a id="panel7"></a>

<a id="54ed4b9a-dd91-458b-8ba7-88f4835283f0"></a><a id="title8"></a>

<a id="header_span8"></a>◇ 2.1. FrontID + SessionID + QuoteRef
<a id="panel8"></a>

当报价报入CTP后，这组序号可以唯一定位一笔报价。可以用于跟踪报价回报的整个生命周期，但无法跟踪报价衍生单，因为报价衍生单回报（[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)和[OnRtnTrade](pages/326-JYJK-CTHOSTFTDCTRADERSPI-ONRTNTRADE.html.md)）里没有这组序号。

注意：CTP清流重启后报价中的该组序号会重新分配，即对于同一笔报价，清流重启前后的这组序号会不一致，使用上需注意。
<a id="f752ae0f-427f-46bb-a0bd-765c59c7190b"></a><a id="title9"></a>

<a id="header_span9"></a>◇ 2.2. FrontID + SessionID + AskOrderRef/BidOrderRef
<a id="panel9"></a>

当报价报入CTP后，可以通过报价回报中的这组序号跟踪报价衍生单，AskOrderRef对应报价衍生卖单的OrderRef，BidOrderRef对应报价衍生买单的OrderRef。规则同上面报单接口的序号1。

注意：CTP清流重启后报单中的该组序号会重新分配。
<a id="6a8f0fa6-6f7f-46ff-8f1a-1c974cf59b16"></a><a id="title10"></a>

<a id="header_span10"></a>◇ 2.3. ExchangeID + TraderID + QuoteLocalID
<a id="panel10"></a>

当报价被CTP接受后，系统会分配QuoteLocalID并发往交易所，这组序号可以用于跟踪此后报价的生命周期，但无法跟踪报价衍生单，因为报价衍生单回报里没有这组序号。衍生单的序号规则同上面报价接口序号2。

当报价被CTP拒绝后，不会分配QuoteLocalID，此时仍要使用第1组序号跟踪报价。

注意：CTP清流重启后报价回报中的该组序号保持不变。
<a id="e784fd90-9838-4906-96cd-2e0285571373"></a><a id="title11"></a>

<a id="header_span11"></a>◇ 2.4. ExchangeID + QuoteSysID
<a id="panel11"></a>

当报价被交易所接受后，交易所系统会分配QuoteSysID并给CTP推送报价回报，这组序号可以用于跟踪此后报价的生命周期，但无法跟踪报价衍生单，因为报价衍生单回报里没有这组序号。

当报价被交易所拒绝后，交易所不会分配QuoteSysID，此时仍要使用第1和第3组序号跟踪报价。

注意：CTP清流重启后报价回报中的该组序号保持不变。
<a id="14119c9e-c6a1-4c9a-8c48-319cb799fe63"></a><a id="title12"></a>

<a id="header_span12"></a>◇ 2.5. ExchangeID + AskOrderSysID/BidOrderSysID
<a id="panel12"></a>

当报价被交易所接受后，上期所、能源中心、中金所可以通过报价回报中的这组序号跟踪报价衍生单，AskOrderSysID对应报价衍生卖单的OrderSysID，BidOrderSysID对应报价衍生买单的OrderSysID。衍生单的序号规则同上面报单接口。

大商所和郑商所的报价回报中，AskOrderSysID和BidOrderSysID为空。

大商所报价回报中QuoteSysID对应买衍生单的的OrderSysID，QuoteSysID+1的值对应卖衍生单的OrderSysID，比如报价回报中QuoteSysID为110，那么衍生单的OrderSysID分别是110和111。

郑商所报价回报中QuoteSysID的前缀加上'a'对应卖衍生单的OrderSysID，QuoteSysID的前缀加上'b'对应买衍生单的OrderSysID，比如报价回报的QuoteSysID为110，那么衍生单的OrderSysID分别是a110、b110。

注意：CTP清流重启后报价回报中的该组序号保持不变。
<a id="ed8bd70a-39f1-4f16-bb69-109a3584d03b"></a><a id="title13"></a>

<a id="header_span13"></a>◇ 2.6. RequestID
<a id="panel13"></a>

[ReqQuoteInsert](pages/143-JYJK-CTHOSTFTDCTRADERAPI-REQQUOTEINSERT.html.md)中的RequestID与[OnRtnQuote](pages/319-JYJK-CTHOSTFTDCTRADERSPI-ONRTNQUOTE.html.md)和[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)中的相对应，用于跟踪报价回报和报价衍生单回报的整个生命周期。例如[ReqQuoteInsert](pages/143-JYJK-CTHOSTFTDCTRADERAPI-REQQUOTEINSERT.html.md)中RequestID=888，则[OnRtnQuote](pages/319-JYJK-CTHOSTFTDCTRADERSPI-ONRTNQUOTE.html.md)的RequestID=888，买衍生单的[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)中的RequestID=888，卖衍生单[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)中的RequestID=888。

注意：CTP清流重启后该序号置零。

<a id="7455770f-0e2f-4c95-8384-ad74113684eb"></a><a id="title14"></a>

<a id="header_span14"></a>◇ 3. 询价
<a id="panel14"></a>

<a id="7c0d4099-323c-4c48-bf44-95d37e07fb4c"></a><a id="title15"></a>

<a id="header_span15"></a>◇ 3.1. ForQuoteRef
<a id="panel15"></a>

当询价请求报入CTP后，该字段用来唯一定位一笔询价。

注意：CTP清流重启后报单中的该组序号会重新分配。
<a id="ca147154-d078-405e-ae00-4cc773d3e75e"></a><a id="title16"></a>

<a id="header_span16"></a>◇ 3.2. ForQuoteSysID
<a id="panel16"></a>

当询价被交易所接受后，交易所系统分配ForQuoteSysID，可以用于跟踪此后询价的的生命周期。

注意：CTP清流重启后报价回报中的该组序号保持不变。

<a id="f4d961ee-942c-4ba2-a1ee-b91100763674"></a><a id="title17"></a>

<a id="header_span17"></a>◇ 4. 行权
<a id="panel17"></a>

<a id="5a2cd136-badf-4830-b9ec-8b357b80b953"></a><a id="title18"></a>

<a id="header_span18"></a>◇ 4.1. FrontID + SessionID + ExecOrderRef
<a id="panel18"></a>

当行权请求报入CTP后，该组序号用来唯一定位一笔行权。

注意：CTP清流重启后报单中的该组序号会重新分配。
<a id="bd5fb171-a7f2-4437-bc92-a59c66cfedba"></a><a id="title19"></a>

<a id="header_span19"></a>◇ 4.2. ExchangeID + TraderID + ExecOrderLocalID
<a id="panel19"></a>

当行权被CTP接受后，可以通过行权回报中的这组序号跟踪此后行权的生命周期。

注意：CTP清流重启后报价回报中的该组序号保持不变。
<a id="dd2ec7c4-433a-4a1d-bf67-65e957351616"></a><a id="title20"></a>

<a id="header_span20"></a>◇ 4.3. ExchangeID + ExecOrderSysID
<a id="panel20"></a>

当行权被交易所接受后，可以通过行权回报中的这组序号跟踪此后行权的生命周期。

注意：CTP清流重启后报价回报中的该组序号保持不变。

<a id="84b7bf7e-5e18-4717-a416-67d9fdc7fe3f"></a><a id="title21"></a>

<a id="header_span21"></a>◇ 5. 自对冲
<a id="panel21"></a>

<a id="06d165b6-7f90-451e-9e36-75edbcb5285c"></a><a id="title22"></a>

<a id="header_span22"></a>◇ 5.1. FrontID + SessionID + OptionSelfCloseRef
<a id="panel22"></a>

当自对冲报入CTP后，这组序号可以唯一定位一笔自对冲。可以用于跟踪自对冲回报的整个生命周期。

注意：CTP清流重启后自对冲中的该组序号会重新分配，即对于同一笔自对冲，清流重启前后的这组序号会不一致，使用上需注意。
<a id="2b852216-84c9-4cb1-8c7a-b891e945860d"></a><a id="title23"></a>

<a id="header_span23"></a>◇ 5.2. ExchangeID + TraderID + OptionSelfCloseLocalID
<a id="panel23"></a>

当自对冲被CTP接受后，系统会分配OptionSelfCloseLocalID并发往交易所，这组序号可以用于跟踪此后自对冲的生命周期。

当自对冲被CTP拒绝后，不会分配OptionSelfCloseLocalID，此时仍要使用第1组序号跟踪自对冲。

注意：CTP清流重启后自对冲回报中的该组序号保持不变。
<a id="697ec1a0-de6a-4b00-999a-bcdab0ee1b55"></a><a id="title24"></a>

<a id="header_span24"></a>◇ 5.3. ExchangeID + OptionSelfCloseSysID
<a id="panel24"></a>

当自对冲被交易所接受后，交易所系统会分配OptionSelfCloseSysID并给CTP推送自对冲回报，这组序号可以用于跟踪此后自对冲的生命周期。

当自对冲被交易所拒绝后，交易所不会分配OptionSelfCloseSysID，此时仍要使用第1和第2组序号跟踪自对冲。

注意：CTP清流重启后自对冲回报中的该组序号保持不变。

大商所不适用此序号，因为大商所不分配OptionSelfCloseSysID值。

<a id="e5e66380-014c-4b4f-9b18-18a92ead570b"></a><a id="title25"></a>

<a id="header_span25"></a>◇ 5.4. RequestID
<a id="panel25"></a>

[ReqOptionSelfCloseInsert](pages/084-JYJK-CTHOSTFTDCTRADERAPI-REQOPTIONSELFCLOSEINSERT.html.md)中的RequestID与[OnRtnOptionSelfClose](pages/316-JYJK-CTHOSTFTDCTRADERSPI-ONRTNOPTIONSELFCLOSE.html.md)中的相对应，用于跟踪自对冲回报的整个生命周期。

注意：CTP清流重启后该序号置零。

<a id="author"></a>

<a id="theme_switcher"></a>
