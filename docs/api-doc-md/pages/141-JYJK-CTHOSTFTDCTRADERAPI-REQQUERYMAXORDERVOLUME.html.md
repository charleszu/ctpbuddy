# ReqQryMaxOrderVolume

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryMaxOrderVolume<a id="content"></a>

<a id="left_menu"></a>

  ** **

查询最大报单数量请求，对应响应[OnRspQryMaxOrderVolume](pages/328-JYJK-CTHOSTFTDCTRADERSPI-OnRspQryMaxOrderVolume.html.md)。

虽然ReqQryMaxOrderVolume可以查询可开，但是交易核心在计算的时候是没有算手续费的，所以不完全准，计算逻辑是按照昨结算价计算的可开；如果需要精确结果的，建议自行计算。另外，可平的查询是已经排除了冻结持仓的。
<a id="691b0c9d-bd9c-469e-ac74-d46525a82db1"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryMaxOrderVolume(CThostFtdcQryMaxOrderVolumeField *pQryMaxOrderVolume, int nRequestID) = 0;

<a id="9ffb3ebb-d06b-486f-9617-2507ba86dad3"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryMaxOrderVolume：查询最大报单数量

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInvestUnitIDType | InvestUnitID | 投资单元代码 | 否 |
| TThostFtdcVolumeType | MaxVolume | 最大允许报单数量 | 否 |
| TThostFtdcDirectionType | Direction | 买卖方向 | 是 |
| TThostFtdcOffsetFlagType | OffsetFlag | 开平标志 | 是 |
| TThostFtdcHedgeFlagType | HedgeFlag | 投机套保标志 | 是 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

MaxVolume：取值结果为，min（可开/可平，限价单最大下单量）；可开根据昨结算价计算得出。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="5f6ff6e3-68e5-492d-be31-e9b6ee0f4e55"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="cf57cb23-4712-411a-88ed-b271ff1e8314"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryMaxOrderVolumeField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
strcpy_s(a.InstrumentID, "rb1809");
a.Direction = THOST_FTDC_D_Buy;
a.OffsetFlag = THOST_FTDC_OF_Open;
a.HedgeFlag = THOST_FTDC_HF_Speculation;
a.MaxVolume = 1;
m_pUserApi->ReqQryMaxOrderVolume(&a, nRequestID++);

```

<a id="3f494387-5f23-4f51-af13-c013b014cfbd"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
