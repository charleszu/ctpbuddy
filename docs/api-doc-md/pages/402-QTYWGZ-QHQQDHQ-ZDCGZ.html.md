# 期货期权的行权、自对冲

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

期货期权的行权、自对冲<a id="content"></a>

<a id="left_menu"></a>

  ** **

本文旨在说明行权和自对冲在CTP上的实现，具体业务规则以交易所为准。
<a id="9b0a4416-56f0-408a-bb80-f6ce36c8a838"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 接口说明
<a id="panel1"></a>

- 行权申请指令

行权申请：[ReqExecOrderInsert](pages/077-JYJK-CTHOSTFTDCTRADERAPI-REQEXECORDERINSERT.html.md)

错误响应：[OnErrRtnExecOrderInsert](pages/208-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNEXECORDERINSERT.html.md)，[OnRspExecOrderInsert](pages/228-JYJK-CTHOSTFTDCTRADERSPI-ONRSPEXECORDERINSERT.html.md)

正确响应：[OnRtnExecOrder](pages/308-JYJK-CTHOSTFTDCTRADERSPI-ONRTNEXECORDER.html.md)

- 行权撤销指令

行权撤销：[ReqExecOrderAction](pages/076-JYJK-CTHOSTFTDCTRADERAPI-REQEXECORDERACTION.html.md)

错误响应：[OnErrRtnExecOrderAction](pages/207-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNEXECORDERACTION.html.md)，[OnRspExecOrderAction](pages/227-JYJK-CTHOSTFTDCTRADERSPI-ONRSPEXECORDERACTION.html.md)

正确响应：[OnRtnExecOrder](pages/308-JYJK-CTHOSTFTDCTRADERSPI-ONRTNEXECORDER.html.md)

- 期权自对冲指令

自对冲申请：[ReqOptionSelfCloseInsert](pages/084-JYJK-CTHOSTFTDCTRADERAPI-REQOPTIONSELFCLOSEINSERT.html.md)

错误响应：[OnErrRtnOptionSelfCloseInsert](pages/212-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNOPTIONSELFCLOSEINSERT.html.md)，[OnRspOptionSelfCloseInsert](pages/235-JYJK-CTHOSTFTDCTRADERSPI-ONRSPOPTIONSELFCLOSEINSERT.html.md)

正确响应：[OnRtnOptionSelfClose](pages/316-JYJK-CTHOSTFTDCTRADERSPI-ONRTNOPTIONSELFCLOSE.html.md)

- 期权自对冲撤销指令

自对冲撤销：[ReqOptionSelfCloseAction](pages/083-JYJK-CTHOSTFTDCTRADERAPI-REQOPTIONSELFCLOSEACTION.html.md)

错误响应: [OnErrRtnOptionSelfCloseAction](pages/211-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNOPTIONSELFCLOSEACTION.html.md)，[OnRspOptionSelfCloseAction](pages/234-JYJK-CTHOSTFTDCTRADERSPI-ONRSPOPTIONSELFCLOSEACTION.html.md)

正确响应: [OnRtnOptionSelfClose](pages/316-JYJK-CTHOSTFTDCTRADERSPI-ONRTNOPTIONSELFCLOSE.html.md)

<a id="4c025b7c-2887-462f-8e7b-d9ab059da335"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 行权执行指令
<a id="panel2"></a>

<a id="51d47bcf-7638-45c3-a284-001f79b133d1"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 2.1. 申请指令
<a id="panel3"></a>

行权指令包括**申请行权**和**放弃行权**。

美式期权的买方在合约到期日及其之前任一交易日均可行使权利，可以在到期日之前任一交易日的交易时间，以及到期日15:30之前提出行权申请；欧式期权的买方只可在合约到期日当天行使权利，到期日同最后交易日。欧式和美式如下表所示：

|  | CFFEX | CZCE | DCE | INE | SHFE |
|---|---|---|---|---|---|
| 美式 |  | 白糖期权棉花期权PTA期权甲醇期权菜籽粕期权 | 豆粕期权玉米期权铁矿石期权液化石油气期权 | 原油期权 | 橡胶期权铝期权锌期权 |
| 欧式 | 沪深300股指期权 |  |  |  | 铜期权黄金期权 |

如果非到期日申请，**CTP不做判断**，报入交易所后会返回报错：报单被拒绝，不在宣告期内。

行权会锁仓，并冻结资金。可行权的仓位根据多头持仓减去冻结持仓，同时计算可用资金满足支付行权权利金的仓位，二者取最小值作为可行权的实际仓位。行权冻结的仓体现在StrikeFrozen字段。

放弃行权会锁仓，但是不冻结资金。放弃行权冻结的仓体现在AbandonFrozen字段。

目前行权和放弃行权都使用[ReqExecOrderInsert](pages/077-JYJK-CTHOSTFTDCTRADERAPI-REQEXECORDERINSERT.html.md)接口，通过ActionType区分：行权申请THOST_FTDC_ACTP_Exec，放弃行权申请THOST_FTDC_ACTP_Abandon。

中金所目前不支持现货期权的相关行权指令，支持期货期权的行权指令。

针对上期所开平标志可以报入平今THOST_FTDC_OF_CloseToday，平昨THOST_FTDC_OF_CloseYesterday和平仓THOST_FTDC_OF_Close。其中平仓的逻辑跟平仓单的逻辑一样，如果客户报入平仓单，实际上CTP会处理为平昨仓。

<a id="598318f6-a4b2-40a0-9f62-526175359e3a"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 2.2. 自动行权、放弃行权和取消到期日自动行权
<a id="panel4"></a>

交易所会在期权合约到期日闭市后，默认自动将客户的实值期权进行行权，虚值期权放弃行权。如果客户不想执行某个实值期权，可在最后交易日申请放弃行权。

-

上期、能源、郑州支持放弃行权。

-

中金所对于现货期权不支持通过柜台系统（例如CTP）放弃行权，只支持通过会服系统进行操作。

-

大商所不支持放弃行权，只支持取消到期日自动行权。例如，投资者10手实值期权，若要放弃其中2手，则需要先取消到期日自动行权，然后再申报行权8手，以此达到目的。

大商所取消到期日自动行权方法为：使用接口[ReqExecOrderInsert](pages/077-JYJK-CTHOSTFTDCTRADERAPI-REQEXECORDERINSERT.html.md)，行权数量**Volume填0**。对数量不做校验，不冻钱不冻仓，只有到期日才能报入。

通过接口[ReqQryExecOrder](pages/102-JYJK-CTHOSTFTDCTRADERAPI-REQQRYEXECORDER.html.md)可以查询已报入交易系统的申请指令。

<a id="anchor-id-02"></a>
<a id="1defc61f-10a6-49cb-a7a1-987977d0ca9f"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 2.3. 撤销申请指令
<a id="panel5"></a>

对于已报入交易系统的行权申请或放弃申请指令，CTP提供接口[ReqExecOrderAction](pages/076-JYJK-CTHOSTFTDCTRADERAPI-REQEXECORDERACTION.html.md)撤销申请。

指令通过**ActionType**字段取值**THOST_FTDC_ACTP_Exec**、

**FrontID**、**SessionID**、**ExecOrderRef**、**InstrumentID**四个字段为一组条件撤销一笔申请（6.7.2开始支持撤销其他席位发起的“取消到期自动行权”指令或“期权自对冲”指令，由于接收其他席位的指令frontid+sessionid都是0，填0即可）。

<a id="c43b802d-b554-4d25-9bd9-2eac97080afa"></a><a id="title6"></a>

<a id="header_span6"></a>◇ 3. 自对冲
<a id="panel6"></a>

自对冲的业务含义是指向交易所提出申请将同一期权的多空头仓位，或者期权行权后或者履约后形成的期货的多空头仓位自我对冲掉。

目前郑商所和中金所的自对冲业务都是走会服系统，不通过交易通道。本栏目内容不包含郑商所和中金所

<a id="482ea55d-8a9c-41ad-bf86-9dbf95db4a46"></a><a id="title7"></a>

<a id="header_span7"></a>◇ 3.1. 行权后期货自对冲
<a id="panel7"></a>

期权买方（卖方）可以申请对其同一交易编码下行权后（履约后）双向期货持仓进行对冲平仓，对冲数量不超过行权获得的期货持仓量。对冲结果从当日期货持仓量中扣除，并计入成交量。

多头持仓的行权后期货自对冲仍采用行权接口[ReqExecOrderInsert](pages/077-JYJK-CTHOSTFTDCTRADERAPI-REQEXECORDERINSERT.html.md)，其中**CloseFlag**取值**THOST_FTDC_EOCF_AutoClose**来实现。

针对空头持仓的履约后期货自对冲提供接口[ReqOptionSelfCloseInsert](pages/084-JYJK-CTHOSTFTDCTRADERAPI-REQOPTIONSELFCLOSEINSERT.html.md)，通过**OptSelfCloseFlag**字段取值**THOST_FTDC_OSCF_SellCloseSelfFuturePosition**实现。
<a id="c60673ec-bb8b-4d44-8faa-e1a1cb10ca98"></a><a id="title8"></a>

<a id="header_span8"></a>◇ 3.2. 期权自对冲
<a id="panel8"></a>

客户可以申请对其同一交易编码下的双向期权持仓进行对冲平仓。对冲结果从当日期权持仓量中扣除，并计入成交量。

期权自对冲通过[ReqOptionSelfCloseInsert](pages/084-JYJK-CTHOSTFTDCTRADERAPI-REQOPTIONSELFCLOSEINSERT.html.md)接口，**OptSelfCloseFlag**字段取值**THOST_FTDC_OSCF_CloseSelfOptionPosition**申报。

该指令支持指定数量申请**Volume**，但是对仓位和资金都不做校验，比如某个投资者对期权合约并没有仓位，但是也可以申请期权自对冲。交易所盘后会去校验。投资者可以修改手数，目前CTP提供接口[ReqOptionSelfCloseAction](pages/083-JYJK-CTHOSTFTDCTRADERAPI-REQOPTIONSELFCLOSEACTION.html.md)，支持撤销自对冲后，重新申请。

上期所期权自对冲仅普通投资者可申请。针对同一客户，同合约和同开平标志(OffsetFlag)，只维护一条信息，后续报入的指令，都是更新操作。既期权自对冲本地报单编号相同。比如同一投资者对某一个铜期权申请期权自对冲3手，然后再报入该指令，申请期权自对冲5手，那么交易所查询的期权自对冲记录依然是一条，但是手数变为5。

大商所的期权自对冲普通投资者和做市商都可以申请，该设置是指定合约，不区分投机和套保，不指定数量，当日有效。
<a id="97d60f5b-f6d4-47a4-92cb-8032d509f82b"></a><a id="title9"></a>

<a id="header_span9"></a>◇ 3.3. 做市商留仓
<a id="panel9"></a>

交易所默认每天对做市商的期权买卖仓位自对冲（平仓），所以提供该选项给做市商，可以去申请留仓，当天不要被自对冲。

沿用[ReqOptionSelfCloseInsert](pages/084-JYJK-CTHOSTFTDCTRADERAPI-REQOPTIONSELFCLOSEINSERT.html.md)接口，**OptSelfCloseFlag**字段取值**THOST_FTDC_OSCF_CloseSelfOptionPosition**申报。

目前仅支持上期所做市商留仓申请

<a id="author"></a>

<a id="theme_switcher"></a>
