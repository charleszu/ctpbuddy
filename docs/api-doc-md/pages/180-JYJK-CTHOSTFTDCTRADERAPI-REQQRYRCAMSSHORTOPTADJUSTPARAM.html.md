# ReqQryRCAMSShortOptAdjustParam

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRCAMSShortOptAdjustParam<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS空头期权风险调整参数查询，对应响应请求[OnRspQryRCAMSShortOptAdjustParam](pages/351-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSSHORTOPTADJUSTPARAM.html.md)
<a id="64fbbc86-9d9b-4576-997e-0d574f53fadf"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRCAMSShortOptAdjustParam(CThostFtdcQryRCAMSShortOptAdjustParamField *pQryRCAMSShortOptAdjustParam, int nRequestID) = 0;

<a id="0aea5eee-cf30-4014-b972-8e1ee095edad"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRCAMSShortOptAdjustParam：RCAMS空头期权风险调整参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcProductIDType | CombProductID | 产品组合代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="49493c13-e6d4-443d-97e3-b7a5213ca909"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="3365a8c6-3533-4ce0-8f7e-d86f1569e276"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="7ca4dcbf-e9cc-4a25-aa37-4d1547718e88"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
