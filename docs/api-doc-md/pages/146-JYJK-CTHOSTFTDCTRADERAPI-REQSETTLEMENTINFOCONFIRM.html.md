# ReqSettlementInfoConfirm

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqSettlementInfoConfirm<a id="content"></a>

<a id="left_menu"></a>

  ** **

投资者结算结果确认，在开始每日交易前都需要先确认上一日结算单，只需要确认一次。对应响应[OnRspSettlementInfoConfirm](pages/296-JYJK-CTHOSTFTDCTRADERSPI-ONRSPSETTLEMENTINFOCONFIRM.html.md)。
<a id="ff8e6774-6c9d-4992-b0f5-daa3c175aca2"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqSettlementInfoConfirm(CThostFtdcSettlementInfoConfirmField *pSettlementInfoConfirm, int nRequestID) = 0;

<a id="8037dc0a-d89e-4324-a31f-20e22ed686fb"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pSettlementInfoConfirm：投资者结算结果确认信息

| 字段类型 | 字段名称 | 含义 | 是否可作为过滤条件 |
|---|---|---|---|
| TThostFtdcBrokerIDType | BrokerID | 经纪公司代码 | 是 |
| TThostFtdcInvestorIDType | InvestorID | 投资者代码 | 是 |
| TThostFtdcDateType | ConfirmDate | 确认日期 | 否 |
| TThostFtdcTimeType | ConfirmTime | 确认时间 | 否 |
| TThostFtdcAccountIDType | AccountID | 投资者帐号 | 否 |
| TThostFtdcCurrencyIDType | CurrencyID | 币种代码 | 否 |
| TThostFtdcSettlementIDType | SettlementID | 结算编号 | 否 |

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="2b008d28-96f1-4620-a798-18026622ad9f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="472e7d0c-770f-447d-97ca-c4a08d74dca3"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcSettlementInfoConfirmField Confirm = { 0 };
strcpy_s(Confirm.BrokerID, "9999");
strcpy_s(Confirm.InvestorID, "1000001");
m_pUserApi->ReqSettlementInfoConfirm(&Confirm, nRequestID++);

```

<a id="3eae68e4-c089-4a26-aa7c-225bd69f06e4"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
