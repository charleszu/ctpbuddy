# 报单回调规则

报单回调规则

本文例举了一些常用的报单操作所对应的回调规则。在给定交易所报单回报顺序的前提下，CTP返回给API的回调顺序是固定的。

目前大商所的回报比较特殊，当一笔委托全部成交后，大商所直接返回该笔委托的成交回报，而不返回状态为全部成交的报单回报，对于这种情况，CTP会补一个状态为全部成交的报单回报，下文会举例说明。

◇ 1. 术语说明

- 报单回报

指报单的状态回报，有未知单报单回报、未成交报单回报、部分成交报单回报、全部成交报单回报和撤单回报。对应回调函数[OnRtnOrder](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNORDER.html)，以OrderStatus字段区分。

- 成交回报

指报单成交后推送的成交回报，对应回调函数[OnRtnTrade](../JYJK/CTHOSTFTDCTRADERSPI/ONRTNTRADE.html)。

- 错单响应

指报单被CTP拒绝后返回的响应通知，对应回调函数[OnRspOrderInsert](../JYJK/CTHOSTFTDCTRADERSPI/ONRSPORDERINSERT.html)，其中ErrorID和ErrorMsg指明了错误原因。

- 错单回报

指报单被CTP或交易所拒绝后的报单的状态回报。对应回调函数[OnErrRtnOrderInsert](../JYJK/CTHOSTFTDCTRADERSPI/ONERRRTNORDERINSERT.html)。

◇ 2. 回调规则

- 测试场景1

报单合约ag1207，报入1手，到交易所先未成交，后全部成交1手。

|  | 报单
ReqOrderInsert
回调顺序：
OnRtnOrder （未知单）
此时CTP接收到交易所的未成交报单回报
OnRtnOrder （未成交）
此时CTP接收到交易所的成交回报和全部成交报单回报
OnRtnOrder （未成交）
OnRtnOrder （全部成交）
OnRtnTrade |  |
|---|---|---|

- 测试场景2

报单合约ag1207，报入1手，到交易所立即全部成交1手。（注意，此案例没有收到交易所的未成交回报状态，而是直接收到全部成交状态）

|  | 报单
ReqOrderInsert
OnRtnOrder （未知单）
此时CTP接收到交易所的成交回报和成交报单回报
OnRtnOrder （未知单）
OnRtnOrder （全部成交报单已提交）
OnRtnTrade |  |
|---|---|---|

- 测试场景3

报单合约ag1207，报入1手，到交易所先未成交，然后撤单1手。

|  | 报单
ReqOrderInsert
OnRtnOrder（未知单）
此时CTP接收到交易所的未成交报单回报
OnRtnOrder（未成交）
ReqOrderAction
此时CTP接收到交易所的撤单报单回报
OnRtnOrder（未成交）
OnRtnOrder（已撤单） |  |
|---|---|---|

- 测试场景4

报单合约ag1207，报入10手，到交易所先未成交，然后成交3手，然后剩余全部成交7手。

|  | 报单
ReqOrderInsert
OnRtnOrder （未知单）
此时CTP接收到交易所的未成交报单回报
OnRtnOrder （未成交）
此时CTP接收到交易所的成交回报和部分成交报单回报
OnRtnOrder （未成交）
OnRtnOrder （部分成交）
OnRtnTrade
此时CTP接收到交易所的成交回报和全部成交报单回报
OnRtnOrder （部分成交）
OnRtnOrder （全部成交）
OnRtnTrade |  |
|---|---|---|

- 测试场景5

报单合约ag1207，报入10手，到交易所先未成交，然后成交4手，然后剩余全部撤单6手。

|  | 报单
ReqOrderInsert
OnRtnOrder （未知单）
此时CTP接收到交易所的未成交报单回报
OnRtnOrder （未成交）
此时CTP接收到交易所的成交回报和部分成交报单回报
OnRtnOrder （未成交）
OnRtnOrder （部分成交）
OnRtnTrade
发起撤单
ReqOrderAction
此时CTP接收到交易所的撤单回报
OnRtnOrder（部分成交）
OnRtnOrder（已撤单） |  |
|---|---|---|

- 测试场景6

对一笔已经撤单的委托单再次撤单。

|  | 对一笔已撤单再次撤单
ReqOrderAction
触发撤单错误响应
OnRspOrderAction （CTP:报单已全成交或已撤销，不能再撤）
触发撤单错误回报
OnErrRtnOrderAction （CTP:报单已全成交或已撤销，不能再撤） |  |
|---|---|---|

- 测试场景7

对一笔未成交委托进行撤单，因为委托号填写错误导致撤单失败

|  | 撤一笔不存在的委托
ReqOrderAction
触发撤单错误响应（CTP:撤单找不到相应报单）
OnRspOrderAction
触发撤单错误回报
OnErrRtnOrderAction （CTP:撤单找不到相应报单） |  |
|---|---|---|

- 测试场景8

报入上期所FAK单子，报入后部分成交部分撤单。

|  | 报入上期所FAK
ReqOrderInsert
OnRtnOrder （未知单）
此时CTP收到了交易所推送的撤单状态的报单回报
OnRtnOrder （已撤单，回报中VolumeTraded字段此时已有值）
此时CTP收到了交易所推送的成交回报
OnRtnOrder （已撤单）
OnRtnTrade |  |
|---|---|---|

相同场景下，能源中心和中金所FAK的回报与上期所一致。

- 测试场景9

报入大商所FAK单子，报入后部分成交部分撤单

|  | 报入大商所FAK
ReqOrderInsert
OnRtnOrder （未知单）
此时CTP收到了交易所推送的未成交状态的报单回报
OnRtnOrder （未成交）
此时CTP收到了交易所推送的成交回报
OnRtnOrder （部分成交）
OnRtnTrade
此时CTP收到了交易所推送的撤单状态的报单回报
OnRtnOrder （已撤单） |  |
|---|---|---|

相同场景下，广期所FAK的回报与大商所一致。

- 测试场景10

报入郑商所FAK单子，报入后部分成交部分撤单

|  | 报入郑商所FAK
ReqOrderInsert
OnRtnOrder （未知单）
此时CTP收到了交易所推送的未成交状态的报单回报
OnRtnOrder （未成交）
此时CTP收到了交易所推送的部分成交状态的报单回报
OnRtnOrder （未成交）
OnRtnOrder （部分成交）
OnRtnTrade
此时CTP收到了交易所推送的撤单状态的报单回报
OnRtnOrder （已撤单） |  |
|---|---|---|

◇ 3. 大商所特殊回调规则

目前大商所的回报比较特殊，当一笔委托全部成交后，大商所直接返回该笔委托的成交回报，而不返回状态为全部成交的报单回报，对于这种情况，CTP会补一个状态为全部成交的报单回报。

- 测试场景1

报单合约c1911-C-1760，报入1手，到交易所先未成交，后全部成交1手。

|  | 报单
ReqOrderInsert
OnRtnOrder （未知单）
此时CTP接收到交易所的未成交报单回报
OnRtnOrder （未成交）
此时CTP接收到交易所的成交回报
OnRtnOrder （全部成交）（CTP补充生成，且不重复推送前一状态）
OnRtnTrade |  |
|---|---|---|

- 测试场景2

报单合约c1911-C-1760，报入1手，到交易所立即全部成交1手。注意，大商所不管会不会立即成交，只要委托进入报单簿后都会返回一笔未成交报单回报给到CTP。

|  | 报单
ReqOrderInsert
OnRtnOrder （未知单）
此时CTP接收到交易所的未成交报单回报
OnRtnOrder （未成交）
此时CTP接收到交易所的成交回报
OnRtnOrder （全部成交）（CTP补充生成，且不重复推送前一状态）
OnRtnTrade |  |
|---|---|---|

- 测试场景3

报单合约c1911-C-1760，报入1手，到交易所先未成交，然后撤单1手。

|  | 报单
ReqOrderInsert
OnRtnOrder （未知单）
此时CTP接收到交易所的未成交报单回报
OnRtnOrder （未成交）
发起撤单
ReqOrderAction
此时CTP接收到交易所的撤单回报
OnRtnOrder （未成交）
OnRtnOrder （已撤单） |  |
|---|---|---|
