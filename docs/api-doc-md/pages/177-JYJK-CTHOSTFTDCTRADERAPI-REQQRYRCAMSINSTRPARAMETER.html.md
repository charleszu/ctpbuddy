# ReqQryRCAMSInstrParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRCAMSInstrParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS同合约风险对冲参数查询，对应响应请求[OnRspQryRCAMSInstrParameter](pages/348-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSINSTRPARAMETER.html.md)
<a id="575382f5-16c2-4f32-b432-3c7d3904df7f"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRCAMSInstrParameter(CThostFtdcQryRCAMSInstrParameterField *pQryRCAMSInstrParameter, int nRequestID) = 0;

<a id="3efa80cb-ef72-4bc7-b52e-f8e322f6950f"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRCAMSInstrParameter：RCAMS同合约风险对冲参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcProductIDType | ProductID | 产品代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="a6f83825-bcea-4a7f-bdfc-3bc34e86bcde"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="8b133109-d71d-4e16-89f1-285362e30361"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="920debf1-07ee-4ccd-9af8-ba9abfde3e39"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
