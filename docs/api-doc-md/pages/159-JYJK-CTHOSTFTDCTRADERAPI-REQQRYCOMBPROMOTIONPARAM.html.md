# ReqQryCombPromotionParam

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryCombPromotionParam<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求组合优惠比例，对应响应请求[OnRspQryCombPromotionParam](pages/330-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCOMBPROMOTIONPARAM.html.md)

暂时只支持期权x参数查询，不支持期货。即大商所2025年12月31日推出的期货对锁、跨期、跨品种组合的新优惠中的x参数暂时不支持查询。

详见  [6.5.1版本更新说明补充说明](pages/006-6.5.1BBGXSMBCSM.html.md)
<a id="b48f3274-0162-4fb3-b68e-d7e8ed3d49bc"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryCombPromotionParam(CThostFtdcQryCombPromotionParamField *pQryCombPromotionParam, int nRequestID) = 0;

<a id="959f377e-4afb-4c0f-ab8e-6276a8e8f9f6"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryCombPromotionParam：查询组合优惠比例

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="d2834cea-8f6e-4e17-a37e-58eb425554b3"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="ccd05309-9963-4042-9ce5-f866356fa5cb"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="bb671bf7-f3f1-4f65-aca2-5c76dd72a279"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
