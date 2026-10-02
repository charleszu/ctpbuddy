# ReqHedgeCfm

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqHedgeCfm<a id="content"></a>

<a id="left_menu"></a>

  ** **

套保确认请求

若CTP校验通过后返回[OnRtnHedgeCfm](pages/379-JYJK-CTHOSTFTDCTRADERSPI-ONRTNHEDGECFM.html.md)通知

如果CTP校验未通过返回[OnRspHedgeCfm](pages/376-JYJK-CTHOSTFTDCTRADERSPI-ONRSPHEDGECFM.html.md)响应和[OnErrRtnHedgeCfm](pages/380-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNHEDGECFM.html.md)错误回报

从交易所回来收到回报后给[OnRtnHedgeCfm](pages/379-JYJK-CTHOSTFTDCTRADERSPI-ONRTNHEDGECFM.html.md)通知（若交易所校验未通过会有[OnErrRtnHedgeCfm](pages/380-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNHEDGECFM.html.md)的）。
<a id="ba21547d-dce2-43c5-aeb4-c3b559768058"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqHedgeCfm(CThostFtdcInputHedgeCfmField *pInputHedgeCfm, int nRequestID) = 0;

<a id="e381e9e0-d65a-44b2-943a-990d0cf86da7"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pInputHedgeCfm：套保确认输入基本信息

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 必填 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 必填 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 必填 |
| TThostFtdcVolumeType | Volume | 数量 | 必填 |
| TThostFtdcDirectionType | Direction | 买卖方向 | 必填 |
| TThostFtdcRequestIDType | RequestID | 请求编号 | 否 |
| TThostFtdcOrderRefType | OrderRef | 报单引用 | 否 |
| TThostFtdcIPAddressType | IPAddress | IP地址 | 否 |
| TThostFtdcMacAddressType | MacAddress | Mac地址 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="0c85e806-41a7-4ec0-8516-23c7ecfa73c7"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="9aa8bcce-9696-4e77-88e9-ccb6da7fac99"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="0875cb09-52f1-4046-ad61-3aa58a3ac364"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
