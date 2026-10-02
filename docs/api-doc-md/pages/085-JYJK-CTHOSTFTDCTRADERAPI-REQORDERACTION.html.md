# ReqOrderAction

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqOrderAction<a id="content"></a>

<a id="left_menu"></a>

  ** **

报单操作请求

错误响应: [OnRspOrderAction](pages/236-JYJK-CTHOSTFTDCTRADERSPI-ONRSPORDERACTION.html.md)，[OnErrRtnOrderAction](pages/213-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNORDERACTION.html.md)

正确响应：[OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)

关于流控详见[报单流控、查询流控和会话数控制](pages/396-QTYWGZ-LK.html.md)

关于接口中的重要序号说明详见[接口中一些重要序号说明](pages/401-QTYWGZ-JKZYXZYXHSM.html.md)
<a id="f7217664-7719-45cd-8569-7aa24546185a"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqOrderAction(CThostFtdcInputOrderActionField *pInputOrderAction, int nRequestID) = 0;

<a id="8cba21cf-5587-448d-85a6-fcaddfd4376e"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputOrderAction：输入报单操作

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcOrderRefType | OrderRef | 报单引用 | 必填*1 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填 |
| TThostFtdcOrderSysIDType | OrderSysID | 报单编号 | 必填*2 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 无 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |
| TThostFtdcOrderActionRefType | OrderActionRef | 报单操作引用 | 必填，递增 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcFrontIDType | FrontID | 前置编号 | 必填*1 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 必填*1 |
| TThostFtdcVolumeType | VolumeChange | 数量变化 | 无 |
| TThostFtdcActionFlagType | ActionFlag | 操作标志 | 必填 |
| TThostFtdcPriceType | LimitPrice | 价格 | 无 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 无 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 无 |
| TThostFtdcOldIPAddressType | reserve2 | 保留的无效字段 | 无 |
| TThostFtdcOrderMemoType | OrderMemo | 报单回显字段 | 无 |
| TThostFtdcSequenceNo12Type | SessionReqSeq | session上请求计数 api自动维护 | 无 |

**必填*1、必填*2**：两组选一组必填，能对应要撤的报单。

OrderRef：对应要撤销的那笔报单的报单引用，需委托回报([OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md))中完整的字符串，不可增减

FrontID: 对应要撤销的那笔报单的前置编号

SessionID: 对应要撤销的那笔报单的会话编号

ExchangeID: 对应要撤销的那笔报单的交易所ID

OrderSysID: 对应要撤销的那笔报单的报单编号，需委托回报([OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md))中完整的字符串，不可增减

ActionFlag：只支持删除，不支持修改、激活、挂起。对于由其他柜台做的激活或挂起的报单，ctp系统能正常接收。

IPAddress：手工填写本机IP地址，不自动获取。填写规则如下：ipv4原样填写，ipv6要转成非零压缩地址，即原始地址，同时要去掉冒号，eg：AAAABBBBCCCCDDDDEEEEFFFFGGGGHHHH

OrderMemo:报单回显字段，OrderMemo字段可供终端厂商标记订单使用，CTP不做处理，即终端填写什么CTP就返回什么

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

SessionReqSeq：一个session中，一笔业务流程周期中唯一序号，此序号由API自动填写，客户维护无效。

OrderMemo：该字段CTP不做任何处理，即终端填写什么CTP就返回什么。可以用来给多账户系统做标记用。

<a id="e193f748-7429-4262-8fba-f0e4580f42ca"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="681b8259-0e6d-41d9-b4aa-45eefb3686e2"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

//第一种方法，使用OrderSysID撤单（**推荐使用**）

```
CThostFtdcInputOrderActionField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.UserID, "1000001");
strcpy_s(a.OrderSysID, "         131");  //对应要撤报单的OrderSysID
strcpy_s(a.ExchangeID, "SHFE");
strcpy_s(a.InstrumentID, "rb1809");
ActionFlag = THOST_FTDC_AF_Delete;
m_pUserApi->ReqOrderAction(&a, nRequestID++);

```

//第二种方法，使用FrontID+SessionID+OrderRef撤单

```
CThostFtdcInputOrderActionField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.UserID, "1000001");
a.FrontID = 1;   //对应要撤报单的FrontID
a.SessionID = -788541;   //对应要撤报单的sessionid
strcpy_s(a.OrderRef, "        3");   //对应要撤报单的OrderRef
strcpy_s(a.ExchangeID, "SHFE");
strcpy_s(a.InstrumentID, "rb1809");
ActionFlag = THOST_FTDC_AF_Delete;
m_pUserApi->ReqOrderAction(&a, nRequestID++);

```

<a id="8717e8bf-f13d-4cd5-803d-d0a606ab30c5"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="anchor-id-01"></a>

<a id="region_header_1"></a>

ctp支持撤销未知单吗？<a id="region_panel_1"></a>

| 目前665版本以上柜台不允许撤郑商所的未知单，其他交易所都是可以撤未知单的。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
