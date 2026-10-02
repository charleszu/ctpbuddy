# OnRspQryAccountregister

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryAccountregister<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询银期签约关系响应，当执行[ReqQryAccountregister](pages/089-JYJK-CTHOSTFTDCTRADERAPI-REQQRYACCOUNTREGISTER.html.md)后，该方法被调用
<a id="ec9b8a72-2ccb-44e2-b448-1f4fb2f10d84"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryAccountregister(CThostFtdcAccountregisterField *pAccountregister, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="15b5c3df-0748-4c43-8c86-fea958a932f9"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pAccountregister：客户开销户信息表

```
struct CThostFtdcAccountregisterField
{
    ///交易日期
    TThostFtdcTradeDateType TradeDay;
    ///银行编码
    TThostFtdcBankIDType BankID;
    ///银行分支机构编码
    TThostFtdcBankBrchIDType BankBranchID;
    ///银行帐号
    TThostFtdcBankAccountType BankAccount;
    ///期货公司编码
    TThostFtdcBrokerIDType BrokerID;
    ///期货公司分支机构编码
    TThostFtdcFutureBranchIDType BrokerBranchID;
    ///投资者帐号
    TThostFtdcAccountIDType AccountID;
    ///证件类型
    TThostFtdcIdCardTypeType IdCardType;
    ///证件号码
    TThostFtdcIdentifiedCardNoType IdentifiedCardNo;
    ///客户姓名
    TThostFtdcIndividualNameType CustomerName;
    ///币种代码
    TThostFtdcCurrencyIDType CurrencyID;
    ///开销户类别
    TThostFtdcOpenOrDestroyType OpenOrDestroy;
    ///签约日期
    TThostFtdcTradeDateType RegDate;
    ///解约日期
    TThostFtdcTradeDateType OutDate;
    ///交易ID
    TThostFtdcTIDType TID;
    ///客户类型
    TThostFtdcCustTypeType CustType;
    ///银行帐号类型
    TThostFtdcBankAccTypeType BankAccType;
    ///长客户姓名
    TThostFtdcLongIndividualNameType LongCustomerName;
};

```

BankID：银行编码

BankBranchID：值为0000

BrokerBranchID：值为0000

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

<a id="2776c735-d558-447b-8614-01a906fbefb2"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="172b9f77-7072-407f-b614-1967507c710f"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
