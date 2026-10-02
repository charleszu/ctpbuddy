# ReqOrderInsert

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqOrderInsert<a id="content"></a>

<a id="left_menu"></a>

  ** **

报单录入请求，录入错误时对应响应[OnRspOrderInsert](pages/237-JYJK-CTHOSTFTDCTRADERSPI-ONRSPORDERINSERT.html.md)、[OnErrRtnOrderInsert](pages/214-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNORDERINSERT.html.md)，正确时对应回报[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)、[OnRtnTrade](pages/326-JYJK-CTHOSTFTDCTRADERSPI-ONRTNTRADE.html.md)。

可以录入限价单、市价单、条件单等交易所支持的指令，撤单时使用[ReqOrderAction](pages/085-JYJK-CTHOSTFTDCTRADERAPI-REQORDERACTION.html.md)。

不支持预埋单录入，预埋单请使用[ReqParkedOrderInsert](pages/088-JYJK-CTHOSTFTDCTRADERAPI-REQPARKEDORDERINSERT.html.md)。

详见[报单回调规则](pages/389-QTYWGZ-DBHB.html.md)

关于流控详见[报单流控、查询流控和会话数控制](pages/396-QTYWGZ-LK.html.md)

关于Tas的说明详见[TAS介绍](pages/398-QTYWGZ-TASJS.html.md)

关于[条件单规则](pages/399-QTYWGZ-TJDGZ.html.md)详见[条件单规则](pages/399-QTYWGZ-TJDGZ.html.md)

关于各个交易所的特殊指令详见[各交易所特殊指令](pages/400-QTYWGZ-GJYSTSZL.html.md)

关于接口中的重要序号说明详见[接口中一些重要序号说明](pages/401-QTYWGZ-JKZYXZYXHSM.html.md)
<a id="f51240e1-0d26-41f4-817d-573c34b38f5d"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqOrderInsert(CThostFtdcInputOrderField *pInputOrder, int nRequestID) = 0;

<a id="c52338c1-1c98-42ec-9a3d-abef4ffcab57"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputOrder：输入报单

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填 |
| TThostFtdcOrderRefType | OrderRef | 报单引用 | 可自定义或不填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcCombOffsetFlagType | CombOffsetFlag | 开平标志 | 必填 |
| TThostFtdcCombHedgeFlagType | CombHedgeFlag | 投机套保标志 | 必填 |
| TThostFtdcDateType | GTDDate | GTD日期 | 无 |
| TThostFtdcBusinessUnitType | BusinessUnit | 业务单元 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 无 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 无 |
| TThostFtdcClientIDType | ClientID | 客户代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcVolumeType | VolumeTotalOriginal | 数量 | 必填 |
| TThostFtdcVolumeType | MinVolume | 最小成交量 | 无 |
| TThostFtdcBoolType | IsAutoSuspend | 自动挂起标志 | 必填0 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcBoolType | UserForceClose | 用户强平标志 | 无 |
| TThostFtdcBoolType | IsSwapOrder | 互换单标志 | 必填 |
| TThostFtdcOrderPriceTypeType | OrderPriceType | 报单价格条件 | 必填 |
| TThostFtdcDirectionType | Direction | 买卖方向 | 必填 |
| TThostFtdcTimeConditionType | TimeCondition | 有效期类型 | 必填 |
| TThostFtdcVolumeConditionType | VolumeCondition | 成交量类型 | 必填 |
| TThostFtdcContingentConditionType | ContingentCondition | 触发条件 | 必填 |
| TThostFtdcForceCloseReasonType | ForceCloseReason | 强平原因 | 必填 |
| TThostFtdcPriceType | LimitPrice | 价格 | 必填 |
| TThostFtdcPriceType | StopPrice | 止损价 | 必填 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 无 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 无 |
| TThostFtdcOrderMemoType | OrderMemo | 报单回显字段 | 无 |
| TThostFtdcSequenceNo12Type | SessionReqSeq | session上请求计数 api自动维护 | 无 |

OrderPriceType：确定输入的报单类型，比如限价单则填写THOST_FTDC_OPT_LimitPrice、市价单则填写THOST_FTDC_OPT_AnyPrice。

Direction：确定买卖方向

<a id="anchor-id-05"></a>

CombOffsetFlag：确定开平标志。上期所和能源交易所支持平昨和平今指令，分别对应昨仓和今仓；如果下平仓指令，则等同于平昨指令。大商所、广期所、郑商所、中金所按交易所规则只能报入平仓指令，如果报入平昨平今指令ctp会转成平仓指令报出。

<a id="anchor-id-04"></a>

CombHedgeFlag：确定投机套保标志，注：郑商所平仓只能报入“投机”属性 例:投机 THOST_FTDC_BHF_Speculation

TimeCondition：确定报单有效期类型，目前只支持GFD和IOC，其他都不支持 例:立即完成，否则撤销 THOST_FTDC_TC_IOC，为支持大商所GIS属性交易指令，新增THOST_FTDC_TC_GFS枚举值

VolumeCondition：确定成交量类型

ContingentCondition：确定触发条件

StopPrice：止损价，用于条件单的触发价格

ForceCloseReason：普通用户下单填写THOST_FTDC_FCC_NotForceClose 非强平

IsSwapOrder：互换单填1

CurrencyID：不填写默认为CNY

OrderRef：OrderRef是本地会话全局唯一编号，必须保持递增；可由用户维护，也可由系统自动填写。一定为数字。

IPAddress：中继需填写客户IP地址；非中继填写无效，直接取登录成功会话中的IP。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

MacAddress：中继需填写客户MAC地址；非中继填写无效，直接取登录成功会话中的MAC。

BusinessUnit：保留字段。

<a id="anchor-id-01"></a>

LimitPrice：郑商所报单的价格会被交易所按照最小变动价位截取。(eg:报入1500.9，最小变动价位是1，会被交易所截取为1500。)其他交易所则直接返回错误。

OrderMemo:报单回显字段，OrderMemo字段可供终端厂商标记订单使用，CTP不做处理，即终端填写什么CTP就返回什么

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

SessionReqSeq：一个session中，一笔业务流程周期中唯一序号，此序号由API自动填写，客户维护无效。

OrderMemo：该字段CTP不做任何处理，即终端填写什么CTP就返回什么。可以用来给多账户系统做标记用。

<a id="3a67f226-a128-47e7-ae7b-388898a53326"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="b97b2f90-f457-452f-937d-44f6eb94f93b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
//限价单:
CThostFtdcInputOrderField ord = { 0 };
strcpy_s(ord.BrokerID, “0000”);
strcpy_s(ord.InvestorID, “00001”);
strcpy_s(ord.ExchangeID, “SHFE”);
strcpy_s(ord.InstrumentID, “ag1801”);
strcpy_s(ord.UserID, “00001”);
ord.OrderPriceType = THOST_FTDC_OPT_LimitPrice;//限价
ord.Direction = THOST_FTDC_D_Buy;//买
ord.CombOffsetFlag[0] = THOST_FTDC_OF_Open;//开
ord.CombHedgeFlag[0] = THOST_FTDC_HF_Speculation;//投机
ord.LimitPrice = 100;
ord.VolumeTotalOriginal = 1;
ord.TimeCondition = THOST_FTDC_TC_GFD;///当日有效
ord.VolumeCondition = THOST_FTDC_VC_AV;///任意数量
ord.MinVolume = 1;
ord.ContingentCondition = THOST_FTDC_CC_Immediately;
ord.StopPrice = 0;
ord.ForceCloseReason = THOST_FTDC_FCC_NotForceClose;
ord.IsAutoSuspend = 0;
m_pUserApi->ReqOrderInsert(&ord, nRequestID++);

```

<a id="681431f9-c307-430f-9e71-8cd049f39a57"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

郑商所白糖期权的跨式和宽跨式套利指令，在ctp里如何实现？<a id="region_panel_1"></a>

| 和普通的限价单类似，不同的地方是在填写InstrumentID的时候
郑商所跨式组合（STD）: STD SR703C5300&SR703P5300
郑商所宽跨式组合（STG): STG SR703C5300&SR703P5200
支持FAK和FOK |
|---|

<a id="region_tail_1"></a>

<a id="region_header_2"></a>

“已撤单报单被拒绝CZCE:出错:不支持无限制期权组合订单”是什么原因？<a id="region_panel_2"></a>

| 郑商所的组合合约下单，TimeCondition应该用THOST_FTDC_TC_IOC（1）。 |
|---|

<a id="region_tail_2"></a>

<a id="region_header_3"></a>

如果下单后遇到报错“ctp：每秒报单数超过许可数”，是什么原因？<a id="region_panel_3"></a>

| 这是由于报单频率太快被前置流控所致。投资者可询问期货公司具体的流控设置；期货公司可以自行查看FLEX"程序化交易频繁报撤单管理"中的设置，以及Front的流控配置。注意：Front的流控会放到下一秒，不会报错；FLEX"程序化交易频繁报撤单管理"中的流控会报错。 |
|---|

<a id="region_tail_3"></a>

<a id="region_header_4"></a>

“CTP:报单错误：不允许重复报单”，是什么原因？<a id="region_panel_4"></a>

| 这是表示OrderRef没有递增。 |
|---|

<a id="region_tail_4"></a>

<a id="region_header_5"></a>

“CTP:无此权限”，是什么原因？<a id="region_panel_5"></a>

| 这可能InvestorID填写错误，或者该InvestorID不属于UserID的组织架构下。 |
|---|

<a id="region_tail_5"></a>

<a id="region_header_6"></a>

“CTP:交易所未处理请求超过许可数”，是什么原因？<a id="region_panel_6"></a>

| 表示报单频率过快，超过了交易所的流控。 |
|---|

<a id="region_tail_6"></a>

<a id="region_header_7"></a>

“CTP:交易所每秒发送请求数超过许可数”，是什么原因？<a id="region_panel_7"></a>

| 表示报单频率过快，超过了交易所的流控。 |
|---|

<a id="region_tail_7"></a>

<a id="anchor-id-02"></a>

<a id="region_header_8"></a>

投机套保标志能够用来判断报单所使用的交易编码类型？<a id="region_panel_8"></a>

| ctp支持，但是有些交易所不支持多交易编码。除中金所有投机、套保、套利三个交易编码外，其他交易所只有投机交易编码。并且交易编码还有做市商类型，但是投机套保中没有做市商这种类型。 |
|---|

<a id="region_tail_8"></a>

<a id="anchor-id-03"></a>

<a id="region_header_9"></a>

req请求返回值为0，可能出现回报收不到的情况吗？<a id="region_panel_9"></a>

| 可能的，Req返回为0，代表socket.write成功，并不表示柜台那边已经收到了请求。不保证一定有OnRsp和OnErrRtn的情况。 |
|---|

<a id="region_tail_9"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
