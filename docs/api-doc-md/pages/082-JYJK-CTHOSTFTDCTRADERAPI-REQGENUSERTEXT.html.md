# ReqGenUserText

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqGenUserText<a id="content"></a>

<a id="left_menu"></a>

  ** **

用户发出获取短信验证码请求，暂不支持

响应: [OnRspGenUserText](pages/233-JYJK-CTHOSTFTDCTRADERSPI-ONRSPGENUSERTEXT.html.md)
<a id="5a7d0c43-1e7b-4f83-ae5e-9b644df7bab5"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqGenUserText(CThostFtdcReqGenUserTextField *pReqGenUserText, int nRequestID) = 0;

<a id="42de1506-2fee-4ef3-8925-ae7426028b98"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pReqGenUserText：用户发出获取安全安全登陆方法请求

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcDateType | TradingDay | 交易日 | 无 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 无 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="6f7c674e-a8f8-4dae-8120-4e901bb5e4ce"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="720c8df2-57c4-49ec-8189-a68001ad6ace"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="d7d1dac1-23fa-40ad-be4f-a92876e014e3"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
