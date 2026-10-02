# OnRspQryContractBank

OnRspQryContractBank

请求查询签约银行响应，当执行[ReqQryContractBank](../CTHOSTFTDCTRADERAPI/REQQRYCONTRACTBANK.html)后，该方法被调用。

◇ 1. 函数原型

virtual void OnRspQryContractBank(CThostFtdcContractBankField *pContractBank, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

◇ 2. 参数

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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
