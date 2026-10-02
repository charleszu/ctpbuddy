# OnErrRtnOrderInsert

OnErrRtnOrderInsert

报单录入错误回报，当执行[ReqOrderInsert](../CTHOSTFTDCTRADERAPI/REQORDERINSERT.html)后有字段填写不对之类的CTP报错则通过此接口返回

详见[报单回调规则](../../QTYWGZ/DBHB.html)

◇ 1. 函数原型

virtual void OnErrRtnOrderInsert(CThostFtdcInputOrderField *pInputOrder, CThostFtdcRspInfoField *pRspInfo) {};

◇ 2. 参数

pInputOrder：输入报单

```
struct CThostFtdcInputOrderField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType  BrokerID;
    ///投资者代码
    TThostFtdcInvestorIDType    InvestorID;
    ///保留的无效字段
    TThostFtdcOldInstrumentIDType   reserve1;
    ///报单引用
    TThostFtdcOrderRefType  OrderRef;
    ///用户代码
    TThostFtdcUserIDType    UserID;
    ///报单价格条件
    TThostFtdcOrderPriceTypeType    OrderPriceType;
    ///买卖方向
    TThostFtdcDirectionType Direction;
    ///组合开平标志
    TThostFtdcCombOffsetFlagType    CombOffsetFlag;
    ///组合投机套保标志
    TThostFtdcCombHedgeFlagType CombHedgeFlag;
    ///价格
    TThostFtdcPriceType LimitPrice;
    ///数量
    TThostFtdcVolumeType    VolumeTotalOriginal;
    ///有效期类型
    TThostFtdcTimeConditionType TimeCondition;
    ///GTD日期
    TThostFtdcDateType  GTDDate;
    ///成交量类型
    TThostFtdcVolumeConditionType   VolumeCondition;
    ///最小成交量
    TThostFtdcVolumeType    MinVolume;
    ///触发条件
    TThostFtdcContingentConditionType   ContingentCondition;
    ///止损价
    TThostFtdcPriceType StopPrice;
    ///强平原因
    TThostFtdcForceCloseReasonType  ForceCloseReason;
    ///自动挂起标志
    TThostFtdcBoolType  IsAutoSuspend;
    ///业务单元
    TThostFtdcBusinessUnitType  BusinessUnit;
    ///请求编号
    TThostFtdcRequestIDType RequestID;
    ///用户强平标志
    TThostFtdcBoolType  UserForceClose;
    ///互换单标志
    TThostFtdcBoolType  IsSwapOrder;
    ///交易所代码
    TThostFtdcExchangeIDType    ExchangeID;
    ///投资单元代码
    TThostFtdcInvestUnitIDType  InvestUnitID;
    ///资金账号
    TThostFtdcAccountIDType AccountID;
    ///币种代码
    TThostFtdcCurrencyIDType    CurrencyID;
    ///交易编码
    TThostFtdcClientIDType  ClientID;
    ///保留的无效字段
    TThostFtdcOldIPAddressType  reserve2;
    ///Mac地址
    TThostFtdcMacAddressType    MacAddress;
    ///合约代码
    TThostFtdcInstrumentIDType  InstrumentID;
    ///IP地址
    TThostFtdcIPAddressType IPAddress;
    ///报单回显字段
    TThostFtdcOrderMemoType OrderMemo;
    ///session上请求计数 api自动维护
    TThostFtdcSequenceNo12Type  SessionReqSeq;
};

```

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

◇ 3. 返回

当查询无记录时，指针返回为null

◇ 4. FAQ

无
