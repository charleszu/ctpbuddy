# ReqFromFutureToBankByFuture

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

ReqFromFutureToBankByFuture<a id="content"></a>

<a id="left_menu"></a>

  ** **

期货发起期货资金转银行请求

错误响应: [OnRspFromFutureToBankByFuture](pages/231-JYJK-CTHOSTFTDCTRADERSPI-ONRSPFROMFUTURETOBANKBYFUTURE.html.md)

正确响应: [OnRtnFromFutureToBankByFuture](pages/313-JYJK-CTHOSTFTDCTRADERSPI-ONRTNFROMFUTURETOBANKBYFUTURE.html.md)
<a id="9b059a85-416c-4382-a4af-f2e882b20efa"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual int ReqFromFutureToBankByFuture(CThostFtdcReqTransferField *pReqTransfer, int nRequestID) = 0;

<a id="dc7f5b68-bc6b-465b-a20d-c4b4cc70b5f4"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pReqTransfer：转账请求

```
struct CThostFtdcReqTransferField
{
    ///业务功能码
    TThostFtdcTradeCodeType TradeCode;
    ///银行代码
    TThostFtdcBankIDType BankID;
    ///银行分支机构代码
    TThostFtdcBankBrchIDType BankBranchID;
    ///期商代码
    TThostFtdcBrokerIDType BrokerID;
    ///期商分支机构代码
    TThostFtdcFutureBranchIDType BrokerBranchID;
    ///交易日期
    TThostFtdcTradeDateType TradeDate;
    ///交易时间
    TThostFtdcTradeTimeType TradeTime;
    ///银行流水号
    TThostFtdcBankSerialType BankSerial;
    ///交易系统日期
    TThostFtdcTradeDateType TradingDay;
    ///银期平台消息流水号
    TThostFtdcSerialType PlateSerial;
    ///最后分片标志
    TThostFtdcLastFragmentType LastFragment;
    ///会话号
    TThostFtdcSessionIDType SessionID;
    ///客户姓名
    TThostFtdcIndividualNameType CustomerName;
    ///证件类型
    TThostFtdcIdCardTypeType IdCardType;
    ///证件号码
    TThostFtdcIdentifiedCardNoType IdentifiedCardNo;
    ///客户类型
    TThostFtdcCustTypeType CustType;
    ///银行帐号
    TThostFtdcBankAccountType BankAccount;
    ///银行密码
    TThostFtdcPasswordType BankPassWord;
    ///投资者帐号
    TThostFtdcAccountIDType AccountID;
    ///期货密码
    TThostFtdcPasswordType Password;
    ///安装编号
    TThostFtdcInstallIDType InstallID;
    ///期货公司流水号
    TThostFtdcFutureSerialType FutureSerial;
    ///用户标识
    TThostFtdcUserIDType UserID;
    ///验证客户证件号码标志
    TThostFtdcYesNoIndicatorType VerifyCertNoFlag;
    ///币种代码
    TThostFtdcCurrencyIDType CurrencyID;
    ///转帐金额
    TThostFtdcTradeAmountType TradeAmount;
    ///期货可取金额
    TThostFtdcTradeAmountType FutureFetchAmount;
    ///费用支付标志
    TThostFtdcFeePayFlagType FeePayFlag;
    ///应收客户费用
    TThostFtdcCustFeeType CustFee;
    ///应收期货公司费用
    TThostFtdcFutureFeeType BrokerFee;
    ///发送方给接收方的消息
    TThostFtdcAddInfoType Message;
    ///摘要
    TThostFtdcDigestType Digest;
    ///银行帐号类型
    TThostFtdcBankAccTypeType BankAccType;
    ///渠道标志
    TThostFtdcDeviceIDType DeviceID;
    ///期货单位帐号类型
    TThostFtdcBankAccTypeType BankSecuAccType;
    ///期货公司银行编码
    TThostFtdcBankCodingForFutureType BrokerIDByBank;
    ///期货单位帐号
    TThostFtdcBankAccountType BankSecuAcc;
    ///银行密码标志
    TThostFtdcPwdFlagType BankPwdFlag;
    ///期货资金密码核对标志
    TThostFtdcPwdFlagType SecuPwdFlag;
    ///交易柜员
    TThostFtdcOperNoType OperNo;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///交易ID
    TThostFtdcTIDType TID;
    ///转账交易状态
    TThostFtdcTransferStatusType TransferStatus;
    ///长客户姓名
    TThostFtdcLongIndividualNameType LongCustomerName;
};

```

TradeCode：202002 期货发起期货资金转银行。

BankID：返回银行在期货公司的代码

BankBranchID：填0000

BrokerBranchID：为空

TradeAmount：转账金额

nRequestID：请求ID，对应响应里的nRequestID，无递增规则，由用户自行维护。

<a id="c6e41611-488c-4fdc-aa46-daf2123392b6"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

0，代表成功。

-1，表示网络连接失败；

-2，表示未处理请求超过许可数；

-3，表示每秒发送请求数超过许可数。

<a id="2be0d204-d0e7-4804-a9b7-0fbdf026e2bc"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. 调用示例
<a id="panel4"></a>

```
CThostFtdcReqTransferField a = { 0 };
strcpy_s(a.TradeCode, "202002");///业务功能码
strcpy_s(a.BankID, "1");
strcpy_s(a.BankBranchID, "0000");///期商代码
strcpy_s(a.BrokerID, "9999");
a.LastFragment = THOST_FTDC_LF_Yes;///最后分片标志 '0'=是最后分片
a.SessionID = SessionID;
a.IdCardType = THOST_FTDC_ICT_IDCard;///证件类型
strcpy_s(a.IdentifiedCardNo, "310110198701011914");///证件号码
strcpy_s(a.BankAccount, "123456789");///银行帐号
strcpy_s(a.AccountID, "1000001");///投资者帐号
strcpy_s(a.Password, "123456");///期货密码
a.InstallID = 1;///安装编号
a.CustType = THOST_FTDC_CUSTT_Person;
a.VerifyCertNoFlag = THOST_FTDC_YNI_No;///验证客户证件号码标志
strcpy_s(a.CurrencyID, "CNY");///币种代码
a.TradeAmount = output_num;///转帐金额
a.FutureFetchAmount = 0;///期货可取金额
a.CustFee = 0;///应收客户费用
a.BrokerFee = 0;///应收期货公司费用
a.RequestID = 0;///请求编号
a.TID = 0;///交易ID
m_pUserApi->ReqFromFutureToBankByFuture(&a, nRequestID++);

```

<a id="461c3e51-5a11-4b1a-8291-844e71a55a63"></a><a id="title5"></a>

<a id="header_span5"></a>◇ 5. FAQ
<a id="panel5"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
