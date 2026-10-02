# ReqQryRiskSettleProductStatus

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRiskSettleProductStatus<a id="content"></a>

<a id="left_menu"></a>

  ** **

风险结算产品查询，对应响应请求[OnRspQryRiskSettleProductStatus](pages/332-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRISKSETTLEPRODUCTSTATUS.html.md)

详见  [6.6.1P1版本更新说明](pages/007-6.6.1P1BBGXSM.html.md)
<a id="6bd7cd72-a5be-4d90-b741-e5e81ebb5153"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRiskSettleProductStatus(CThostFtdcQryRiskSettleProductStatusField *pQryRiskSettleProductStatus, int nRequestID) = 0;

<a id="38bd25c9-fd78-44c8-9f22-66e3cfcf1db1"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRiskSettleProductStatus：风险结算产品查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | ProductID | 产品代码 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="aa27b5c7-ea26-4e9c-aba0-5fef46c91f43"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="c6318e1e-daf2-4bbb-8505-9a5704137446"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="527e90a5-d30e-46c4-9fdc-937f208e543d"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
