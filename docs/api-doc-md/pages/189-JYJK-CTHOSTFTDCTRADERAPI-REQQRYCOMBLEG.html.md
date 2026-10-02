# ReqQryCombLeg

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryCombLeg<a id="content"></a>

<a id="left_menu"></a>

  ** **

组合腿信息查询，对应响应请求[OnRspQryCombLeg](pages/360-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCOMBLEG.html.md)

该接口支持查询可申请组合合约的信息和套利合约信息，查询条件中单腿合约代码**（必填）**。

注：业务上只有大商所、广期所有可组合的合约（中金所的RCAMS也有，但目前未上线）。该接口若填写‘Z’（郑商所）时，也可返回郑商所的组合合约，业务上不建议这么用。

<a id="19241abe-c858-43bc-939d-01d0ec61210b"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryCombLeg(CThostFtdcQryCombLegField *pQryCombLeg, int nRequestID) = 0;

<a id="9c741d44-9de4-497e-9f64-0e3bf03bcfc5"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryCombLeg：组合腿信息查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | LegInstrumentID | 单腿合约代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="0fc65d73-4a90-4e5c-9769-ed08398b0944"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="62d67e7f-ef20-49c9-935e-680b2921d669"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="c3ff4bd0-3f4f-4bfc-9828-3a732bd744fb"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
