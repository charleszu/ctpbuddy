# ReqCancelOffsetSetting

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqCancelOffsetSetting<a id="content"></a>

<a id="left_menu"></a>

  ** **

对冲设置撤销请求，对应响应：对冲设置撤销请求响应[OnRspCancelOffsetSetting](pages/362-JYJK-CTHOSTFTDCTRADERSPI-ONRSPCANCELOFFSETSETTING.html.md),对冲设置通知[OnRtnOffsetSetting](pages/363-JYJK-CTHOSTFTDCTRADERSPI-ONRTNOFFSETSETTING.html.md),对冲设置撤销错误回报[OnErrRtnCancelOffsetSetting](pages/365-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNCANCELOFFSETSETTING.html.md)

大商所二阶段行权优化详见[大商所行权优化二阶段业务](pages/404-QTYWGZ-DSSHQYHEJDYW.html.md)

**注意：该接口仅适用大商所。**
<a id="4d607efb-21ce-4458-a9de-27f8e346c2ac"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqCancelOffsetSetting(CThostFtdcInputOffsetSettingField *pInputOffsetSetting, int nRequestID) = 0;

<a id="906836e7-0e93-4332-ba07-2bd523fd2af0"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputOffsetSetting：输入的对冲设置

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 按报入填 |
| TThostFtdcInstrumentIDType | UnderlyingInstrID | 标的期货合约代码 | 按报入填 |
| TThostFtdcProductIDType | ProductID | 产品代码 | 按报入填 |
| TThostFtdcOffsetTypeType | OffsetType | 对冲类型 | 必填 |
| TThostFtdcVolumeType | Volume | 申请对冲的合约数量 | 无 |
| TThostFtdcBoolType | IsOffset | 是否对冲 | 无 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 无 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填"DCE" |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 无 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 无 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="60e6b200-73cb-4365-a6b5-033967bef93e"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="ae7495b1-1c82-429a-acab-1cd4b7ffa511"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcInputOffsetSettingField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "00001");
strcpy_s(a.InstrumentID, "a2407-C-3850");
a.OffsetType = THOST_FTDC_OT_OPT_OFFSET;
strcpy_s(a.ExchangeID, "DCE);
m_pUserApi->ReqCancelOffsetSetting(&a, nRequestID++);

```

<a id="0b6f7ce6-db67-41f0-842a-75e54c53248c"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
