# OnRspQryTransferSerial

OnRspQryTransferSerial

请求查询转帐流水响应，当执行[ReqQryTransferSerial](../CTHOSTFTDCTRADERAPI/REQQRYTRANSFERSERIAL.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryTransferSerial(CThostFtdcTransferSerialField *pTransferSerial, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

pTransferSerial：银期转账交易流水表

```
struct CThostFtdcTransferSerialField
{
    ///平台流水号
    TThostFtdcPlateSerialType PlateSerial;
    ///交易发起方日期
    TThostFtdcTradeDateType TradeDate;
    ///交易日期
    TThostFtdcDateType TradingDay;
    ///交易时间
    TThostFtdcTradeTimeType TradeTime;
    ///交易代码
    TThostFtdcTradeCodeType TradeCode;
    ///会话编号
    TThostFtdcSessionIDType SessionID;
    ///银行编码
    TThostFtdcBankIDType BankID;
    ///银行分支机构编码
    TThostFtdcBankBrchIDType BankBranchID;
    ///银行帐号类型
    TThostFtdcBankAccTypeType BankAccType;
    ///银行帐号
    TThostFtdcBankAccountType BankAccount;
    ///银行流水号
    TThostFtdcBankSerialType BankSerial;
    ///期货公司编码
    TThostFtdcBrokerIDType BrokerID;
    ///期商分支机构代码
    TThostFtdcFutureBranchIDType BrokerBranchID;
    ///期货公司帐号类型
    TThostFtdcFutureAccTypeType FutureAccType;
    ///投资者帐号
    TThostFtdcAccountIDType AccountID;
    ///投资者代码
    TThostFtdcInvestorIDType InvestorID;
    ///期货公司流水号
    TThostFtdcFutureSerialType FutureSerial;
    ///证件类型
    TThostFtdcIdCardTypeType IdCardType;
    ///证件号码
    TThostFtdcIdentifiedCardNoType IdentifiedCardNo;
    ///币种代码
    TThostFtdcCurrencyIDType CurrencyID;
    ///交易金额
    TThostFtdcTradeAmountType TradeAmount;
    ///应收客户费用
    TThostFtdcCustFeeType CustFee;
    ///应收期货公司费用
    TThostFtdcFutureFeeType BrokerFee;
    ///有效标志
    TThostFtdcAvailabilityFlagType AvailabilityFlag;
    ///操作员
    TThostFtdcOperatorCodeType OperatorCode;
    ///新银行帐号
    TThostFtdcBankAccountType BankNewAccount;
    ///错误代码
    TThostFtdcErrorIDType ErrorID;
    ///错误信息
    TThostFtdcErrorMsgType ErrorMsg;
};

```

BankBranchID：填0000

BrokerBranchID：填0000

AvailabilityFlag：是否转账成功，不成功则CTP会自动发起冲正

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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
