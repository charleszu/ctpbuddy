# ReqQryRCAMSIntraParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRCAMSIntraParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS品种内风险对冲参数查询，对应响应请求[OnRspQryRCAMSIntraParameter](pages/349-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSINTRAPARAMETER.html.md)
<a id="52f29712-ec72-435c-9e1c-70f8d4250270"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRCAMSIntraParameter(CThostFtdcQryRCAMSIntraParameterField *pQryRCAMSIntraParameter, int nRequestID) = 0;

<a id="376308c2-e1da-4b43-88e6-2621e8651fce"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRCAMSIntraParameter：RCAMS品种内风险对冲参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcProductIDType | CombProductID | 产品组合代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="f52e12d5-ed09-4182-b650-bc03d294e2b8"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="eb7af659-13dc-488c-b75a-2993b91369ae"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="6dce4a50-3289-48d3-b672-34d29de6f6cd"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
