# ReqQryUserSession

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryUserSession<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询用户会话，查询响应[OnRspQryUserSession](pages/367-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYUSERSESSION.html.md)
<a id="fd2a1093-6a27-4acf-a018-73149f0b8010"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryUserSession(CThostFtdcQryUserSessionField *pQryUserSession, int nRequestID) = 0;

<a id="028aa70c-52da-46cd-b92a-7e347dda67f2"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryUserSession：查询用户会话

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcFrontIDType | FrontID | 前置编号 | 必填 |
| TThostFtdcSessionIDType | SessionID | 会话编号 | 否 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 必填 |
| TThostFtdcUserIDType | UserID | 用户代码 | 必填 |

ProductID：填写产品后，返回该产品下的所有设置。

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="c96b4ee2-9238-4cca-b7bc-65019a138fae"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="874a47a5-c287-431a-bb79-6e3ee5efac64"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="917f6d1c-1ea6-4862-9d74-8c96841c3eac"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
