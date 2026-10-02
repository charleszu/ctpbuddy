# ReqSpdApply

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqSpdApply<a id="content"></a>

<a id="left_menu"></a>

  ** **

套利确认请求

若CTP校验通过后给返回[OnRtnSpdApply](pages/373-JYJK-CTHOSTFTDCTRADERSPI-ONRTNSPDAPPLY.html.md)通知

如果CTP校验未通过返回[OnRspSpdApply](pages/370-JYJK-CTHOSTFTDCTRADERSPI-ONRSPSPDAPPLY.html.md)响应和[OnErrRtnSpdApply](pages/374-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNSPDAPPLY.html.md)错误回报

从交易所回来收到回报后给[OnRtnSpdApply](pages/373-JYJK-CTHOSTFTDCTRADERSPI-ONRTNSPDAPPLY.html.md)通知（若交易所校验未通过是没有[OnErrRtnSpdApply](pages/374-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNSPDAPPLY.html.md)的）。
<a id="6f1c505b-38d1-4cf3-a16e-e475ed602a94"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqSpdApply(CThostFtdcInputSpdApplyField *pInputSpdApply, int nRequestID) = 0;

<a id="a04d7460-a776-40c3-8a26-c23219d53492"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputSpdApply：套利确认输入基本信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填 |
| TThostFtdcInstrumentIDType | FirstLegInstrumentID | 合约代码 | 必填 |
| TThostFtdcInstrumentIDType | SecondLegInstrumentID | 合约代码 | 必填 |
| TThostFtdcVolumeType | Volume | 数量 | 必填 |
| TThostFtdcDirectionType | Direction | 买卖方向 | 必填 |
| TThostFtdcCmbTypeType | CmbType | 组合定单类型 | 必填 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 否 |
| TThostFtdcOrderRefType | OrderRef | 报单引用 | 否 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 否 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="aa34f337-86ae-4a40-938b-2eff348993d2"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="3dc5e716-f49a-4b72-ad1d-0c5d49d204ad"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="5c7dd35c-c1d6-469c-925a-c842565f851a"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
