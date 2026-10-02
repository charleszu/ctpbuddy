# ReqQryRCAMSInterParameter

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRCAMSInterParameter<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS跨品种风险折抵参数查询，对应响应请求[OnRspQryRCAMSInterParameter](pages/350-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSINTERPARAMETER.html.md)
<a id="16826c1c-5499-4751-8d64-1742859d4d40"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRCAMSInterParameter(CThostFtdcQryRCAMSInterParameterField *pQryRCAMSInterParameter, int nRequestID) = 0;

<a id="7a97d1f4-e558-4db4-b01f-d3816f9a209f"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRCAMSInterParameter：RCAMS跨品种风险折抵参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcProductIDType | ProductGroupID | 商品群代码 | 是 |
| TThostFtdcProductIDType | CombProduct1 | 产品组合代码1 | 是 |
| TThostFtdcProductIDType | CombProduct2 | 产品组合代码2 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="cc142800-2622-47c2-b201-d914d98e3c6c"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="52adbde4-66a1-47a8-a63b-55fbe73cd7d5"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="dbe9e2c8-3483-4bd0-a098-2acb52d2a86e"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
