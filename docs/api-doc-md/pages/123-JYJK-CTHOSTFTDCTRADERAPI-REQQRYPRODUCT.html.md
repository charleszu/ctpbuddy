# ReqQryProduct

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryProduct<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询产品

响应: [OnRspQryProduct](pages/274-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYPRODUCT.html.md)
<a id="5d2fb7eb-f5a5-4824-9f5e-d2a2e503f588"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryProduct(CThostFtdcQryProductField *pQryProduct, int nRequestID) = 0;

<a id="5db04708-dfd2-4c09-ae62-441856e6f30f"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryProduct：查询产品

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcInstrumentIDType | ProductID | 产品代码 | 是 |
| TThostFtdcExchangeIDType | ExchangeID | 交易所代码 | 否 |
| TThostFtdcProductClassType | ProductClass | 产品类型 | 否 |
| TThostFtdcOldInstrumentIDType | reserve1 | 保留的无效字段 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="7f1ba78b-7455-4f59-96e1-b0dbe130c45e"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="4421ef61-9c28-4a60-a393-ec857460c943"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryProductField a = { 0 };
strcpy_s(a.ProductID, "sc");
a.ProductClass = THOST_FTDC_PC_Futures;
strcpy_s(a.ExchangeID, "SHFE");
m_pUserApi->ReqQryProduct(&a, nRequestID++);

```

<a id="465b27b8-80af-4ff1-b764-b04148f840e3"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
