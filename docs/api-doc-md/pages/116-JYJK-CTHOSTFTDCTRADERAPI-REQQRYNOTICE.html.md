# ReqQryNotice

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryNotice<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询客户通知，用于查询柜台的客户通知书，

响应: [OnRspQryNotice](pages/267-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYNOTICE.html.md)
<a id="185b163a-aa70-4b34-8e17-e1c2727c3786"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryNotice(CThostFtdcQryNoticeField *pQryNotice, int nRequestID) = 0;

<a id="1bf86f37-2c52-4fc8-bef4-9b1e0962ec1c"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryNotice：查询客户通知

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="77bf9220-3046-46f2-bace-f5061d8d5682"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="82469f75-4eb2-4e03-92bc-6c581a32c45e"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="ea9d42d2-0fbd-40cf-ba9c-a50471659d55"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
