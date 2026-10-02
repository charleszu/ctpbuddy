# ReqQryTransferBank

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryTransferBank<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询转帐银行

响应: [OnRspQryTransferBank](pages/288-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRANSFERBANK.html.md)
<a id="59dedad9-2ab8-4d3f-b8b9-7e51a531ffcf"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryTransferBank(CThostFtdcQryTransferBankField *pQryTransferBank, int nRequestID) = 0;

<a id="d876981c-4f36-4400-a0d9-6960b12606b2"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryTransferBank：查询转帐银行

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBankIDType | BankID | 银行代码 | 是 |
| TThostFtdcBankBrchIDType | BankBrchID | 银行分中心代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="29ce8636-7427-4837-bf86-1a86dc196d29"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="81c48621-0a2d-4f7c-a11b-bf837b05f440"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="787e505d-6dcf-40f9-8fc1-df87cef4bb41"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
