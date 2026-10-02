# ReqUserAuthMethod

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqUserAuthMethod<a id="content"></a>

<a id="left_menu"></a>

  ** **

查询用户当前支持的认证模式，暂不支持

响应: [OnRspUserAuthMethod](pages/298-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERAUTHMETHOD.html.md)
<a id="7da37171-1a5d-4517-8901-d7522ab703c2"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqUserAuthMethod(CThostFtdcReqUserAuthMethodField *pReqUserAuthMethod, int nRequestID) = 0;

<a id="63582582-5634-4a73-ad5d-ed2c516ad2e4"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pReqUserAuthMethod：用户发出获取安全安全登陆方法请求

| 字段类型 | 字段名称 | 含义 | 值 |
|---|---|---|---|
| TThostFtdcDateType | TradingDay | 交易日 | 无 |
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 无 |
| TThostFtdcUserIDType | UserID | 用户代码 | 无 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="1e6ca63c-2dc8-411a-96fb-78db7e52460b"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="c76e2bd7-6247-46dc-9418-7b856887ca83"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

无

<a id="beb26316-d084-4455-92b1-5e5b9e17c7bc"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
