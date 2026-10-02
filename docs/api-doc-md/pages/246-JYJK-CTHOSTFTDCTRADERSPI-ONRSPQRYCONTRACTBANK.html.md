# OnRspQryContractBank

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryContractBank<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询签约银行响应，当执行[ReqQryContractBank](pages/095-JYJK-CTHOSTFTDCTRADERAPI-REQQRYCONTRACTBANK.html.md)后，该方法被调用。
<a id="1470e2f9-b157-484f-8465-ced960a872ea"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryContractBank(CThostFtdcContractBankField *pContractBank, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="4dd07e50-8ae9-4ca2-84bd-653d481743e0"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pContractBank：查询签约银行响应

```
struct CThostFtdcContractBankField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///银行代码
    TThostFtdcBankIDType BankID;
    ///银行分中心代码
    TThostFtdcBankBrchIDType BankBrchID;
    ///银行名称
    TThostFtdcBankNameType BankName;
    ///上报csrc的银行代码
    TThostFtdcBankIDType    csrcBankID;
};

```

BankID：对应期货公司内部设置的银行编码

BankName：对应期货公司内部设置的银行名称

pRspInfo：响应信息

```
struct CThostFtdcRspInfoField
{
    ///错误代码
    TThostFtdcErrorIDType ErrorID;
    ///错误信息
    TThostFtdcErrorMsgType ErrorMsg;
};

```

nRequestID：返回用户操作请求的ID，该ID 由用户在操作请求时指定。

bIsLast：指示该次返回是否为针对nRequestID的最后一次返回。

<a id="95eb21d1-e86a-400c-97df-f60ab5879d36"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="ab528ba9-887e-47fd-9c87-d381f6ff9dfc"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
