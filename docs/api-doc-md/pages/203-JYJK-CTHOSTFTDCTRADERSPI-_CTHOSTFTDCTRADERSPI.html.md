# CThostFtdcTraderSpi

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

CThostFtdcTraderSpi<a id="content"></a>

<a id="left_menu"></a>

  ** **

|  | ■ 6.7.13_API接口说明
└△ 交易接口
　└◆ CThostFtdcTraderSpi |  |
|---|---|---|

CThostFtdcTraderSpi类提供了交易相关的回调接口，用户需要继承该类并重载这些接口，以获取响应数据。
<a id="661d7fc4-d85c-47eb-a31d-212ba4b43728"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 接口
<a id="panel1"></a>

```
class CThostFtdcTraderSpi
{
public:
    ///当客户端与交易后台建立起通信连接时（还未登录前），该方法被调用。
    virtual void OnFrontConnected(){};
    ///当客户端与交易后台通信连接断开时，该方法被调用。当发生这个情况后，API会自动重新连接，客户端可不做处理。
    ///@param nReason 错误原因
    ///        0x1001 网络读失败
    ///        0x1002 网络写失败
    ///        0x2001 接收心跳超时
    ///        0x2002 发送心跳失败
    ///        0x2003 收到错误报文
    virtual void OnFrontDisconnected(int nReason){};
    ///心跳超时警告。当长时间未收到报文时，该方法被调用。
    ///@param nTimeLapse 距离上次接收报文的时间
    virtual void OnHeartBeatWarning(int nTimeLapse){};
    ///客户端认证响应
    virtual void OnRspAuthenticate(CThostFtdcRspAuthenticateField *pRspAuthenticateField, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///登录请求响应
    virtual void OnRspUserLogin(CThostFtdcRspUserLoginField *pRspUserLogin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///登出请求响应
    virtual void OnRspUserLogout(CThostFtdcUserLogoutField *pUserLogout, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///用户口令更新请求响应
    virtual void OnRspUserPasswordUpdate(CThostFtdcUserPasswordUpdateField *pUserPasswordUpdate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///资金账户口令更新请求响应
    virtual void OnRspTradingAccountPasswordUpdate(CThostFtdcTradingAccountPasswordUpdateField *pTradingAccountPasswordUpdate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///查询用户当前支持的认证模式的回复
    virtual void OnRspUserAuthMethod(CThostFtdcRspUserAuthMethodField *pRspUserAuthMethod, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///获取图形验证码请求的回复
    virtual void OnRspGenUserCaptcha(CThostFtdcRspGenUserCaptchaField *pRspGenUserCaptcha, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///获取短信验证码请求的回复
    virtual void OnRspGenUserText(CThostFtdcRspGenUserTextField *pRspGenUserText, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///报单录入请求响应
    virtual void OnRspOrderInsert(CThostFtdcInputOrderField *pInputOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///预埋单录入请求响应
    virtual void OnRspParkedOrderInsert(CThostFtdcParkedOrderField *pParkedOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///预埋撤单录入请求响应
    virtual void OnRspParkedOrderAction(CThostFtdcParkedOrderActionField *pParkedOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///报单操作请求响应
    virtual void OnRspOrderAction(CThostFtdcInputOrderActionField *pInputOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///查询最大报单数量响应
    virtual void OnRspQryMaxOrderVolume(CThostFtdcQryMaxOrderVolumeField *pQryMaxOrderVolume, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///投资者结算结果确认响应
    virtual void OnRspSettlementInfoConfirm(CThostFtdcSettlementInfoConfirmField *pSettlementInfoConfirm, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///删除预埋单响应
    virtual void OnRspRemoveParkedOrder(CThostFtdcRemoveParkedOrderField *pRemoveParkedOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///删除预埋撤单响应
    virtual void OnRspRemoveParkedOrderAction(CThostFtdcRemoveParkedOrderActionField *pRemoveParkedOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///执行宣告录入请求响应
    virtual void OnRspExecOrderInsert(CThostFtdcInputExecOrderField *pInputExecOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///执行宣告操作请求响应
    virtual void OnRspExecOrderAction(CThostFtdcInputExecOrderActionField *pInputExecOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///询价录入请求响应
    virtual void OnRspForQuoteInsert(CThostFtdcInputForQuoteField *pInputForQuote, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///报价录入请求响应
    virtual void OnRspQuoteInsert(CThostFtdcInputQuoteField *pInputQuote, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///报价操作请求响应
    virtual void OnRspQuoteAction(CThostFtdcInputQuoteActionField *pInputQuoteAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///批量报单操作请求响应
    virtual void OnRspBatchOrderAction(CThostFtdcInputBatchOrderActionField *pInputBatchOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///期权自对冲录入请求响应
    virtual void OnRspOptionSelfCloseInsert(CThostFtdcInputOptionSelfCloseField *pInputOptionSelfClose, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///期权自对冲操作请求响应
    virtual void OnRspOptionSelfCloseAction(CThostFtdcInputOptionSelfCloseActionField *pInputOptionSelfCloseAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///申请组合录入请求响应
    virtual void OnRspCombActionInsert(CThostFtdcInputCombActionField *pInputCombAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询报单响应
    virtual void OnRspQryOrder(CThostFtdcOrderField *pOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询成交响应
    virtual void OnRspQryTrade(CThostFtdcTradeField *pTrade, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询投资者持仓响应
    virtual void OnRspQryInvestorPosition(CThostFtdcInvestorPositionField *pInvestorPosition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询资金账户响应
    virtual void OnRspQryTradingAccount(CThostFtdcTradingAccountField *pTradingAccount, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询投资者响应
    virtual void OnRspQryInvestor(CThostFtdcInvestorField *pInvestor, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询交易编码响应
    virtual void OnRspQryTradingCode(CThostFtdcTradingCodeField *pTradingCode, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询合约保证金率响应
    virtual void OnRspQryInstrumentMarginRate(CThostFtdcInstrumentMarginRateField *pInstrumentMarginRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询合约手续费率响应
    virtual void OnRspQryInstrumentCommissionRate(CThostFtdcInstrumentCommissionRateField *pInstrumentCommissionRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询用户会话响应
    virtual void OnRspQryUserSession(CThostFtdcUserSessionField *pUserSession, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询交易所响应
    virtual void OnRspQryExchange(CThostFtdcExchangeField *pExchange, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询产品响应
    virtual void OnRspQryProduct(CThostFtdcProductField *pProduct, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询合约响应
    virtual void OnRspQryInstrument(CThostFtdcInstrumentField *pInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询行情响应
    virtual void OnRspQryDepthMarketData(CThostFtdcDepthMarketDataField *pDepthMarketData, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询投资者结算结果响应
    virtual void OnRspQrySettlementInfo(CThostFtdcSettlementInfoField *pSettlementInfo, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询转帐银行响应
    virtual void OnRspQryTransferBank(CThostFtdcTransferBankField *pTransferBank, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询投资者持仓明细响应
    virtual void OnRspQryInvestorPositionDetail(CThostFtdcInvestorPositionDetailField *pInvestorPositionDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询客户通知响应
    virtual void OnRspQryNotice(CThostFtdcNoticeField *pNotice, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询结算信息确认响应
    virtual void OnRspQrySettlementInfoConfirm(CThostFtdcSettlementInfoConfirmField *pSettlementInfoConfirm, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询投资者持仓明细响应
    virtual void OnRspQryInvestorPositionCombineDetail(CThostFtdcInvestorPositionCombineDetailField *pInvestorPositionCombineDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///查询保证金监管系统经纪公司资金账户密钥响应
    virtual void OnRspQryCFMMCTradingAccountKey(CThostFtdcCFMMCTradingAccountKeyField *pCFMMCTradingAccountKey, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询仓单折抵信息响应
    virtual void OnRspQryEWarrantOffset(CThostFtdcEWarrantOffsetField *pEWarrantOffset, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询投资者品种/跨品种保证金响应
    virtual void OnRspQryInvestorProductGroupMargin(CThostFtdcInvestorProductGroupMarginField *pInvestorProductGroupMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询交易所保证金率响应
    virtual void OnRspQryExchangeMarginRate(CThostFtdcExchangeMarginRateField *pExchangeMarginRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询交易所调整保证金率响应
    virtual void OnRspQryExchangeMarginRateAdjust(CThostFtdcExchangeMarginRateAdjustField *pExchangeMarginRateAdjust, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询汇率响应
    virtual void OnRspQryExchangeRate(CThostFtdcExchangeRateField *pExchangeRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询二级代理操作员银期权限响应
    virtual void OnRspQrySecAgentACIDMap(CThostFtdcSecAgentACIDMapField *pSecAgentACIDMap, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询产品报价汇率
    virtual void OnRspQryProductExchRate(CThostFtdcProductExchRateField *pProductExchRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询产品组
    virtual void OnRspQryProductGroup(CThostFtdcProductGroupField *pProductGroup, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询做市商合约手续费率响应
    virtual void OnRspQryMMInstrumentCommissionRate(CThostFtdcMMInstrumentCommissionRateField *pMMInstrumentCommissionRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询做市商期权合约手续费响应
    virtual void OnRspQryMMOptionInstrCommRate(CThostFtdcMMOptionInstrCommRateField *pMMOptionInstrCommRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询报单手续费响应
    virtual void OnRspQryInstrumentOrderCommRate(CThostFtdcInstrumentOrderCommRateField *pInstrumentOrderCommRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询资金账户响应
    virtual void OnRspQrySecAgentTradingAccount(CThostFtdcTradingAccountField *pTradingAccount, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询二级代理商资金校验模式响应
    virtual void OnRspQrySecAgentCheckMode(CThostFtdcSecAgentCheckModeField *pSecAgentCheckMode, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询二级代理商信息响应
    virtual void OnRspQrySecAgentTradeInfo(CThostFtdcSecAgentTradeInfoField *pSecAgentTradeInfo, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询期权交易成本响应
    virtual void OnRspQryOptionInstrTradeCost(CThostFtdcOptionInstrTradeCostField *pOptionInstrTradeCost, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询期权合约手续费响应
    virtual void OnRspQryOptionInstrCommRate(CThostFtdcOptionInstrCommRateField *pOptionInstrCommRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询执行宣告响应
    virtual void OnRspQryExecOrder(CThostFtdcExecOrderField *pExecOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询询价响应
    virtual void OnRspQryForQuote(CThostFtdcForQuoteField *pForQuote, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询报价响应
    virtual void OnRspQryQuote(CThostFtdcQuoteField *pQuote, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询期权自对冲响应
    virtual void OnRspQryOptionSelfClose(CThostFtdcOptionSelfCloseField *pOptionSelfClose, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询投资单元响应
    virtual void OnRspQryInvestUnit(CThostFtdcInvestUnitField *pInvestUnit, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询组合合约安全系数响应
    virtual void OnRspQryCombInstrumentGuard(CThostFtdcCombInstrumentGuardField *pCombInstrumentGuard, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询申请组合响应
    virtual void OnRspQryCombAction(CThostFtdcCombActionField *pCombAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询转帐流水响应
    virtual void OnRspQryTransferSerial(CThostFtdcTransferSerialField *pTransferSerial, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询银期签约关系响应
    virtual void OnRspQryAccountregister(CThostFtdcAccountregisterField *pAccountregister, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///错误应答
    virtual void OnRspError(CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///报单通知
    virtual void OnRtnOrder(CThostFtdcOrderField *pOrder) {};
    ///成交通知
    virtual void OnRtnTrade(CThostFtdcTradeField *pTrade) {};
    ///报单录入错误回报
    virtual void OnErrRtnOrderInsert(CThostFtdcInputOrderField *pInputOrder, CThostFtdcRspInfoField *pRspInfo) {};
    ///报单操作错误回报
    virtual void OnErrRtnOrderAction(CThostFtdcOrderActionField *pOrderAction, CThostFtdcRspInfoField *pRspInfo) {};
    ///合约交易状态通知
    virtual void OnRtnInstrumentStatus(CThostFtdcInstrumentStatusField *pInstrumentStatus) {};
    ///交易所公告通知
    virtual void OnRtnBulletin(CThostFtdcBulletinField *pBulletin) {};
    ///交易通知
    virtual void OnRtnTradingNotice(CThostFtdcTradingNoticeInfoField *pTradingNoticeInfo) {};
    ///提示条件单校验错误
    virtual void OnRtnErrorConditionalOrder(CThostFtdcErrorConditionalOrderField *pErrorConditionalOrder) {};
    ///执行宣告通知
    virtual void OnRtnExecOrder(CThostFtdcExecOrderField *pExecOrder) {};
    ///执行宣告录入错误回报
    virtual void OnErrRtnExecOrderInsert(CThostFtdcInputExecOrderField *pInputExecOrder, CThostFtdcRspInfoField *pRspInfo) {};
    ///执行宣告操作错误回报
    virtual void OnErrRtnExecOrderAction(CThostFtdcExecOrderActionField *pExecOrderAction, CThostFtdcRspInfoField *pRspInfo) {};
    ///询价录入错误回报
    virtual void OnErrRtnForQuoteInsert(CThostFtdcInputForQuoteField *pInputForQuote, CThostFtdcRspInfoField *pRspInfo) {};
    ///报价通知
    virtual void OnRtnQuote(CThostFtdcQuoteField *pQuote) {};
    ///报价录入错误回报
    virtual void OnErrRtnQuoteInsert(CThostFtdcInputQuoteField *pInputQuote, CThostFtdcRspInfoField *pRspInfo) {};
    ///报价操作错误回报
    virtual void OnErrRtnQuoteAction(CThostFtdcQuoteActionField *pQuoteAction, CThostFtdcRspInfoField *pRspInfo) {};
    ///询价通知
    virtual void OnRtnForQuoteRsp(CThostFtdcForQuoteRspField *pForQuoteRsp) {};
    ///保证金监控中心用户令牌
    virtual void OnRtnCFMMCTradingAccountToken(CThostFtdcCFMMCTradingAccountTokenField *pCFMMCTradingAccountToken) {};
    ///批量报单操作错误回报
    virtual void OnErrRtnBatchOrderAction(CThostFtdcBatchOrderActionField *pBatchOrderAction, CThostFtdcRspInfoField *pRspInfo) {};
    ///期权自对冲通知
    virtual void OnRtnOptionSelfClose(CThostFtdcOptionSelfCloseField *pOptionSelfClose) {};
    ///期权自对冲录入错误回报
    virtual void OnErrRtnOptionSelfCloseInsert(CThostFtdcInputOptionSelfCloseField *pInputOptionSelfClose, CThostFtdcRspInfoField *pRspInfo) {};
    ///期权自对冲操作错误回报
    virtual void OnErrRtnOptionSelfCloseAction(CThostFtdcOptionSelfCloseActionField *pOptionSelfCloseAction, CThostFtdcRspInfoField *pRspInfo) {};
    ///申请组合通知
    virtual void OnRtnCombAction(CThostFtdcCombActionField *pCombAction) {};
    ///申请组合录入错误回报
    virtual void OnErrRtnCombActionInsert(CThostFtdcInputCombActionField *pInputCombAction, CThostFtdcRspInfoField *pRspInfo) {};
    ///请求查询签约银行响应
    virtual void OnRspQryContractBank(CThostFtdcContractBankField *pContractBank, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询预埋单响应
    virtual void OnRspQryParkedOrder(CThostFtdcParkedOrderField *pParkedOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询预埋撤单响应
    virtual void OnRspQryParkedOrderAction(CThostFtdcParkedOrderActionField *pParkedOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询交易通知响应
    virtual void OnRspQryTradingNotice(CThostFtdcTradingNoticeField *pTradingNotice, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询经纪公司交易参数响应
    virtual void OnRspQryBrokerTradingParams(CThostFtdcBrokerTradingParamsField *pBrokerTradingParams, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询经纪公司交易算法响应
    virtual void OnRspQryBrokerTradingAlgos(CThostFtdcBrokerTradingAlgosField *pBrokerTradingAlgos, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求查询监控中心用户令牌
    virtual void OnRspQueryCFMMCTradingAccountToken(CThostFtdcQueryCFMMCTradingAccountTokenField *pQueryCFMMCTradingAccountToken, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///银行发起银行资金转期货通知
    virtual void OnRtnFromBankToFutureByBank(CThostFtdcRspTransferField *pRspTransfer) {};
    ///银行发起期货资金转银行通知
    virtual void OnRtnFromFutureToBankByBank(CThostFtdcRspTransferField *pRspTransfer) {};
    ///银行发起冲正银行转期货通知
    virtual void OnRtnRepealFromBankToFutureByBank(CThostFtdcRspRepealField *pRspRepeal) {};
    ///银行发起冲正期货转银行通知
    virtual void OnRtnRepealFromFutureToBankByBank(CThostFtdcRspRepealField *pRspRepeal) {};
    ///期货发起银行资金转期货通知
    virtual void OnRtnFromBankToFutureByFuture(CThostFtdcRspTransferField *pRspTransfer) {};
    ///期货发起期货资金转银行通知
    virtual void OnRtnFromFutureToBankByFuture(CThostFtdcRspTransferField *pRspTransfer) {};
    ///系统运行时期货端手工发起冲正银行转期货请求，银行处理完毕后报盘发回的通知
    virtual void OnRtnRepealFromBankToFutureByFutureManual(CThostFtdcRspRepealField *pRspRepeal) {};
    ///系统运行时期货端手工发起冲正期货转银行请求，银行处理完毕后报盘发回的通知
    virtual void OnRtnRepealFromFutureToBankByFutureManual(CThostFtdcRspRepealField *pRspRepeal) {};
    ///期货发起查询银行余额通知
    virtual void OnRtnQueryBankBalanceByFuture(CThostFtdcNotifyQueryAccountField *pNotifyQueryAccount) {};
    ///期货发起银行资金转期货错误回报
    virtual void OnErrRtnBankToFutureByFuture(CThostFtdcReqTransferField *pReqTransfer, CThostFtdcRspInfoField *pRspInfo) {};
    ///期货发起期货资金转银行错误回报
    virtual void OnErrRtnFutureToBankByFuture(CThostFtdcReqTransferField *pReqTransfer, CThostFtdcRspInfoField *pRspInfo) {};
    ///系统运行时期货端手工发起冲正银行转期货错误回报
    virtual void OnErrRtnRepealBankToFutureByFutureManual(CThostFtdcReqRepealField *pReqRepeal, CThostFtdcRspInfoField *pRspInfo) {};
    ///系统运行时期货端手工发起冲正期货转银行错误回报
    virtual void OnErrRtnRepealFutureToBankByFutureManual(CThostFtdcReqRepealField *pReqRepeal, CThostFtdcRspInfoField *pRspInfo) {};
    ///期货发起查询银行余额错误回报
    virtual void OnErrRtnQueryBankBalanceByFuture(CThostFtdcReqQueryAccountField *pReqQueryAccount, CThostFtdcRspInfoField *pRspInfo) {};
    ///期货发起冲正银行转期货请求，银行处理完毕后报盘发回的通知
    virtual void OnRtnRepealFromBankToFutureByFuture(CThostFtdcRspRepealField *pRspRepeal) {};
    ///期货发起冲正期货转银行请求，银行处理完毕后报盘发回的通知
    virtual void OnRtnRepealFromFutureToBankByFuture(CThostFtdcRspRepealField *pRspRepeal) {};
    ///期货发起银行资金转期货应答
    virtual void OnRspFromBankToFutureByFuture(CThostFtdcReqTransferField *pReqTransfer, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///期货发起期货资金转银行应答
    virtual void OnRspFromFutureToBankByFuture(CThostFtdcReqTransferField *pReqTransfer, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///期货发起查询银行余额应答
    virtual void OnRspQueryBankAccountMoneyByFuture(CThostFtdcReqQueryAccountField *pReqQueryAccount, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///银行发起银期开户通知
    virtual void OnRtnOpenAccountByBank(CThostFtdcOpenAccountField *pOpenAccount) {};
    ///银行发起银期销户通知
    virtual void OnRtnCancelAccountByBank(CThostFtdcCancelAccountField *pCancelAccount) {};
    ///银行发起变更银行账号通知
    virtual void OnRtnChangeAccountByBank(CThostFtdcChangeAccountField *pChangeAccount) {};
    ///请求查询分类合约响应
    virtual void OnRspQryClassifiedInstrument(CThostFtdcInstrumentField *pInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///请求组合优惠比例响应
    virtual void OnRspQryCombPromotionParam(CThostFtdcCombPromotionParamField *pCombPromotionParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///SPBM期货合约参数查询响应
    virtual void OnRspQrySPBMFutureParameter(CThostFtdcSPBMFutureParameterField *pSPBMFutureParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///SPBM期权合约参数查询响应
    virtual void OnRspQrySPBMOptionParameter(CThostFtdcSPBMOptionParameterField *pSPBMOptionParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///SPBM品种内对锁仓折扣参数查询响应
    virtual void OnRspQrySPBMIntraParameter(CThostFtdcSPBMIntraParameterField *pSPBMIntraParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///SPBM跨品种抵扣参数查询响应
    virtual void OnRspQrySPBMInterParameter(CThostFtdcSPBMInterParameterField *pSPBMInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///SPBM组合保证金套餐查询响应
    virtual void OnRspQrySPBMPortfDefinition(CThostFtdcSPBMPortfDefinitionField *pSPBMPortfDefinition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///投资者SPBM套餐选择查询响应
    virtual void OnRspQrySPBMInvestorPortfDef(CThostFtdcSPBMInvestorPortfDefField *pSPBMInvestorPortfDef, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///投资者新型组合保证金系数查询响应
    virtual void OnRspQryInvestorPortfMarginRatio(CThostFtdcInvestorPortfMarginRatioField *pInvestorPortfMarginRatio, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///投资者产品SPBM明细查询响应
    virtual void OnRspQryInvestorProdSPBMDetail(CThostFtdcInvestorProdSPBMDetailField *pInvestorProdSPBMDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///投资者商品组SPMM记录查询响应
    virtual void OnRspQryInvestorCommoditySPMMMargin(CThostFtdcInvestorCommoditySPMMMarginField *pInvestorCommoditySPMMMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///投资者商品群SPMM记录查询响应
    virtual void OnRspQryInvestorCommodityGroupSPMMMargin(CThostFtdcInvestorCommodityGroupSPMMMarginField *pInvestorCommodityGroupSPMMMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///SPMM合约参数查询响应
    virtual void OnRspQrySPMMInstParam(CThostFtdcSPMMInstParamField *pSPMMInstParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///SPMM产品参数查询响应
    virtual void OnRspQrySPMMProductParam(CThostFtdcSPMMProductParamField *pSPMMProductParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///SPBM附加跨品种抵扣参数查询响应
    virtual void OnRspQrySPBMAddOnInterParameter(CThostFtdcSPBMAddOnInterParameterField *pSPBMAddOnInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///RCAMS产品组合信息查询响应
    virtual void OnRspQryRCAMSCombProductInfo(CThostFtdcRCAMSCombProductInfoField *pRCAMSCombProductInfo, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///RCAMS同合约风险对冲参数查询响应
    virtual void OnRspQryRCAMSInstrParameter(CThostFtdcRCAMSInstrParameterField *pRCAMSInstrParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///RCAMS品种内风险对冲参数查询响应
    virtual void OnRspQryRCAMSIntraParameter(CThostFtdcRCAMSIntraParameterField *pRCAMSIntraParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///RCAMS跨品种风险折抵参数查询响应
    virtual void OnRspQryRCAMSInterParameter(CThostFtdcRCAMSInterParameterField *pRCAMSInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///RCAMS空头期权风险调整参数查询响应
    virtual void OnRspQryRCAMSShortOptAdjustParam(CThostFtdcRCAMSShortOptAdjustParamField *pRCAMSShortOptAdjustParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///RCAMS策略组合持仓查询响应
    virtual void OnRspQryRCAMSInvestorCombPosition(CThostFtdcRCAMSInvestorCombPositionField *pRCAMSInvestorCombPosition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///投资者品种RCAMS保证金查询响应
    virtual void OnRspQryInvestorProdRCAMSMargin(CThostFtdcInvestorProdRCAMSMarginField *pInvestorProdRCAMSMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///RULE合约保证金参数查询响应
    virtual void OnRspQryRULEInstrParameter(CThostFtdcRULEInstrParameterField *pRULEInstrParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///RULE品种内对锁仓折扣参数查询响应
    virtual void OnRspQryRULEIntraParameter(CThostFtdcRULEIntraParameterField *pRULEIntraParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///RULE跨品种抵扣参数查询响应
    virtual void OnRspQryRULEInterParameter(CThostFtdcRULEInterParameterField *pRULEInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///投资者产品RULE保证金查询响应
    virtual void OnRspQryInvestorProdRULEMargin(CThostFtdcInvestorProdRULEMarginField *pInvestorProdRULEMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///投资者投资者新组保设置查询响应
    virtual void OnRspQryInvestorPortfSetting(CThostFtdcInvestorPortfSettingField *pInvestorPortfSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///投资者申报费阶梯收取记录查询响应
    virtual void OnRspQryInvestorInfoCommRec(CThostFtdcInvestorInfoCommRecField *pInvestorInfoCommRec, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///组合腿信息查询响应
    virtual void OnRspQryCombLeg(CThostFtdcCombLegField *pCombLeg, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///对冲设置请求响应
    virtual void OnRspOffsetSetting(CThostFtdcInputOffsetSettingField *pInputOffsetSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///对冲设置撤销请求响应
    virtual void OnRspCancelOffsetSetting(CThostFtdcInputOffsetSettingField *pInputOffsetSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
    ///对冲设置通知
    virtual void OnRtnOffsetSetting(CThostFtdcOffsetSettingField *pOffsetSetting) {};
    ///对冲设置错误回报
    virtual void OnErrRtnOffsetSetting(CThostFtdcInputOffsetSettingField *pInputOffsetSetting, CThostFtdcRspInfoField *pRspInfo) {};
    ///对冲设置撤销错误回报
    virtual void OnErrRtnCancelOffsetSetting(CThostFtdcCancelOffsetSettingField *pCancelOffsetSetting, CThostFtdcRspInfoField *pRspInfo) {};
    ///投资者对冲设置查询响应
    virtual void OnRspQryOffsetSetting(CThostFtdcOffsetSettingField *pOffsetSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};
};

```

<a id="76ec0c04-8462-4fc1-be82-fa7a5e3a8128"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 代码示例
<a id="panel2"></a>

```
// CTraderHandler继承CThostFtdcTraderSpi
class CTraderHandler : public CThostFtdcTraderSpi
{
  //重载，报单通知
  void OnRtnOrder(CThostFtdcOrderField *pOrder)
  {
      printf("OnRtnOrder\n");
  }
  //重载，成交通知
  void OnRtnTrade(CThostFtdcTradeField *pTrade)
  {
      printf("OnRtnTrade\n");
  }
  //重载，报单录入请求响应
  void OnRspOrderInsert(CThostFtdcInputOrderField *pInputOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
  {
      printf("OnRspOrderInsert\n");
  }
  //重载，错误应答
  void OnRspError(CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast)
  {
      printf("OnRspError\n");
  }
}

```

<a id="2c429655-2372-41bf-ac1e-a46b2f29cc1e"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3.FAQ
<a id="panel3"></a>

无

请参阅：

    [OnErrRtnBankToFutureByFuture](pages/204-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNBANKTOFUTUREBYFUTURE.html.md)

    [OnErrRtnBatchOrderAction](pages/205-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNBATCHORDERACTION.html.md)

    [OnErrRtnCombActionInsert](pages/206-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNCOMBACTIONINSERT.html.md)

    [OnErrRtnExecOrderAction](pages/207-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNEXECORDERACTION.html.md)

    [OnErrRtnExecOrderInsert](pages/208-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNEXECORDERINSERT.html.md)

    [OnErrRtnForQuoteInsert](pages/209-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNFORQUOTEINSERT.html.md)

    [OnErrRtnFutureToBankByFuture](pages/210-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNFUTURETOBANKBYFUTURE.html.md)

    [OnErrRtnOptionSelfCloseAction](pages/211-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNOPTIONSELFCLOSEACTION.html.md)

    [OnErrRtnOptionSelfCloseInsert](pages/212-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNOPTIONSELFCLOSEINSERT.html.md)

    [OnErrRtnOrderAction](pages/213-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNORDERACTION.html.md)

    [OnErrRtnOrderInsert](pages/214-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNORDERINSERT.html.md)

    [OnErrRtnQueryBankBalanceByFuture](pages/215-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNQUERYBANKBALANCEBYFUTURE.html.md)

    [OnErrRtnQuoteAction](pages/216-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNQUOTEACTION.html.md)

    [OnErrRtnQuoteInsert](pages/217-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNQUOTEINSERT.html.md)

    [OnErrRtnRepealBankToFutureByFutureManual](pages/218-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNREPEALBANKTOFUTUREBYFUTUREMANUAL.html.md)

    [OnErrRtnRepealFutureToBankByFutureManual](pages/219-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNREPEALFUTURETOBANKBYFUTUREMANUAL.html.md)

    [OnFrontConnected](pages/220-JYJK-CTHOSTFTDCTRADERSPI-ONFRONTCONNECTED.html.md)

    [OnFrontDisconnected](pages/221-JYJK-CTHOSTFTDCTRADERSPI-ONFRONTDISCONNECTED.html.md)

    [OnHeartBeatWarning](pages/222-JYJK-CTHOSTFTDCTRADERSPI-ONHEARTBEATWARNING.html.md)

    [OnRspAuthenticate](pages/223-JYJK-CTHOSTFTDCTRADERSPI-ONRSPAUTHENTICATE.html.md)

    [OnRspBatchOrderAction](pages/224-JYJK-CTHOSTFTDCTRADERSPI-ONRSPBATCHORDERACTION.html.md)

    [OnRspCombActionInsert](pages/225-JYJK-CTHOSTFTDCTRADERSPI-ONRSPCOMBACTIONINSERT.html.md)

    [OnRspError](pages/226-JYJK-CTHOSTFTDCTRADERSPI-ONRSPERROR.html.md)

    [OnRspExecOrderAction](pages/227-JYJK-CTHOSTFTDCTRADERSPI-ONRSPEXECORDERACTION.html.md)

    [OnRspExecOrderInsert](pages/228-JYJK-CTHOSTFTDCTRADERSPI-ONRSPEXECORDERINSERT.html.md)

    [OnRspForQuoteInsert](pages/229-JYJK-CTHOSTFTDCTRADERSPI-ONRSPFORQUOTEINSERT.html.md)

    [OnRspFromBankToFutureByFuture](pages/230-JYJK-CTHOSTFTDCTRADERSPI-ONRSPFROMBANKTOFUTUREBYFUTURE.html.md)

    [OnRspFromFutureToBankByFuture](pages/231-JYJK-CTHOSTFTDCTRADERSPI-ONRSPFROMFUTURETOBANKBYFUTURE.html.md)

    [OnRspGenUserCaptcha](pages/232-JYJK-CTHOSTFTDCTRADERSPI-ONRSPGENUSERCAPTCHA.html.md)

    [OnRspGenUserText](pages/233-JYJK-CTHOSTFTDCTRADERSPI-ONRSPGENUSERTEXT.html.md)

    [OnRspOptionSelfCloseAction](pages/234-JYJK-CTHOSTFTDCTRADERSPI-ONRSPOPTIONSELFCLOSEACTION.html.md)

    [OnRspOptionSelfCloseInsert](pages/235-JYJK-CTHOSTFTDCTRADERSPI-ONRSPOPTIONSELFCLOSEINSERT.html.md)

    [OnRspOrderAction](pages/236-JYJK-CTHOSTFTDCTRADERSPI-ONRSPORDERACTION.html.md)

    [OnRspOrderInsert](pages/237-JYJK-CTHOSTFTDCTRADERSPI-ONRSPORDERINSERT.html.md)

    [OnRspParkedOrderAction](pages/238-JYJK-CTHOSTFTDCTRADERSPI-ONRSPPARKEDORDERACTION.html.md)

    [OnRspParkedOrderInsert](pages/239-JYJK-CTHOSTFTDCTRADERSPI-ONRSPPARKEDORDERINSERT.html.md)

    [OnRspQryAccountregister](pages/240-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYACCOUNTREGISTER.html.md)

    [OnRspQryBrokerTradingAlgos](pages/241-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYBROKERTRADINGALGOS.html.md)

    [OnRspQryBrokerTradingParams](pages/242-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYBROKERTRADINGPARAMS.html.md)

    [OnRspQryCFMMCTradingAccountKey](pages/243-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCFMMCTRADINGACCOUNTKEY.html.md)

    [OnRspQryCombAction](pages/244-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCOMBACTION.html.md)

    [OnRspQryCombInstrumentGuard](pages/245-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCOMBINSTRUMENTGUARD.html.md)

    [OnRspQryContractBank](pages/246-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCONTRACTBANK.html.md)

    [OnRspQryDepthMarketData](pages/247-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYDEPTHMARKETDATA.html.md)

    [OnRspQryEWarrantOffset](pages/248-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYEWARRANTOFFSET.html.md)

    [OnRspQryExchange](pages/249-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYEXCHANGE.html.md)

    [OnRspQryExchangeMarginRate](pages/250-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYEXCHANGEMARGINRATE.html.md)

    [OnRspQryExchangeMarginRateAdjust](pages/251-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYEXCHANGEMARGINRATEADJUST.html.md)

    [OnRspQryExchangeRate](pages/252-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYEXCHANGERATE.html.md)

    [OnRspQryExecOrder](pages/253-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYEXECORDER.html.md)

    [OnRspQryForQuote](pages/254-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYFORQUOTE.html.md)

    [OnRspQryInstrument](pages/255-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINSTRUMENT.html.md)

    [OnRspQryInstrumentCommissionRate](pages/256-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINSTRUMENTCOMMISSIONRATE.html.md)

    [OnRspQryInstrumentMarginRate](pages/257-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINSTRUMENTMARGINRATE.html.md)

    [OnRspQryInstrumentOrderCommRate](pages/258-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINSTRUMENTORDERCOMMRATE.html.md)

    [OnRspQryInvestor](pages/259-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTOR.html.md)

    [OnRspQryInvestorPosition](pages/260-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPOSITION.html.md)

    [OnRspQryInvestorPositionCombineDetail](pages/261-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPOSITIONCOMBINEDETAIL.html.md)

    [OnRspQryInvestorPositionDetail](pages/262-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPOSITIONDETAIL.html.md)

    [OnRspQryInvestorProductGroupMargin](pages/263-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPRODUCTGROUPMARGIN.html.md)

    [OnRspQryInvestUnit](pages/264-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTUNIT.html.md)

    [OnRspQryMMInstrumentCommissionRate](pages/265-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYMMINSTRUMENTCOMMISSIONRATE.html.md)

    [OnRspQryMMOptionInstrCommRate](pages/266-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYMMOPTIONINSTRCOMMRATE.html.md)

    [OnRspQryNotice](pages/267-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYNOTICE.html.md)

    [OnRspQryOptionInstrCommRate](pages/268-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYOPTIONINSTRCOMMRATE.html.md)

    [OnRspQryOptionInstrTradeCost](pages/269-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYOPTIONINSTRTRADECOST.html.md)

    [OnRspQryOptionSelfClose](pages/270-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYOPTIONSELFCLOSE.html.md)

    [OnRspQryOrder](pages/271-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYORDER.html.md)

    [OnRspQryParkedOrder](pages/272-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYPARKEDORDER.html.md)

    [OnRspQryParkedOrderAction](pages/273-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYPARKEDORDERACTION.html.md)

    [OnRspQryProduct](pages/274-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYPRODUCT.html.md)

    [OnRspQryProductExchRate](pages/275-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYPRODUCTEXCHRATE.html.md)

    [OnRspQryProductGroup](pages/276-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYPRODUCTGROUP.html.md)

    [OnRspQryQuote](pages/277-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYQUOTE.html.md)

    [OnRspQrySecAgentACIDMap](pages/278-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSECAGENTACIDMAP.html.md)

    [OnRspQrySecAgentCheckMode](pages/279-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSECAGENTCHECKMODE.html.md)

    [OnRspQrySecAgentTradeInfo](pages/280-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSECAGENTTRADEINFO.html.md)

    [OnRspQrySecAgentTradingAccount](pages/281-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSECAGENTTRADINGACCOUNT.html.md)

    [OnRspQrySettlementInfo](pages/282-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSETTLEMENTINFO.html.md)

    [OnRspQrySettlementInfoConfirm](pages/283-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSETTLEMENTINFOCONFIRM.html.md)

    [OnRspQryTrade](pages/284-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRADE.html.md)

    [OnRspQryTradingAccount](pages/285-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRADINGACCOUNT.html.md)

    [OnRspQryTradingCode](pages/286-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRADINGCODE.html.md)

    [OnRspQryTradingNotice](pages/287-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRADINGNOTICE.html.md)

    [OnRspQryTransferBank](pages/288-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRANSFERBANK.html.md)

    [OnRspQryTransferSerial](pages/289-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRANSFERSERIAL.html.md)

    [OnRspQueryBankAccountMoneyByFuture](pages/290-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQUERYBANKACCOUNTMONEYBYFUTURE.html.md)

    [OnRspQueryCFMMCTradingAccountToken](pages/291-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQUERYCFMMCTRADINGACCOUNTTOKEN.html.md)

    [OnRspQuoteAction](pages/292-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQUOTEACTION.html.md)

    [OnRspQuoteInsert](pages/293-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQUOTEINSERT.html.md)

    [OnRspRemoveParkedOrder](pages/294-JYJK-CTHOSTFTDCTRADERSPI-ONRSPREMOVEPARKEDORDER.html.md)

    [OnRspRemoveParkedOrderAction](pages/295-JYJK-CTHOSTFTDCTRADERSPI-ONRSPREMOVEPARKEDORDERACTION.html.md)

    [OnRspSettlementInfoConfirm](pages/296-JYJK-CTHOSTFTDCTRADERSPI-ONRSPSETTLEMENTINFOCONFIRM.html.md)

    [OnRspTradingAccountPasswordUpdate](pages/297-JYJK-CTHOSTFTDCTRADERSPI-ONRSPTRADINGACCOUNTPASSWORDUPDATE.html.md)

    [OnRspUserAuthMethod](pages/298-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERAUTHMETHOD.html.md)

    [OnRspUserLogin](pages/299-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERLOGIN.html.md)

    [OnRspUserLogout](pages/300-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERLOGOUT.html.md)

    [OnRspUserPasswordUpdate](pages/301-JYJK-CTHOSTFTDCTRADERSPI-ONRSPUSERPASSWORDUPDATE.html.md)

    [OnRtnBulletin](pages/302-JYJK-CTHOSTFTDCTRADERSPI-ONRTNBULLETIN.html.md)

    [OnRtnCancelAccountByBank](pages/303-JYJK-CTHOSTFTDCTRADERSPI-ONRTNCANCELACCOUNTBYBANK.html.md)

    [OnRtnCFMMCTradingAccountToken](pages/304-JYJK-CTHOSTFTDCTRADERSPI-ONRTNCFMMCTRADINGACCOUNTTOKEN.html.md)

    [OnRtnChangeAccountByBank](pages/305-JYJK-CTHOSTFTDCTRADERSPI-ONRTNCHANGEACCOUNTBYBANK.html.md)

    [OnRtnCombAction](pages/306-JYJK-CTHOSTFTDCTRADERSPI-ONRTNCOMBACTION.html.md)

    [OnRtnErrorConditionalOrder](pages/307-JYJK-CTHOSTFTDCTRADERSPI-ONRTNERRORCONDITIONALORDER.html.md)

    [OnRtnExecOrder](pages/308-JYJK-CTHOSTFTDCTRADERSPI-ONRTNEXECORDER.html.md)

    [OnRtnForQuoteRsp](pages/309-JYJK-CTHOSTFTDCTRADERSPI-ONRTNFORQUOTERSP.html.md)

    [OnRtnFromBankToFutureByBank](pages/310-JYJK-CTHOSTFTDCTRADERSPI-ONRTNFROMBANKTOFUTUREBYBANK.html.md)

    [OnRtnFromBankToFutureByFuture](pages/311-JYJK-CTHOSTFTDCTRADERSPI-ONRTNFROMBANKTOFUTUREBYFUTURE.html.md)

    [OnRtnFromFutureToBankByBank](pages/312-JYJK-CTHOSTFTDCTRADERSPI-ONRTNFROMFUTURETOBANKBYBANK.html.md)

    [OnRtnFromFutureToBankByFuture](pages/313-JYJK-CTHOSTFTDCTRADERSPI-ONRTNFROMFUTURETOBANKBYFUTURE.html.md)

    [OnRtnInstrumentStatus](pages/314-JYJK-CTHOSTFTDCTRADERSPI-ONRTNINSTRUMENTSTATUS.html.md)

    [OnRtnOpenAccountByBank](pages/315-JYJK-CTHOSTFTDCTRADERSPI-ONRTNOPENACCOUNTBYBANK.html.md)

    [OnRtnOptionSelfClose](pages/316-JYJK-CTHOSTFTDCTRADERSPI-ONRTNOPTIONSELFCLOSE.html.md)

    [OnRtnOrder](pages/317-JYJK-CTHOSTFTDCTRADERSPI-ONRTNORDER.html.md)

    [OnRtnQueryBankBalanceByFuture](pages/318-JYJK-CTHOSTFTDCTRADERSPI-ONRTNQUERYBANKBALANCEBYFUTURE.html.md)

    [OnRtnQuote](pages/319-JYJK-CTHOSTFTDCTRADERSPI-ONRTNQUOTE.html.md)

    [OnRtnRepealFromBankToFutureByBank](pages/320-JYJK-CTHOSTFTDCTRADERSPI-ONRTNREPEALFROMBANKTOFUTUREBYBANK.html.md)

    [OnRtnRepealFromBankToFutureByFuture](pages/321-JYJK-CTHOSTFTDCTRADERSPI-ONRTNREPEALFROMBANKTOFUTUREBYFUTURE.html.md)

    [OnRtnRepealFromBankToFutureByFutureManual](pages/322-JYJK-CTHOSTFTDCTRADERSPI-ONRTNREPEALFROMBANKTOFUTUREBYFUTUREMANUAL.html.md)

    [OnRtnRepealFromFutureToBankByBank](pages/323-JYJK-CTHOSTFTDCTRADERSPI-ONRTNREPEALFROMFUTURETOBANKBYBANK.html.md)

    [OnRtnRepealFromFutureToBankByFuture](pages/324-JYJK-CTHOSTFTDCTRADERSPI-ONRTNREPEALFROMFUTURETOBANKBYFUTURE.html.md)

    [OnRtnRepealFromFutureToBankByFutureManual](pages/325-JYJK-CTHOSTFTDCTRADERSPI-ONRTNREPEALFROMFUTURETOBANKBYFUTUREMANUAL.html.md)

    [OnRtnTrade](pages/326-JYJK-CTHOSTFTDCTRADERSPI-ONRTNTRADE.html.md)

    [OnRtnTradingNotice](pages/327-JYJK-CTHOSTFTDCTRADERSPI-ONRTNTRADINGNOTICE.html.md)

    [OnRspQryMaxOrderVolume](pages/328-JYJK-CTHOSTFTDCTRADERSPI-OnRspQryMaxOrderVolume.html.md)

    [OnRspQryClassifiedInstrument](pages/329-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCLASSIFIEDINSTRUMENT.html.md)

    [OnRspQryCombPromotionParam](pages/330-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCOMBPROMOTIONPARAM.html.md)

    [OnRspQryRiskSettleInvstPosition](pages/331-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRISKSETTLEINVSTPOSITION.html.md)

    [OnRspQryRiskSettleProductStatus](pages/332-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRISKSETTLEPRODUCTSTATUS.html.md)

    [OnRspQryTraderOffer](pages/333-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYTRADEROFFER.html.md)

    [OnRspQrySPBMFutureParameter](pages/334-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMFUTUREPARAMETER.html.md)

    [OnRspQrySPBMOptionParameter](pages/335-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMOPTIONPARAMETER.html.md)

    [OnRspQrySPBMIntraParameter](pages/336-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMINTRAPARAMETER.html.md)

    [OnRspQrySPBMInterParameter](pages/337-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMINTERPARAMETER.html.md)

    [OnRspQrySPBMPortfDefinition](pages/338-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMPORTFDEFINITION.html.md)

    [OnRspQrySPBMInvestorPortfDef](pages/339-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMINVESTORPORTFDEF.html.md)

    [OnRspQryInvestorPortfMarginRatio](pages/340-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPORTFMARGINRATIO.html.md)

    [OnRspQryInvestorProdSPBMDetail](pages/341-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPRODSPBMDETAIL.html.md)

    [OnRspQryInvestorCommoditySPMMMargin](pages/342-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORCOMMODITYSPMMMARGIN.html.md)

    [OnRspQryInvestorCommodityGroupSPMMMargin](pages/343-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORCOMMODITYGROUPSPMMMARGIN.html.md)

    [OnRspQrySPMMInstParam](pages/344-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPMMINSTPARAM.html.md)

    [OnRspQrySPMMProductParam](pages/345-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPMMPRODUCTPARAM.html.md)

    [OnRspQrySPBMAddOnInterParameter](pages/346-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPBMADDONINTERPARAMETER.html.md)

    [OnRspQryRCAMSCombProductInfo](pages/347-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSCOMBPRODUCTINFO.html.md)

    [OnRspQryRCAMSInstrParameter](pages/348-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSINSTRPARAMETER.html.md)

    [OnRspQryRCAMSIntraParameter](pages/349-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSINTRAPARAMETER.html.md)

    [OnRspQryRCAMSInterParameter](pages/350-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSINTERPARAMETER.html.md)

    [OnRspQryRCAMSShortOptAdjustParam](pages/351-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSSHORTOPTADJUSTPARAM.html.md)

    [OnRspQryRCAMSInvestorCombPosition](pages/352-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRCAMSINVESTORCOMBPOSITION.html.md)

    [OnRspQryInvestorProdRCAMSMargin](pages/353-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPRODRCAMSMARGIN.html.md)

    [OnRspQryRULEInstrParameter](pages/354-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRULEINSTRPARAMETER.html.md)

    [OnRspQryRULEIntraParameter](pages/355-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRULEINTRAPARAMETER.html.md)

    [OnRspQryRULEInterParameter](pages/356-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYRULEINTERPARAMETER.html.md)

    [OnRspQryInvestorProdRULEMargin](pages/357-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPRODRULEMARGIN.html.md)

    [OnRspQryInvestorPortfSetting](pages/358-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORPORTFSETTING.html.md)

    [OnRspQryInvestorInfoCommRec](pages/359-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYINVESTORINFOCOMMREC.html.md)

    [OnRspQryCombLeg](pages/360-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYCOMBLEG.html.md)

    [OnRspOffsetSetting](pages/361-JYJK-CTHOSTFTDCTRADERSPI-ONRSPOFFSETSETTING.html.md)

    [OnRspCancelOffsetSetting](pages/362-JYJK-CTHOSTFTDCTRADERSPI-ONRSPCANCELOFFSETSETTING.html.md)

    [OnRtnOffsetSetting](pages/363-JYJK-CTHOSTFTDCTRADERSPI-ONRTNOFFSETSETTING.html.md)

    [OnErrRtnOffsetSetting](pages/364-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNOFFSETSETTING.html.md)

    [OnErrRtnCancelOffsetSetting](pages/365-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNCANCELOFFSETSETTING.html.md)

    [OnRspQryOffsetSetting](pages/366-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYOFFSETSETTING.html.md)

    [OnRspQryUserSession](pages/367-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYUSERSESSION.html.md)

    [OnRtnPrivateSeqNo](pages/368-JYJK-CTHOSTFTDCTRADERSPI-ONRTNPRIVATESEQNO.html.md)

    [OnRspGenSMSCode](pages/369-JYJK-CTHOSTFTDCTRADERSPI-ONRSPGENSMSCODE.html.md)

    [OnRspSpdApply](pages/370-JYJK-CTHOSTFTDCTRADERSPI-ONRSPSPDAPPLY.html.md)

    [OnRspSpdApplyAction](pages/371-JYJK-CTHOSTFTDCTRADERSPI-ONRSPSPDAPPLYACTION.html.md)

    [OnRspQrySpdApply](pages/372-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYSPDAPPLY.html.md)

    [OnRtnSpdApply](pages/373-JYJK-CTHOSTFTDCTRADERSPI-ONRTNSPDAPPLY.html.md)

    [OnErrRtnSpdApply](pages/374-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNSPDAPPLY.html.md)

    [OnErrRtnSpdApplyAction](pages/375-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNSPDAPPLYACTION.html.md)

    [OnRspHedgeCfm](pages/376-JYJK-CTHOSTFTDCTRADERSPI-ONRSPHEDGECFM.html.md)

    [OnRspHedgeCfmAction](pages/377-JYJK-CTHOSTFTDCTRADERSPI-ONRSPHEDGECFMACTION.html.md)

    [OnRspQryHedgeCfm](pages/378-JYJK-CTHOSTFTDCTRADERSPI-ONRSPQRYHEDGECFM.html.md)

    [OnRtnHedgeCfm](pages/379-JYJK-CTHOSTFTDCTRADERSPI-ONRTNHEDGECFM.html.md)

    [OnErrRtnHedgeCfm](pages/380-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNHEDGECFM.html.md)

    [OnErrRtnHedgeCfmAction](pages/381-JYJK-CTHOSTFTDCTRADERSPI-ONERRRTNHEDGECFMACTION.html.md)

<a id="author"></a>

<a id="theme_switcher"></a>
