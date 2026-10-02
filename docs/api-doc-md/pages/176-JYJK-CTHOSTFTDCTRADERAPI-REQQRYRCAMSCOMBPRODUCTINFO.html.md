# ReqQryRCAMSCombProductInfo

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryRCAMSCombProductInfo<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求RCAMS产品组合信息查询，对应响应请求[OnRspQryRCAMSCombProductInfo](pages/347-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSCOMBPRODUCTINFO.html.md)
<a id="7821f1a0-8b14-42fe-ae52-a83ec112275f"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryRCAMSCombProductInfo(CThostFtdcQryRCAMSCombProductInfoField *pQryRCAMSCombProductInfo, int nRequestID) = 0;

<a id="2a360743-63a4-44b9-9213-032fa8e3faf1"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryRCAMSCombProductInfo：RCAMS产品组合信息查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcProductIDType | ProductID | 产品代码 | 是 |
| TThostFtdcProductIDType | CombProductID | 商品组代码 | 是 |
| TThostFtdcProductIDType | ProductGroupID | 商品群代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="8ba9e87e-41b2-4443-888a-6a115b0d670d"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="5c1046ad-7e1b-45c2-ac43-b478b616206a"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="c40ed51a-ac70-4dd1-98fe-6a61dafad16f"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
