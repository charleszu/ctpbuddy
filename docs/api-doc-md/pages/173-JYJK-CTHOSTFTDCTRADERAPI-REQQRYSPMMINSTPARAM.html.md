# ReqQrySPMMInstParam

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQrySPMMInstParam<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求SPMM合约参数查询，对应响应请求[OnRspQrySPMMInstParam](pages/344-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPMMINSTPARAM.html.md)
<a id="6d85de5d-61a9-4a42-93f0-7cf143502985"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQrySPMMInstParam(CThostFtdcQrySPMMInstParamField *pQrySPMMInstParam, int nRequestID) = 0;

<a id="db07bb39-d599-4bc1-bd54-cccda229c1d2"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQrySPMMInstParam：SPMM合约参数查询

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | InstrumentID | 合约代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="886bf87b-9fe5-446f-888b-b89d8bdd574d"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="23d7d474-8eff-4414-8443-20c73e324fbc"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="8d960998-6f19-4fe3-a578-c66c6ffab9ed"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
