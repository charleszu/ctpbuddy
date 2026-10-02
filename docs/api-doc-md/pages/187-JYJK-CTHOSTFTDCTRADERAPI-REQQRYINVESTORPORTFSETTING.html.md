# ReqQryInvestorPortfSetting

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryInvestorPortfSetting<a id="content"></a>

<a id="left_menu"></a>

  ** **

投资者新型组合保证金开关查询，对应响应请求[OnRspQryInvestorPortfSetting](pages/358-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPORTFSETTING.html.md)
<a id="8cee851b-0834-418a-a2bb-1be33287a9d3"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryInvestorPortfSetting(CThostFtdcQryInvestorPortfSettingField *pQryInvestorPortfSetting, int nRequestID) = 0;

<a id="3a912f82-41b7-41d2-8a6b-aca847ea6fb5"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryInvestorPortfSetting：投资者新组保设置查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 是 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |

注：返回记录是交易编码级的，除中金所可能存在多条记录外，其他交易所均是一条记录。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="b4d8ea05-0fc9-4df3-bfcf-5304093ae1f7"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="8f5043fb-486c-4f70-8365-8557ffe73e24"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="579f5b53-5678-44a2-b7ae-eded0070f9b4"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
