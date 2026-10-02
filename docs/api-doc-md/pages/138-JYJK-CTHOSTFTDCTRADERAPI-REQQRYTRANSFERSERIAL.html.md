# ReqQryTransferSerial

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqQryTransferSerial<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询转帐流水。

响应：[OnRspQryTransferSerial](pages/289-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRANSFERSERIAL.html.md)
<a id="34218dd8-a420-4835-a571-131cba915171"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqQryTransferSerial(CThostFtdcQryTransferSerialField *pQryTransferSerial, int nRequestID) = 0;

<a id="0bb55493-7281-4a36-bc40-4b458df3e238"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pQryTransferSerial：请求查询转帐流水

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 是 |
| TThostFtdcBankIDType | BankID | 银行代码 | 是 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 是 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="ddd9f798-8088-4350-adb0-22e9dac9da63"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="d1af6a95-bbbf-4af2-ab70-694246511401"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcQryTransferSerialField a = { 0 };
strcpy_s(a.BrokerID, "9999");
strcpy_s(a.AccountID, "1000001");
strcpy_s(a.BankID, “1”);
strcpy_s(a.CurrencyID, "CNY");
m_pUserApi->ReqQryTransferSerial(&a, nRequestID++);

```

<a id="7e3e74b0-0c80-4bc0-b6f8-382f9ce5b2ba"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

<a id="region_header_1"></a>

我在柜台上做了手工出入金，通过这个接口查不到？<a id="region_panel_1"></a>

| ReqQryTransferSerial不能查柜台手工出入金，只能查询到期货发起期货转银行或者银行转期货的内容。 |
|---|

<a id="region_tail_1"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
