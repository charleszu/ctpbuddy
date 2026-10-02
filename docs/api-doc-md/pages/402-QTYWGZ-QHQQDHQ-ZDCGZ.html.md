# 期货期权的行权、自对冲

期货期权的行权、自对冲

本文旨在说明行权和自对冲在CTP上的实现，具体业务规则以交易所为准。

◇ 1. 接口说明

- 行权申请指令

行权申请：[ReqExecOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQEXECORDERINSERT.html)

错误响应：[OnErrRtnExecOrderInsert](../JYJK/CTHOSTFTDCTRADERSPI/ONERRRTNEXECORDERINSERT.html)，[OnRspExecOrderInsert](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPEXECORDERINSERT.html)

正确响应：[OnRtnExecOrder](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNEXECORDER.html)

- 行权撤销指令

行权撤销：[ReqExecOrderAction](../JYJK/CTHOSTFTDCTRADERAPI/REQEXECORDERACTION.html)

错误响应：[OnErrRtnExecOrderAction](../JYJK/CTHOSTFTDCTRADERSPI/ONERRRTNEXECORDERACTION.html)，[OnRspExecOrderAction](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPEXECORDERACTION.html)

正确响应：[OnRtnExecOrder](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNEXECORDER.html)

- 期权自对冲指令

自对冲申请：[ReqOptionSelfCloseInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQOPTIONSELFCLOSEINSERT.html)

错误响应：[OnErrRtnOptionSelfCloseInsert](../JYJK/CTHOSTFTDCTRADERSPI/ONERRRTNOPTIONSELFCLOSEINSERT.html)，[OnRspOptionSelfCloseInsert](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPOPTIONSELFCLOSEINSERT.html)

正确响应：[OnRtnOptionSelfClose](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNOPTIONSELFCLOSE.html)

- 期权自对冲撤销指令

自对冲撤销：[ReqOptionSelfCloseAction](../JYJK/CTHOSTFTDCTRADERAPI/REQOPTIONSELFCLOSEACTION.html)

错误响应: [OnErrRtnOptionSelfCloseAction](../JYJK/CTHOSTFTDCTRADERSPI/ONERRRTNOPTIONSELFCLOSEACTION.html)，[OnRspOptionSelfCloseAction](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPOPTIONSELFCLOSEACTION.html)

正确响应: [OnRtnOptionSelfClose](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNOPTIONSELFCLOSE.html)

◇ 2. 行权执行指令

◇ 2.1. 申请指令

行权指令包括**申请行权**和**放弃行权**。

美式期权的买方在合约到期日及其之前任一交易日均可行使权利，可以在到期日之前任一交易日的交易时间，以及到期日15:30之前提出行权申请；欧式期权的买方只可在合约到期日当天行使权利，到期日同最后交易日。欧式和美式如下表所示：

|  | CFFEX | CZCE | DCE | INE | SHFE |
|---|---|---|---|---|---|
| 美式 |  | 白糖期权棉花期权PTA期权甲醇期权菜籽粕期权 | 豆粕期权玉米期权铁矿石期权液化石油气期权 | 原油期权 | 橡胶期权铝期权锌期权 |
| 欧式 | 沪深300股指期权 |  |  |  | 铜期权黄金期权 |

如果非到期日申请，**CTP不做判断**，报入交易所后会返回报错：报单被拒绝，不在宣告期内。

行权会锁仓，并冻结资金。可行权的仓位根据多头持仓减去冻结持仓，同时计算可用资金满足支付行权权利金的仓位，二者取最小值作为可行权的实际仓位。行权冻结的仓体现在StrikeFrozen字段。

放弃行权会锁仓，但是不冻结资金。放弃行权冻结的仓体现在AbandonFrozen字段。

目前行权和放弃行权都使用[ReqExecOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQEXECORDERINSERT.html)接口，通过ActionType区分：行权申请THOST_FTDC_ACTP_Exec，放弃行权申请THOST_FTDC_ACTP_Abandon。

中金所目前不支持现货期权的相关行权指令，支持期货期权的行权指令。

针对上期所开平标志可以报入平今THOST_FTDC_OF_CloseToday，平昨THOST_FTDC_OF_CloseYesterday和平仓THOST_FTDC_OF_Close。其中平仓的逻辑跟平仓单的逻辑一样，如果客户报入平仓单，实际上CTP会处理为平昨仓。

◇ 2.2. 自动行权、放弃行权和取消到期日自动行权

交易所会在期权合约到期日闭市后，默认自动将客户的实值期权进行行权，虚值期权放弃行权。如果客户不想执行某个实值期权，可在最后交易日申请放弃行权。

-

上期、能源、郑州支持放弃行权。

-

中金所对于现货期权不支持通过柜台系统（例如CTP）放弃行权，只支持通过会服系统进行操作。

-

大商所不支持放弃行权，只支持取消到期日自动行权。例如，投资者10手实值期权，若要放弃其中2手，则需要先取消到期日自动行权，然后再申报行权8手，以此达到目的。

大商所取消到期日自动行权方法为：使用接口[ReqExecOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQEXECORDERINSERT.html)，行权数量**Volume填0**。对数量不做校验，不冻钱不冻仓，只有到期日才能报入。

通过接口[ReqQryExecOrder](../JYJK/CTHOSTFTDCTRADERAPI/REQQRYEXECORDER.html)可以查询已报入交易系统的申请指令。

◇ 2.3. 撤销申请指令

对于已报入交易系统的行权申请或放弃申请指令，CTP提供接口[ReqExecOrderAction](../JYJK/CTHOSTFTDCTRADERAPI/REQEXECORDERACTION.html)撤销申请。

指令通过**ActionType**字段取值**THOST_FTDC_ACTP_Exec**、

**FrontID**、**SessionID**、**ExecOrderRef**、**InstrumentID**四个字段为一组条件撤销一笔申请（6.7.2开始支持撤销其他席位发起的“取消到期自动行权”指令或“期权自对冲”指令，由于接收其他席位的指令frontid+sessionid都是0，填0即可）。

◇ 3. 自对冲

自对冲的业务含义是指向交易所提出申请将同一期权的多空头仓位，或者期权行权后或者履约后形成的期货的多空头仓位自我对冲掉。

目前郑商所和中金所的自对冲业务都是走会服系统，不通过交易通道。本栏目内容不包含郑商所和中金所

◇ 3.1. 行权后期货自对冲

期权买方（卖方）可以申请对其同一交易编码下行权后（履约后）双向期货持仓进行对冲平仓，对冲数量不超过行权获得的期货持仓量。对冲结果从当日期货持仓量中扣除，并计入成交量。

多头持仓的行权后期货自对冲仍采用行权接口[ReqExecOrderInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQEXECORDERINSERT.html)，其中**CloseFlag**取值**THOST_FTDC_EOCF_AutoClose**来实现。

针对空头持仓的履约后期货自对冲提供接口[ReqOptionSelfCloseInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQOPTIONSELFCLOSEINSERT.html)，通过**OptSelfCloseFlag**字段取值**THOST_FTDC_OSCF_SellCloseSelfFuturePosition**实现。

◇ 3.2. 期权自对冲

客户可以申请对其同一交易编码下的双向期权持仓进行对冲平仓。对冲结果从当日期权持仓量中扣除，并计入成交量。

期权自对冲通过[ReqOptionSelfCloseInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQOPTIONSELFCLOSEINSERT.html)接口，**OptSelfCloseFlag**字段取值**THOST_FTDC_OSCF_CloseSelfOptionPosition**申报。

该指令支持指定数量申请**Volume**，但是对仓位和资金都不做校验，比如某个投资者对期权合约并没有仓位，但是也可以申请期权自对冲。交易所盘后会去校验。投资者可以修改手数，目前CTP提供接口[ReqOptionSelfCloseAction](../JYJK/CTHOSTFTDCTRADERAPI/REQOPTIONSELFCLOSEACTION.html)，支持撤销自对冲后，重新申请。

上期所期权自对冲仅普通投资者可申请。针对同一客户，同合约和同开平标志(OffsetFlag)，只维护一条信息，后续报入的指令，都是更新操作。既期权自对冲本地报单编号相同。比如同一投资者对某一个铜期权申请期权自对冲3手，然后再报入该指令，申请期权自对冲5手，那么交易所查询的期权自对冲记录依然是一条，但是手数变为5。

大商所的期权自对冲普通投资者和做市商都可以申请，该设置是指定合约，不区分投机和套保，不指定数量，当日有效。

◇ 3.3. 做市商留仓

交易所默认每天对做市商的期权买卖仓位自对冲（平仓），所以提供该选项给做市商，可以去申请留仓，当天不要被自对冲。

沿用[ReqOptionSelfCloseInsert](../JYJK/CTHOSTFTDCTRADERAPI/REQOPTIONSELFCLOSEINSERT.html)接口，**OptSelfCloseFlag**字段取值**THOST_FTDC_OSCF_CloseSelfOptionPosition**申报。

目前仅支持上期所做市商留仓申请
