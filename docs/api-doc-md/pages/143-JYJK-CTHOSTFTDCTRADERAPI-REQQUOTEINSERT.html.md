# ReqQuoteInsert

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQuoteInsert<a id="content"></a>

<a id="left_menu"></a>

  ** **

报价录入请求，如果出错，则返回响应[OnRspQuoteInsert](pages/293-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQUOTEINSERT.html.md)和[OnErrRtnQuoteInsert](pages/217-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNQUOTEINSERT.html.md)；正确则推送[OnRtnQuote](pages/319-JYJK-CTHOSTFTDCTRADERSPI-ONRTNQUOTE.html.md)、[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)和[OnRtnTrade](pages/326-JYJK-CTHOSTFTDCTRADERSPI-ONRTNTRADE.html.md)。

单边报价和双边报价，都是用一个接口 ReqQuoteInsert。

在单边报价的时候，只需要另一边的数量填0，交易核心就能区分开。另外，无论是单边还是双边，Ask/BidOrderRef都是要填的。

除上期所的期货合约使用[ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)接口报价，其他交易所均使用本接口报价。

详见[做市商询价和报价](pages/388-QTYWGZ-BJHXJ.html.md)

回调规则详见[报价回调规则](pages/391-QTYWGZ-DJHDGZ.html.md)

关于接口中的重要序号说明详见[接口中一些重要序号说明](pages/401-QTYWGZ-JKZYXZYXHSM.html.md)
<a id="6f292b8b-a627-439a-b3f0-ab82a514f9bf"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQuoteInsert(CThostFtdcInputQuoteField *pInputQuote, int nRequestID) = 0;

<a id="906d3985-6f41-4dbf-9ff5-308cb17860aa"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputQuote：输入的报价

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填 |
| TThostFtdcOrderRefType | QuoteRef | 报价引用 | 无 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcBusinessUnitType | BusinessUnit | 业务单元 | 无 |
| TThostFtdcOrderRefType | AskOrderRef | 衍生卖报单引用 | 选填 |
| TThostFtdcOrderRefType | BidOrderRef | 衍生买报单引用 | 选填 |
| TThostFtdcOrderSysIDType | ForQuoteSysID | 应价编号 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 无 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcClientIDType | ClientID | 客户代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcVolumeType | AskVolume | 卖数量 | 必填 |
| TThostFtdcVolumeType | BidVolume | 买数量 | 必填 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcOffsetFlagType | AskOffsetFlag | 卖开平标志 | 必填 |
| TThostFtdcOffsetFlagType | BidOffsetFlag | 买开平标志 | 必填 |
| TThostFtdcHedgeFlagType | AskHedgeFlag | 卖投机套保标志 | 投机 |
| TThostFtdcHedgeFlagType | BidHedgeFlag | 买投机套保标志 | 投机 |
| TThostFtdcPriceType | AskPrice | 卖价格 | 必填 |
| TThostFtdcPriceType | BidPrice | 买价格 | 必填 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 否 |
| TThostFtdcOrderSysIDType | ReplaceSysID | 被顶单编号 | 否 |
| TThostFtdcTimeConditionType | TimeCondition | 有效期类型 | 否 |
| TThostFtdcOrderMemoType | OrderMemo | 报单回显字段 | 无 |
| TThostFtdcSequenceNo12Type | SessionReqSeq | session上请求计数 api自动维护 | 无 |

ForQuoteSysID：询价编号，用于唯一定位一笔询价，需要纯数字递增

AskOrderRef：需要纯数字递增

BidOrderRef：要比AskOrderRef大，需要纯数字递增

IPAddress：中继需填写客户IP地址；非中继填写无效，直接取登录成功会话中的IP。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

MacAddress：中继需填写客户MAC地址；非中继填写无效，直接取登录成功会话中的MAC。

OrderMemo:报单回显字段，OrderMemo字段可供终端厂商标记订单使用，CTP不做处理，即终端填写什么CTP就返回什么

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

SessionReqSeq：一个session中，一笔业务流程周期中唯一序号，此序号由API自动填写，客户维护无效。

OrderMemo：该字段CTP不做任何处理，即终端填写什么CTP就返回什么。可以用来给多账户系统做标记用。

<a id="fbd407eb-5a96-42eb-9dca-5145a980a1da"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="14d524d2-b334-444d-8d8c-38ec1b47eb0a"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4.示例调用
<a id="panel4"></a>

```
CThostFtdcInputQuoteField t = { 0 };
strcpy_s(t.BrokerID, "9999");
strcpy_s(t.InvestorID, "1000001");
strcpy_s(t.InstrumentID, "rb1809");
strcpy_s(t.UserID, "1000001");
strcpy_s(t.ExchangeID, "SHFE");
t.AskPrice = 200;
t.BidPrice = 150;
t.AskVolume = 1;
t.BidVolume = 1;
t.AskOffsetFlag = THOST_FTDC_OF_Open;///卖开平标志
t.BidOffsetFlag = THOST_FTDC_OF_Open;///买开平标志
t.AskHedgeFlag = THOST_FTDC_HF_Hedge;///卖投机套保标志
t.BidHedgeFlag = THOST_FTDC_HF_Hedge;///买投机套保标志
_itoa_s(OrderRef, t.AskOrderRef, 10);///衍生卖报单引用
OrderRef++;
_itoa_s(OrderRef, t.BidOrderRef, 10);///衍生买报单引用
OrderRef++;
m_pUserApi->ReqQuoteInsert(&t, nRequestID++);

```

<a id="ec8a2949-67ca-4aee-8680-9495c00eabe9"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

询价时报：“没有该合约的做市商”？<a id="region_panel_1"></a>

| 这是因为询价合约不对，目前期权合约可以参加询价。 |
|---|

<a id="region_tail_1"></a>

<a id="region_header_2"></a>

询价时报：“CTP：当前时间禁止询价”？<a id="region_panel_2"></a>

| 这是因为期货公司一般把询价限制时间设置为60秒询价一次，周期内不能多次询价。 |
|---|

<a id="region_tail_2"></a>

<a id="region_header_3"></a>

询价时报：“CTP：当前价差禁止询价”？<a id="region_panel_3"></a>

| 经纪公司代码 | 合约代码 | 交易所代码 | 最新价 | 价差 |
|---|---|---|---|---|
| 1008 | SRC | CZCE | 0 | 8 |
| 1008 | SRC | CZCE | 50 | 10 |
| 1008 | SRC | CZCE | 100 | 20 |
| 1008 | SRC | CZCE | 200 | 30 |
| 1008 | SRC | CZCE | 300 | 50 |
| 1008 | SRC | CZCE | 500 | 75 |

询价价差的判断过程

1）看最新价对应于上面的哪一档次，确定价差的最小值

2）计算买价和卖价的价差，看是否大于设置的价差（等于也不行）

如果2）通过，那么询价单报入交易所，否则会被CTP直接拒绝。

| 1008 | SRC | CZCE | 500 | 75 | 75 |
|---|---|---|---|---|---|

<a id="region_tail_3"></a>

<a id="region_header_4"></a>

“非法的做市商响应”是什么原因？<a id="region_panel_4"></a>

| 这可能是所用交易编码非做市商专用所致。 |
|---|

<a id="region_tail_4"></a>

<a id="region_header_5"></a>

各家交易所第二次报价是否会撤销第一次报价？<a id="region_panel_5"></a>

| 中金所：不会撤销
大商所：不会撤销
郑商所：会撤销
上期所：会撤销 |
|---|

<a id="region_tail_5"></a>

<a id="region_header_6"></a>

各家交易所使用的报价接口是否一样？<a id="region_panel_6"></a>

| 中金所：只有期权做市商,使用ReqQuoteInsert
大商所：使用ReqQuoteInsert
郑商所：使用ReqQuoteInsert
上期所：期货合约使用ReqOrderInsert，期权合约使用ReqQuoteInsert |
|---|

<a id="region_tail_6"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
