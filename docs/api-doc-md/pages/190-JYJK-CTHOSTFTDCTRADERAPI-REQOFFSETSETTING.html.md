# ReqOffsetSetting

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqOffsetSetting<a id="content"></a>

<a id="left_menu"></a>

  ** **

对冲设置请求，对应响应：对冲设置请求响应[OnRspOffsetSetting](pages/361-JYJK-CTHOSTFTDCTRADERSPI-ONRSPOFFSETSETTING.html.md),对冲设置通知[OnRtnOffsetSetting](pages/363-JYJK-CTHOSTFTDCTRADERSPI-ONRTNOFFSETSETTING.html.md),对冲设置错误回报[OnErrRtnOffsetSetting](pages/364-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNOFFSETSETTING.html.md)

大商所二阶段行权优化详见[大商所行权优化二阶段业务](pages/404-QTYWGZ-DSSHQYHEJDYW.html.md)

**注意：该接口仅适用大商所**。
<a id="53e461b5-507a-4835-8d68-383127ffac3b"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqOffsetSetting(CThostFtdcInputOffsetSettingField *pInputOffsetSetting, int nRequestID) = 0;

<a id="2e779b08-bb03-4952-b5ea-e11b41a59a82"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputOffsetSetting：输入的对冲设置

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 见示例 |
| TThostFtdcInstrumentIDType | UnderlyingInstrID | 标的期货合约代码 | 见示例 |
| TThostFtdcProductIDType | ProductID | 产品代码 | 见示例 |
| TThostFtdcOffsetTypeType | OffsetType | 对冲类型 | 必填 |
| TThostFtdcVolumeType | Volume | 申请对冲的合约数量 | 必填 |
| TThostFtdcBoolType | IsOffset | 是否对冲 | 必填 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填"DCE" |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |

关于**InstrumentID、UnderlyingInstrID、ProductID**填写规范

（1）期权对冲，选填InstrumentID(期权合约)、UnderlyingInstrID(标的期货合约)、ProductID(期权产品)三个字段，至少一个不为空；

（2）期货对冲，选填InstrumentID(期货合约)、ProductID(期货品种)字段，UnderlyingInstrID字段不填;

（3）行权后对冲、履约后对冲，选填UnderlyingInstrID(标的期货合约)，ProductID(期权品种)字段，InstrumentID不填;

**Volume**：设置合约级别的期权对冲和期货对冲且IsOffset=true时，要求填入的Volume在[1,99999999]区间内，其他情况不检查手数字段，99999999表示全部手数。

**RequestID**：对于同一“投资者+对冲粒度+对冲类型”的对冲，后台只会保留一笔;若用户需要区分每笔不同的报单，建议使用RequestID字段区分，此字段只做回显，不做其他检查。

例：

（1）期权对冲

品种级别对冲：ProductID需要填具体，UnderlyingInstrID、InstrumentID为空

期权系列级别对冲：ProductID为空或具体，UnderlyingInstrID需要填具体，InstrumentID为空

合约级别对冲：ProductID、UnderlyingInstrID为空或者具体，InstrumentID需要填具体，Volume在[1,99999999]区间内，99999999表示全部手数

（2）行权后对冲、履约后对冲

客户级别对冲：ProductID、UnderlyingInstrID为空

品种级别对冲：ProductID需要填具体，UnderlyingInstrID为空

合约级别对冲：ProductID为空或者具体，UnderlyingInstrID为具体

（3）期货对冲

客户级别对冲：ProductID、InstrumentID为空

品种级别对冲：ProductID需要填具体，InstrumentID为空

合约级别对冲：ProductID为空或者具体，InstrumentID需要填具体，Volume在[1,99999999]区间内，99999999表示全部手数

<a id="f62f49b1-4a4a-437f-a410-bdc334878172"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="af781719-1e94-4d90-a81f-948af27b7930"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
// 合约级别对冲
CThostFtdcInputOffsetSettingField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "00001");
strcpy_s(a.InstrumentID, "a2507-C-3850");
strcpy_s(a.UnderlyingInstrID, "a2507");
strcpy_s(a.ProductID, "a_o");
a.OffsetType = THOST_FTDC_OT_OPT_OFFSET;
a.Volume = 1;
a.IsOffset = 1;
strcpy_s(a.UserID, "00001");
strcpy_s(a.ExchangeID, "DCE");
m_pUserApi->ReqOffsetSetting(&a, 1);

```

<a id="d29a5757-a988-46b4-bdfb-d08c28346902"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
