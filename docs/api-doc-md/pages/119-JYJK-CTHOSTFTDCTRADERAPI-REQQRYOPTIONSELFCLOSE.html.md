# ReqQryOptionSelfClose

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryOptionSelfClose<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询期权自对冲

响应: [OnRspQryOptionSelfClose](pages/270-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYOPTIONSELFCLOSE.html.md)
<a id="ae27be41-fa95-4784-b4e0-44e0c28b94b6"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryOptionSelfClose(CThostFtdcQryOptionSelfCloseField *pQryOptionSelfClose, int nRequestID) = 0;

<a id="9c5ec602-be5b-4ff2-baf1-b86d6ddbd537"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryOptionSelfClose：期权自对冲查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcOrderSysIDType | OptionSelfCloseSysID | 期权自对冲编号 | 是 |
| TThostFtdcTimeType | InsertTimeStart | 开始时间 | 是 |
| TThostFtdcTimeType | InsertTimeEnd | 结束时间 | 是 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

[OnRspQryOptionSelfClose](pages/270-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYOPTIONSELFCLOSE.html.md)：由此能定位一笔期权自对冲的报单

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="621d764c-1740-4a78-adf7-b2e86a87a221"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="c760ae76-6db3-4330-ab3a-17e19180a91c"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryOptionSelfCloseField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
strcpy_s(a.ExchangeID, "SHFE");
m_pUserApi->ReqQryOptionSelfClose(&a, nRequestID++);

```

<a id="e483a0e5-549b-4e53-b4d8-d0fd1b3cd56c"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
