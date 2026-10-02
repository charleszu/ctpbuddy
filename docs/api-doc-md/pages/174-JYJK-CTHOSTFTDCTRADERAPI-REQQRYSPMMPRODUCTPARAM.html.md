# ReqQrySPMMProductParam

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySPMMProductParam<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求SPMM产品参数查询，对应响应请求[OnRspQrySPMMProductParam](pages/345-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPMMPRODUCTPARAM.html.md)
<a id="c354f9cf-a95b-4aa1-8850-5e519fd21afd"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySPMMProductParam(CThostFtdcQrySPMMProductParamField *pQrySPMMProductParam, int nRequestID) = 0;

<a id="fc79724c-4a0c-4368-891a-04852183d66c"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySPMMProductParam：SPMM产品参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcSPMMProductIDType | ProductID | 产品代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="b74fb67a-471d-4b23-bd1d-6be8f75bfef3"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="a4661538-076d-46ed-abed-e8374d06a57b"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="d7d2b3bf-8e01-4c4d-bdc7-b02279759000"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
