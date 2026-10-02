# ReqUserLogout

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqUserLogout<a id="content"></a>

<a id="left_menu"></a>

  ** **

登出请求，对应响应[OnRspUserLogout](pages/300-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERLOGOUT.html.md)。暂不支持。
<a id="5caef795-bbd5-4149-beb6-2ecaa30e8be6"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqUserLogout(CThostFtdcUserLogoutField *pUserLogout, int nRequestID) = 0;

<a id="143e638a-0309-456e-a640-8d8697938be7"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pUserLogout：用户登出请求

```
struct CThostFtdcUserLogoutField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///用户代码
    TThostFtdcUserIDType UserID;
};

```

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="e73da229-81a1-4b81-b652-1a652ccff439"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="5b15e105-28f7-47d1-b5c6-819af83ce892"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcUserLogoutField a = { 0 };
m_pUserApi->ReqUserLogout(&a, nRequestID++);

```

<a id="a602fc71-c3eb-4a63-8680-40b1242c7013"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
