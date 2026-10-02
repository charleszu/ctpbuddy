# ReqQryBrokerTradingParams

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryBrokerTradingParams<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询经纪公司交易参数

<a id="anchor-id-01"></a>

**只能用投资者账号登录才能查到，不能用操作员登录去查**

响应: [OnRspQryBrokerTradingParams](pages/242-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYBROKERTRADINGPARAMS.html.md)
<a id="e8817b3b-0306-4081-949c-6dec4f12f1cc"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryBrokerTradingParams(CThostFtdcQryBrokerTradingParamsField *pQryBrokerTradingParams, int nRequestID) = 0;

<a id="b5cc7b97-9bce-4e8c-82ad-e5f0d58672df"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryBrokerTradingParams：查询经纪公司交易参数

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 是 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="a1328c2c-13a2-4e86-8b50-dbe9a838d1b6"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="3c516058-985d-4b26-a531-c327eb3b44a7"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryBrokerTradingParamsField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.InvestorID, "1000001");
m_pUserApi->ReqQryBrokerTradingParams(&a, nRequestID++);

```

<a id="b19168bc-6dfe-47a8-b97b-5794d124221b"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
