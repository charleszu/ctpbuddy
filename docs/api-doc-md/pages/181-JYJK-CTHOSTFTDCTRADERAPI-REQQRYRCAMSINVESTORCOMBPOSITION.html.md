# ReqQryRCAMSInvestorCombPosition

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRCAMSInvestorCombPosition<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS策略组合持仓查询，对应响应请求[OnRspQryRCAMSInvestorCombPosition](pages/352-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSINVESTORCOMBPOSITION.html.md)
<a id="27750fd5-6322-45dd-826a-3b738c9a9278"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRCAMSInvestorCombPosition(CThostFtdcQryRCAMSInvestorCombPositionField *pQryRCAMSInvestorCombPosition, int nRequestID) = 0;

<a id="64081b45-81f1-4240-affb-8288be3ca56b"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRCAMSInvestorCombPosition：RCAMS策略组合持仓查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |
| TThostFtdcInstrumentIDType | CombInstrumentID | 组合合约代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="6577e6a5-3790-456a-9181-2acfa4184f67"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="f6680388-0ebc-4c7c-a313-9fe5a021227e"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="de1087fa-f69e-487a-a685-7fdf2cc28bdb"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
