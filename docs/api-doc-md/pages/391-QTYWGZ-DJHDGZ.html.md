# 报价回调规则

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

报价回调规则<a id="content"></a>

<a id="left_menu"></a>

  ** **

本文例举了一些常用的报价操作所对应的回调规则。在给定交易所报价回报顺序的前提下，CTP返回给API的回调顺序是固定的。

特别要注意，大商所只返回一次报价状态回报，例如一笔报价从报入到未成交到最终成交，一共有三个状态，但是大商所只推送未成交报价回报，全部成交后不推送成交报价回报。甚至在查询报价时，其报价状态也仍然是未成交状态。用户在测试和实现自己业务的时候要格外注意这点。

目前国内几家交易所的报价规则各不相同，本文不做详细例举，希望大家在梳理规则时学会举一反三。
<a id="464cf512-13bf-4505-a216-198610fc6798"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 术语说明
<a id="panel1"></a>

- 报价衍生单

报价报入交易所时，同时会衍生出两笔买卖报单，以普通报单的形式报入交易所。这两笔报单称之为报价衍生单。

- 报价回报

指报价的状态回报，有未知单报价回报、未成交报价回报、全部成交报价回报。对应回调函数[OnRtnQuote](pages/319-JYJK-CTHOSTFTDCTRADERSPI-ONRTNQUOTE.html.md)，以QuoteStatus字段区分。

- 报单回报

指报单的状态回报，有未知单报单回报、未成交报单回报、部分成交报单回报、全部成交报单回报和撤单回报。对应回调函数[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)，以OrderStatus字段区分。

- 成交回报

指报单成交后推送的成交回报，对应回调函数[OnRtnTrade](pages/326-JYJK-CTHOSTFTDCTRADERSPI-ONRTNTRADE.html.md)。

<a id="ecb1c6f0-ed5f-47b1-a4b5-55c275e2428b"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 回调规则
<a id="panel2"></a>

- 测试场景1

CZCE做市商报价，报价合约SR911P4100，开仓，买报价100，卖报价150，买数量1手，卖数量1手，先买成交1手，后又卖成交1手。

|  | 报价
ReqQuoteInsert
OnRtnOrder（未知单-卖）
OnRtnOrder（未知单-买）
OnRtnQuote（未知单）
此时CTP接收到交易所未成交报价回报和报单回报。
OnRtnQuote（未成交）
OnRtnOrder（未成交-买）
OnRtnOrder（未成交-卖）
此时CTP接收到交易所买方向全部成交的成交回报和报单回报。
OnRtnOrder（全部成交-买）
OnRtnTrade（买）
此时CTP接收到交易所卖方向的全部成交成交回报和报单回报，以及全部成交报价回报。
OnRtnQuote（全部成交）
OnRtnOrder（全部成交-卖）
OnRtnTrade（卖） |  |
|---|---|---|

注意：报价的衍生单（报价两条单腿报单）的回报顺序取决于交易所返回的衍生单的回报顺序。例如测试场景1中，交易所先推送买未成交报单回报后推送卖未成交回报，则CTP也会照此顺序推送。成交回报同理，因此对于报价及衍生单的买卖回报顺序是不固定的。

- 测试场景2

DCE做市商报价，报价合约m1911-C-2400，开仓，买报价400，卖报价500，买数量1手，卖数量1手，买成交1手，卖成交1手。

|  | 报价
ReqQuoteInsert
OnRtnOrder（未知单-卖）
OnRtnOrder（未知单-买）
OnRtnQuote（未知单）
此时CTP接收到交易所未成交报价回报和报单回报。
OnRtnQuote（未成交）
OnRtnOrder（未成交-卖）
OnRtnOrder（未成交-买）
此时CTP接收到交易所买方向全部成交的成交回报和报单回报。
OnRtnOrder（全部成交-卖）
OnRtnTrade（卖）
此时CTP接收到交易所卖方向的全部成交成交回报和报单回报。
OnRtnOrder（全部成交-买）
OnRtnTrade（买） |  |
|---|---|---|

注意：DCE区别于其他交易所，其报价状态只推送一次，即使后续状态发生变化，也不再推送。因此测试场景2相比场景1，会少一笔状态为全部成交的报价回报。

- 测试场景3

CZCE做市商报价撤单，报价合约CF001C12200，开仓，买报价2199，卖报价3369，买数量1手，卖数量1手，撤报价。

|  | 报价
ReqQuoteInsert
OnRtnOrder  （未知单—卖）
OnRtnOrder    （未知单—买）
OnRtnQuote  （未知单）
此时CTP接收到交易所未成交报价回报和报单回报。
OnRtnOrder    （未成交—买）
OnRtnOrder    （未成交—卖）
OnRtnQuote    （未成交）
此时撤报价
此时CTP接收到交易所已撤单报价回报和报单回报。
OnRtnQuote    （已撤单）
OnRtnOrder    （已撤单—卖）
OnRtnOrder    （已撤单—买） |  |
|---|---|---|

- 测试场景4

DCE做市商报价，撤单腿报价。报价合约c1909-C-1660，开仓，买报价179，卖报价371，买数量1手，卖数量1手；撤买单腿1手，撤卖单腿1手。

|  | 报价
ReqQuoteInsert
OnRtnOrder  （未知单—卖）
OnRtnOrder    （未知单—买）
OnRtnQuote  （未知单）
此时CTP接收到交易所未成交报价回报和报单回报。
OnRtnQuote    （未成交）
OnRtnOrder    （未成交—卖）
OnRtnOrder    （未成交—买）
此时撤单腿操作
OnRtnOrder    （未成交—买）
OnRtnOrder    （已撤单—买）
OnRtnOrder    （未成交—卖）
OnRtnOrder    （已撤单—卖） |  |
|---|---|---|

- 测试场景5

CZCE做市商报价，撤单腿报价，报价合约CF001C12200，开仓，买报价2199，卖报价3369，买数量1手，卖数量1手，撤买单腿1手，撤卖单腿1手。

|  | 报价
ReqQuoteInsert
OnRtnOrder  （未知单—卖）
OnRtnOrder    （未知单—买）
OnRtnQuote  （未知单）
此时CTP接收到交易所未成交报价回报和报单回报。
OnRtnQuote    （未成交）
OnRtnOrder    （未成交—买）
OnRtnOrder    （未成交—卖）
此时撤买单腿
此时CTP接收到交易所买撤单报单回报和错单回报。
OnRtnOrder    （未成交—买）
OnErrRtnOrderAction   [CZCE:出错: 所撤委托单没有找到]
此时撤卖单腿
此时CTP接收到交易所买撤单报单回报和错单回报。
OnRtnOrder    （未成交—卖）
OnErrRtnOrderAction   [CZCE:出错: 所撤委托单没有找到] |  |
|---|---|---|

郑商所和上期所报价只支持撤报价，不支持撤单腿，因此报错。

- 测试场景6

CZCE做市商报价。报价合约CF001C13000，开仓，买报价1700，卖报价2800，买数量1手，卖数量1手；再报价，报价合约仍是CF001C13000，开仓。

|  | 第一次报价
ReqQuoteInsert
OnRtnOrder  （未知单—卖）
OnRtnOrder    （未知单—买）
OnRtnQuote  （未知单）
此时CTP接收到交易所未成交报价回报和报单回报。
OnRtnQuote    （未成交）
OnRtnOrder    （未成交—买）
OnRtnOrder    （未成交—卖）
第二次报价（郑商所的第二次报价会自动撤销第一次报价）
ReqQuoteInsert
OnRtnOrder  （未知单—卖）
OnRtnOrder    （未知单—买）
OnRtnQuote  （未知单）
此时CTP接收到交易所如下回报：第一次报价的已撤单报价回报和报单回报，第二次报价的未成交报价回报和报单回报。
OnRtnQuote  （已撤销，第一次报价）
OnRtnOrder    （已撤单—买，第一次报价）
OnRtnOrder    （已撤单—卖，第一次报价）
OnRtnQuote    （未成交，第二次报价）
OnRtnOrder    （未成交—买，第二次报价）
OnRtnOrder    （未成交—卖，第二次报价） |  |
|---|---|---|

郑商所的第二次报价会自动撤销第一次报价。

- 测试场景7

DCE做市商报价，撤报价。报价合约m1707-C-2700，开仓，买报价63，卖报价349，买数量1手，卖数量1手；撤报价1手。

|  | 报价
ReqQuoteInsert
OnRtnOrder    （未知单—卖）
OnRtnOrder    （未知单—买）
OnRtnQuote    （未知单）
此时CTP接收到交易所未成交报价回报和报单回报。
OnRtnQuote    （未成交）
OnRtnOrder    （未成交—卖）
OnRtnOrder    （未成交—买）
此时撤报价操作
OnRtnOrder    （未成交—卖）
OnRtnOrder    （未成交—买）
OnRtnQuote    （未成交）
OnRtnOrder    （已撤单—卖）
OnRtnOrder    （已撤单—买） |  |
|---|---|---|

大商所报价后，交易所推送报价响应（未成交），此时撤报价，返回两腿的报单回报（已撤单），但是报价状态不会再推送

<a id="author"></a>

<a id="theme_switcher"></a>
