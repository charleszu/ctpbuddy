# 做市商询价和报价

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

做市商询价和报价<a id="content"></a>

<a id="left_menu"></a>

  ** **<a id="ccf37c7e-0545-4b16-a2a5-06b5296f0cd5"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 指令介绍
<a id="panel1"></a>

- 1.请求询价指令

发起询价指令使用[ReqForQuoteInsert](pages/078-JYJK-CTHOSTFTDCTRADERAPI-REQFORQUOTEINSERT.html.md)函数。

参数CThostFtdcInputForQuoteField中，ForQuoteRef非必填，可由CTP交易核心自动生成。

交易所有询价价差的限制，期货公司可以在柜台上进行设置，一般如下

| 经纪公司代码 | 合约代码 | 交易所代码 | 最新价 | 价差 |
|---|---|---|---|---|
| 1008 | SRC | CZCE | 0 | 8 |
| 1008 | SRC | CZCE | 50 | 10 |
| 1008 | SRC | CZCE | 100 | 20 |
| 1008 | SRC | CZCE | 200 | 30 |
| 1008 | SRC | CZCE | 300 | 50 |
| 1008 | SRC | CZCE | 500 | 75 |

询价价差的判断过程

1)  看最新价对应于上面的哪一档次，确定价差的最小值

2)  计算买价和卖价的价差，看是否大于设置的价差（等于也不行）

3)  如果2)通过，那么询价单报入交易所，否则会被CTP直接拒绝。

如果询价的时候，当前合约价差不在范围，则报“CTP：当前价差禁止询价”。

另外，交易所还有询价时间间隔限制，一般在交易所端有控制，但郑商所控制在CTP柜台端，柜台设置例如：

对于郑商所、中金所，如果询价过于频繁，则会报“CTP：当前时间禁止询价”。其他交易所报错类似。

中金所询价开放了交易编码类型，支持“投机，套保，套利”的编码询价。

- 2.接收询价响应

询价响应有以下几种：

| 函数名称 | 说明 |
|---|---|
| OnRspForQuoteInsert | 只在录入错误的情况下返回，如字段校验失败、被限制询价等；可以通过ErrorID和ErrorMsg查看错误信息 |
| OnRtnForQuoteRsp（交易spi） | 暂时不用 |
| OnRtnForQuoteRsp（行情spi） | 录入成功后会推送，但是事先要调用行情api里的SubscribeForQuoteRsp去订阅该消息。 |
| OnErrRtnForQuoteInsert | 被交易所拒绝的询价，通过此接口返回；可以通过ErrorID和ErrorMsg查看错误信息 |

由于中金所、上期所和大商所的询价是来自交易所交易流，而郑商所是来自交易所行情流，所以连接mdfront只能看到郑商所的询价通知，连接行情front能收到所有交易所的询价通知。

- 3.查询询价

查询询价使用[ReqQryForQuote](pages/103-JYJK-CTHOSTFTDCTRADERAPI-REQQRYFORQUOTE.html.md)函数。

接收查询结果使用[OnRspQryForQuote](pages/254-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYFORQUOTE.html.md)函数

- 4.请求报价指令

请求报价指令使用[ReqQuoteInsert](pages/143-JYJK-CTHOSTFTDCTRADERAPI-REQQUOTEINSERT.html.md)函数。

上期所期货合约使用[ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)报价

参数CThostFtdcInputQuoteField中，QuoteRef、AskOrderRef、BidOrderRef非必填，如果手工填则要求AskOrderRef小于BidOrderRef，否则会报“CTP：重复的报单”。

- 5.接收报价响应

报价响应有以下几种：

| 函数名称 | 说明 |
|---|---|
| OnRspQuoteInsert | 只在录入错误的情况下返回，如字段校验失败、无做市商权限等；可以通过ErrorID和ErrorMsg查看错误信息 |
| OnRtnQuote | 当报价录入成功后，CTP给出该报价响应 |
| OnRtnOrder | 报价录入成功后，如果是双边报价，CTP还会同时衍生出买卖两笔报价衍生单报入交易所，并返回相应的OnRtnOrder |
| OnRtnTrade | 衍生单成交后，返回该成交回报 |

对于大商所，交易所在接受报价后，将不再维护报价状态的更新，所以客户端在接收CTP的报价回报时，其报价状态没有完整生命周期，因此采用CTP接口的开发人员需要重点关注报价对应衍生报单的状态。上期所会维护报价的状态，但是跟报单状态独立，比如，如果两腿报单都已经成交，依然可以撤报价单，并且能成功。中金所的报价和撤销报价的时候，交易所会发送[OnRtnQuote](pages/319-JYJK-CTHOSTFTDCTRADERSPI-ONRTNQUOTE.html.md)，会更新为已撤销。但是如果单独撤销两个衍生单，中金所不再更新报价的状态。

详见[报价回调规则](pages/391-QTYWGZ-DJHDGZ.html.md)

- 6.撤销报价请求

撤销报价请求使用[ReqQuoteAction](pages/142-JYJK-CTHOSTFTDCTRADERAPI-REQQUOTEACTION.html.md)函数。

参数CThostFtdcInputQuoteActionField中，使用QuoteRef+SessionID+FrontID组合来撤单。

- 7.撤销报价衍生单

撤销报价衍生单使用[ReqOrderAction](pages/085-JYJK-CTHOSTFTDCTRADERAPI-REQORDERACTION.html.md)函数。

参数CThostFtdcInputOrderActionField中，使用FrontID+SessionID+OrderRef组合来撤单。

要注意的是，如果在报价的时候没有填写AskOrderRef和BidOrderRef，那么报价衍生单的响应[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)里的FrontID和SessionID是0。

- 8.查询报价

查询报价使用[ReqQryQuote](pages/126-JYJK-CTHOSTFTDCTRADERAPI-REQQRYQUOTE.html.md)函数。

接收查询结果使用[OnRspQryQuote](pages/277-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYQUOTE.html.md)函数

<a id="c9e63622-07f6-407a-b5dd-dc9a585c3a2f"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 四所区别
<a id="panel2"></a>

下面列出四所在询价和报价上的一些区别。**注意：以下内容仅供参考，如果交易所的规则发生变化，文档并不会实时更新。**

- 1.四所询价对比

|  | 中金所 | 大商所 | 郑商所 | 上期所 |
|---|---|---|---|---|
| 询价间隔如何设置 | 交易所限制 | 交易所限制 | CTP柜台 | 交易所限制 |
| 普通投资者是否可接收询价申报 | 可以 | 可以 | 可以 | 可以 |

- 2.四所报价对比

<a id="anchor-id-01"></a>

|  | 中金所 | 大商所 | 郑商所 | 上期所 |
|---|---|---|---|---|
| 是否支持平仓 | 支持 | 支持 | 支持 | 支持 |
| 是否必须关联询价 | 询价通知没有询价编号，不用填 | 询价通知有编号，但是非必填，如做市商要完成义务率则要填 | 询价通知有编号，但是非必填，如做市商要完成义务率则要填 | 询价通知没有询价编号，不用填 |
| 是否支持单边报价 | 不支持，提示“CTP:报单字段有误” | 支持，数量填0 | 不支持，提示“CTP:报单字段有误” | 不支持，提示“CTP:报单字段有误” |
| 是否支持市价单 | 不支持，交易所提示：“已撤单报单被拒绝CFFEX:价格跌破跌停板” | 不支持，交易所提示：已撤单报单被拒绝DCE:期权价格必须大于等于TICK! | 不支持，交易所提示：“已撤单报单被拒绝CZCE:出错: 限价单价格不能为0或负数” | 不支持 |
| 双边报价买卖数量可否不一致 | 可以不一致 | 可以不一致 | CTP:不可以，提示“CTP：报单字段有误” | CTP:不可以，提示“CTP：报单字段有误” |
| 双边报价卖价是否必须大于买价 | 一定是低价买，高价卖。否则会形成自成交 | 必须，否则交易所提示： 已撤单报单被拒绝DCE:双边报价指令卖价必须大于买价 | 必须，否则交易所提示：已撤单报单被拒绝CZCE: 出错：卖价(40800)必需大于买价(94800) | 一定是低价买，高价卖。否则会形成自成交 |
| 是否支持套保属性 | 支持 | 支持 | 不支持，提示“CTP：该期权合约只支持投机类型报单” | 支持 |
| 是否会撤销前一次报价 | 后台版本在6.6.1以上后支持自动撤销 | 不会撤销 接入大商所七期之后支持自动撤销 | 会撤销 | 会撤销 |
| 是否支持撤单腿 | 支持顶单功能，不支持撤单边衍生单 | 可以 | 不可以 | 不可以 |
| 双边报价开平可否不一致 | 可以 | 可以 | 可以 | 可以 |

<a id="cd2e1ab5-461a-4b37-b74f-527f7af258b0"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. OrderType和TradeType字段说明
<a id="panel3"></a>

<a id="anchor-id-02"></a>

一笔报价会有两笔报价衍生单（[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)），报价成交后还会有成交回报（[OnRtnTrade](pages/326-JYJK-CTHOSTFTDCTRADERSPI-ONRTNTRADE.html.md)）。[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)的OrderType和[OnRtnTrade](pages/326-JYJK-CTHOSTFTDCTRADERSPI-ONRTNTRADE.html.md)的TradeType取自交易所回报。

上期所：交易所不返回TradeType和OrderType，所以在收到交易所回报后，两个字段都是空值。

大商所：报价会转换成两笔普通报单出去，所以TradeType和OrderType都是0（普通成交）

中金所：交易所返回OrderType为1（报价衍生单），TradeType为0（普通成交）

郑商所：交易所返回OrderType为1（报价衍生单），TradeType为0（普通成交）

<a id="author"></a>

<a id="theme_switcher"></a>
