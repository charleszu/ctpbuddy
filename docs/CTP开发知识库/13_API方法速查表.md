# API 方法速查表（交易/行情）

> 取自官方 SDK 6.7.13（20260225 构建）头文件。Req* 为主动请求，OnRsp*/OnRtn*/OnErrRtn* 为 SPI 回调。

## CThostFtdcTraderApi（交易 API 主动调用）（140）

| 方法 | 返回 | 参数 | 说明 |
|---|---|---|---|
| `CreateFtdcTraderApi` | `CThostFtdcTraderApi *` | `const char *pszFlowPath = "", bool bIsProductionMode = true` | 创建TraderApi @param pszFlowPath 存贮订阅信息文件的目录，默认为当前目录 @param bIsProductionMode true:使用生产版本的API  false:使用测评版本的API @return 创建出的UserApi |
| `Release` | `void` | `` | 删除接口对象本身 @remark 不再使用本接口对象时,调用该函数删除接口对象 |
| `Init` | `void` | `` | 初始化 @remark 初始化运行环境,只有调用后,接口才开始工作 |
| `Join` | `int` | `` | 等待接口线程结束运行 @return 线程退出代码 |
| `GetFrontInfo` | `void` | `CThostFtdcFrontInfoField* pFrontInfo` | 获取已连接的前置的信息  @param pFrontInfo：输入输出参数，用于存储获取到的前置信息，不能为空  @remark 连接成功后，可获取正确的前置地址信息  @remark 登录成功后，可获取正确的前置流控信息 |
| `RegisterFront` | `void` | `char *pszFrontAddress` | 注册前置机网络地址 @param pszFrontAddress：前置机网络地址。 @remark 网络地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:17001”。 @remark “tcp”代表传输协议，“127.0.0.1”代表服务器地址。”17001”代表服务器端口号。 |
| `RegisterNameServer` | `void` | `char *pszNsAddress` | 注册名字服务器网络地址 @param pszNsAddress：名字服务器网络地址。 @remark 网络地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:12001”。 @remark “tcp”代表传输协议，“127.0.0.1”代表服务器地址。”12001”代表服务器端口号。 @remark RegisterNameServer优先于RegisterFront |
| `RegisterFensUserInfo` | `void` | `CThostFtdcFensUserInfoField * pFensUserInfo` | 注册名字服务器用户信息 @param pFensUserInfo：用户信息。 |
| `RegisterSpi` | `void` | `CThostFtdcTraderSpi *pSpi` | 注册回调接口 @param pSpi 派生自回调接口类的实例 |
| `SubscribePrivateTopic` | `void` | `THOST_TE_RESUME_TYPE nResumeType, int nSeqNo=1` | 订阅私有流。 @param nResumeType 私有流重传方式         THOST_TERT_RESTART:从本交易日开始重传         THOST_TERT_RESUME:从上次收到的续传         THOST_TERT_QUICK:只传送登录后私有流的内容         THOST_TERT_RESUME_FROM_SEQ_NO:从指定序号开始重传，序号从1开始 @param nSeqNo 私有流序号，只在THOST_TERT_RESUME_FROM_SEQ_NO模式下有效 @remark 该方法要在Init方法前调用。若不调用则不会收到私有流的数据。 |
| `SubscribePublicTopic` | `void` | `THOST_TE_RESUME_TYPE nResumeType` | 订阅公共流。 @param nResumeType 公共流重传方式         THOST_TERT_RESTART:从本交易日开始重传         THOST_TERT_RESUME:从上次收到的续传         THOST_TERT_QUICK:只传送登录后公共流的内容         THOST_TERT_NONE:取消订阅公共流 @remark 该方法要在Init方法前调用。若不调用则不会收到公共流的数据。 |
| `ReqAuthenticate` | `int` | `CThostFtdcReqAuthenticateField *pReqAuthenticateField, int nRequestID` | 客户端认证请求 |
| `RegisterUserSystemInfo` | `int` | `CThostFtdcUserSystemInfoField *pUserSystemInfo` | 注册用户终端信息，用于中继服务器多连接模式 需要在终端认证成功后，用户登录前调用该接口 |
| `SubmitUserSystemInfo` | `int` | `CThostFtdcUserSystemInfoField *pUserSystemInfo` | 上报用户终端信息，用于中继服务器操作员登录模式 操作员登录后，可以多次调用该接口上报客户信息 |
| `RegisterWechatUserSystemInfo` | `int` | `CThostFtdcWechatUserSystemInfoField *pUserSystemInfo` | 注册用户终端信息，用于中继服务器多连接模式.用于微信小程序等应用上报信息. |
| `SubmitWechatUserSystemInfo` | `int` | `CThostFtdcWechatUserSystemInfoField *pUserSystemInfo` | 上报用户终端信息，用于中继服务器操作员登录模式.用于微信小程序等应用上报信息. |
| `ReqUserLogin` | `int` | `CThostFtdcReqUserLoginField *pReqUserLoginField, int nRequestID` | 用户登录请求 |
| `ReqUserLogout` | `int` | `CThostFtdcUserLogoutField *pUserLogout, int nRequestID` | 登出请求 |
| `ReqUserPasswordUpdate` | `int` | `CThostFtdcUserPasswordUpdateField *pUserPasswordUpdate, int nRequestID` | 用户口令更新请求 |
| `ReqTradingAccountPasswordUpdate` | `int` | `CThostFtdcTradingAccountPasswordUpdateField *pTradingAccountPasswordUpdate, int nRequestID` | 资金账户口令更新请求 |
| `ReqUserAuthMethod` | `int` | `CThostFtdcReqUserAuthMethodField *pReqUserAuthMethod, int nRequestID` | 查询用户当前支持的认证模式 |
| `ReqGenUserCaptcha` | `int` | `CThostFtdcReqGenUserCaptchaField *pReqGenUserCaptcha, int nRequestID` | 用户发出获取图形验证码请求 |
| `ReqGenUserText` | `int` | `CThostFtdcReqGenUserTextField *pReqGenUserText, int nRequestID` | 用户发出获取短信验证码请求 |
| `ReqUserLoginWithCaptcha` | `int` | `CThostFtdcReqUserLoginWithCaptchaField *pReqUserLoginWithCaptcha, int nRequestID` | 用户发出带有图片验证码的登陆请求 |
| `ReqUserLoginWithText` | `int` | `CThostFtdcReqUserLoginWithTextField *pReqUserLoginWithText, int nRequestID` | 用户发出带有短信验证码的登陆请求 |
| `ReqUserLoginWithOTP` | `int` | `CThostFtdcReqUserLoginWithOTPField *pReqUserLoginWithOTP, int nRequestID` | 用户发出带有动态口令的登陆请求 |
| `ReqOrderInsert` | `int` | `CThostFtdcInputOrderField *pInputOrder, int nRequestID` | 报单录入请求 |
| `ReqParkedOrderInsert` | `int` | `CThostFtdcParkedOrderField *pParkedOrder, int nRequestID` | 预埋单录入请求 |
| `ReqParkedOrderAction` | `int` | `CThostFtdcParkedOrderActionField *pParkedOrderAction, int nRequestID` | 预埋撤单录入请求 |
| `ReqOrderAction` | `int` | `CThostFtdcInputOrderActionField *pInputOrderAction, int nRequestID` | 报单操作请求 |
| `ReqQryMaxOrderVolume` | `int` | `CThostFtdcQryMaxOrderVolumeField *pQryMaxOrderVolume, int nRequestID` | 查询最大报单数量请求 |
| `ReqSettlementInfoConfirm` | `int` | `CThostFtdcSettlementInfoConfirmField *pSettlementInfoConfirm, int nRequestID` | 投资者结算结果确认 |
| `ReqRemoveParkedOrder` | `int` | `CThostFtdcRemoveParkedOrderField *pRemoveParkedOrder, int nRequestID` | 请求删除预埋单 |
| `ReqRemoveParkedOrderAction` | `int` | `CThostFtdcRemoveParkedOrderActionField *pRemoveParkedOrderAction, int nRequestID` | 请求删除预埋撤单 |
| `ReqExecOrderInsert` | `int` | `CThostFtdcInputExecOrderField *pInputExecOrder, int nRequestID` | 执行宣告录入请求 |
| `ReqExecOrderAction` | `int` | `CThostFtdcInputExecOrderActionField *pInputExecOrderAction, int nRequestID` | 执行宣告操作请求 |
| `ReqForQuoteInsert` | `int` | `CThostFtdcInputForQuoteField *pInputForQuote, int nRequestID` | 询价录入请求 |
| `ReqQuoteInsert` | `int` | `CThostFtdcInputQuoteField *pInputQuote, int nRequestID` | 报价录入请求 |
| `ReqQuoteAction` | `int` | `CThostFtdcInputQuoteActionField *pInputQuoteAction, int nRequestID` | 报价操作请求 |
| `ReqBatchOrderAction` | `int` | `CThostFtdcInputBatchOrderActionField *pInputBatchOrderAction, int nRequestID` | 批量报单操作请求 |
| `ReqOptionSelfCloseInsert` | `int` | `CThostFtdcInputOptionSelfCloseField *pInputOptionSelfClose, int nRequestID` | 期权自对冲录入请求 |
| `ReqOptionSelfCloseAction` | `int` | `CThostFtdcInputOptionSelfCloseActionField *pInputOptionSelfCloseAction, int nRequestID` | 期权自对冲操作请求 |
| `ReqCombActionInsert` | `int` | `CThostFtdcInputCombActionField *pInputCombAction, int nRequestID` | 申请组合录入请求 |
| `ReqQryOrder` | `int` | `CThostFtdcQryOrderField *pQryOrder, int nRequestID` | 请求查询报单 |
| `ReqQryTrade` | `int` | `CThostFtdcQryTradeField *pQryTrade, int nRequestID` | 请求查询成交 |
| `ReqQryInvestorPosition` | `int` | `CThostFtdcQryInvestorPositionField *pQryInvestorPosition, int nRequestID` | 请求查询投资者持仓 |
| `ReqQryTradingAccount` | `int` | `CThostFtdcQryTradingAccountField *pQryTradingAccount, int nRequestID` | 请求查询资金账户 |
| `ReqQryInvestor` | `int` | `CThostFtdcQryInvestorField *pQryInvestor, int nRequestID` | 请求查询投资者 |
| `ReqQryTradingCode` | `int` | `CThostFtdcQryTradingCodeField *pQryTradingCode, int nRequestID` | 请求查询交易编码 |
| `ReqQryInstrumentMarginRate` | `int` | `CThostFtdcQryInstrumentMarginRateField *pQryInstrumentMarginRate, int nRequestID` | 请求查询合约保证金率 |
| `ReqQryInstrumentCommissionRate` | `int` | `CThostFtdcQryInstrumentCommissionRateField *pQryInstrumentCommissionRate, int nRequestID` | 请求查询合约手续费率 |
| `ReqQryUserSession` | `int` | `CThostFtdcQryUserSessionField *pQryUserSession, int nRequestID` | 请求查询用户会话 |
| `ReqQryExchange` | `int` | `CThostFtdcQryExchangeField *pQryExchange, int nRequestID` | 请求查询交易所 |
| `ReqQryProduct` | `int` | `CThostFtdcQryProductField *pQryProduct, int nRequestID` | 请求查询产品 |
| `ReqQryInstrument` | `int` | `CThostFtdcQryInstrumentField *pQryInstrument, int nRequestID` | 请求查询合约 |
| `ReqQryDepthMarketData` | `int` | `CThostFtdcQryDepthMarketDataField *pQryDepthMarketData, int nRequestID` | 请求查询行情 |
| `ReqQryTraderOffer` | `int` | `CThostFtdcQryTraderOfferField *pQryTraderOffer, int nRequestID` | 请求查询交易员报盘机 |
| `ReqQrySettlementInfo` | `int` | `CThostFtdcQrySettlementInfoField *pQrySettlementInfo, int nRequestID` | 请求查询投资者结算结果 |
| `ReqQryTransferBank` | `int` | `CThostFtdcQryTransferBankField *pQryTransferBank, int nRequestID` | 请求查询转帐银行 |
| `ReqQryInvestorPositionDetail` | `int` | `CThostFtdcQryInvestorPositionDetailField *pQryInvestorPositionDetail, int nRequestID` | 请求查询投资者持仓明细 |
| `ReqQryNotice` | `int` | `CThostFtdcQryNoticeField *pQryNotice, int nRequestID` | 请求查询客户通知 |
| `ReqQrySettlementInfoConfirm` | `int` | `CThostFtdcQrySettlementInfoConfirmField *pQrySettlementInfoConfirm, int nRequestID` | 请求查询结算信息确认 |
| `ReqQryInvestorPositionCombineDetail` | `int` | `CThostFtdcQryInvestorPositionCombineDetailField *pQryInvestorPositionCombineDetail, int nRequestID` | 请求查询投资者持仓明细 |
| `ReqQryCFMMCTradingAccountKey` | `int` | `CThostFtdcQryCFMMCTradingAccountKeyField *pQryCFMMCTradingAccountKey, int nRequestID` | 请求查询保证金监管系统经纪公司资金账户密钥 |
| `ReqQryEWarrantOffset` | `int` | `CThostFtdcQryEWarrantOffsetField *pQryEWarrantOffset, int nRequestID` | 请求查询仓单折抵信息 |
| `ReqQryInvestorProductGroupMargin` | `int` | `CThostFtdcQryInvestorProductGroupMarginField *pQryInvestorProductGroupMargin, int nRequestID` | 请求查询投资者品种/跨品种保证金 |
| `ReqQryExchangeMarginRate` | `int` | `CThostFtdcQryExchangeMarginRateField *pQryExchangeMarginRate, int nRequestID` | 请求查询交易所保证金率 |
| `ReqQryExchangeMarginRateAdjust` | `int` | `CThostFtdcQryExchangeMarginRateAdjustField *pQryExchangeMarginRateAdjust, int nRequestID` | 请求查询交易所调整保证金率 |
| `ReqQryExchangeRate` | `int` | `CThostFtdcQryExchangeRateField *pQryExchangeRate, int nRequestID` | 请求查询汇率 |
| `ReqQrySecAgentACIDMap` | `int` | `CThostFtdcQrySecAgentACIDMapField *pQrySecAgentACIDMap, int nRequestID` | 请求查询二级代理操作员银期权限 |
| `ReqQryProductExchRate` | `int` | `CThostFtdcQryProductExchRateField *pQryProductExchRate, int nRequestID` | 请求查询产品报价汇率 |
| `ReqQryProductGroup` | `int` | `CThostFtdcQryProductGroupField *pQryProductGroup, int nRequestID` | 请求查询产品组 |
| `ReqQryMMInstrumentCommissionRate` | `int` | `CThostFtdcQryMMInstrumentCommissionRateField *pQryMMInstrumentCommissionRate, int nRequestID` | 请求查询做市商合约手续费率 |
| `ReqQryMMOptionInstrCommRate` | `int` | `CThostFtdcQryMMOptionInstrCommRateField *pQryMMOptionInstrCommRate, int nRequestID` | 请求查询做市商期权合约手续费 |
| `ReqQryInstrumentOrderCommRate` | `int` | `CThostFtdcQryInstrumentOrderCommRateField *pQryInstrumentOrderCommRate, int nRequestID` | 请求查询报单手续费 |
| `ReqQrySecAgentTradingAccount` | `int` | `CThostFtdcQryTradingAccountField *pQryTradingAccount, int nRequestID` | 请求查询资金账户 |
| `ReqQrySecAgentCheckMode` | `int` | `CThostFtdcQrySecAgentCheckModeField *pQrySecAgentCheckMode, int nRequestID` | 请求查询二级代理商资金校验模式 |
| `ReqQrySecAgentTradeInfo` | `int` | `CThostFtdcQrySecAgentTradeInfoField *pQrySecAgentTradeInfo, int nRequestID` | 请求查询二级代理商信息 |
| `ReqQryOptionInstrTradeCost` | `int` | `CThostFtdcQryOptionInstrTradeCostField *pQryOptionInstrTradeCost, int nRequestID` | 请求查询期权交易成本 |
| `ReqQryOptionInstrCommRate` | `int` | `CThostFtdcQryOptionInstrCommRateField *pQryOptionInstrCommRate, int nRequestID` | 请求查询期权合约手续费 |
| `ReqQryExecOrder` | `int` | `CThostFtdcQryExecOrderField *pQryExecOrder, int nRequestID` | 请求查询执行宣告 |
| `ReqQryForQuote` | `int` | `CThostFtdcQryForQuoteField *pQryForQuote, int nRequestID` | 请求查询询价 |
| `ReqQryQuote` | `int` | `CThostFtdcQryQuoteField *pQryQuote, int nRequestID` | 请求查询报价 |
| `ReqQryOptionSelfClose` | `int` | `CThostFtdcQryOptionSelfCloseField *pQryOptionSelfClose, int nRequestID` | 请求查询期权自对冲 |
| `ReqQryInvestUnit` | `int` | `CThostFtdcQryInvestUnitField *pQryInvestUnit, int nRequestID` | 请求查询投资单元 |
| `ReqQryCombInstrumentGuard` | `int` | `CThostFtdcQryCombInstrumentGuardField *pQryCombInstrumentGuard, int nRequestID` | 请求查询组合合约安全系数 |
| `ReqQryCombAction` | `int` | `CThostFtdcQryCombActionField *pQryCombAction, int nRequestID` | 请求查询申请组合 |
| `ReqQryTransferSerial` | `int` | `CThostFtdcQryTransferSerialField *pQryTransferSerial, int nRequestID` | 请求查询转帐流水 |
| `ReqQryAccountregister` | `int` | `CThostFtdcQryAccountregisterField *pQryAccountregister, int nRequestID` | 请求查询银期签约关系 |
| `ReqQryContractBank` | `int` | `CThostFtdcQryContractBankField *pQryContractBank, int nRequestID` | 请求查询签约银行 |
| `ReqQryParkedOrder` | `int` | `CThostFtdcQryParkedOrderField *pQryParkedOrder, int nRequestID` | 请求查询预埋单 |
| `ReqQryParkedOrderAction` | `int` | `CThostFtdcQryParkedOrderActionField *pQryParkedOrderAction, int nRequestID` | 请求查询预埋撤单 |
| `ReqQryTradingNotice` | `int` | `CThostFtdcQryTradingNoticeField *pQryTradingNotice, int nRequestID` | 请求查询交易通知 |
| `ReqQryBrokerTradingParams` | `int` | `CThostFtdcQryBrokerTradingParamsField *pQryBrokerTradingParams, int nRequestID` | 请求查询经纪公司交易参数 |
| `ReqQryBrokerTradingAlgos` | `int` | `CThostFtdcQryBrokerTradingAlgosField *pQryBrokerTradingAlgos, int nRequestID` | 请求查询经纪公司交易算法 |
| `ReqQueryCFMMCTradingAccountToken` | `int` | `CThostFtdcQueryCFMMCTradingAccountTokenField *pQueryCFMMCTradingAccountToken, int nRequestID` | 请求查询监控中心用户令牌 |
| `ReqFromBankToFutureByFuture` | `int` | `CThostFtdcReqTransferField *pReqTransfer, int nRequestID` | 期货发起银行资金转期货请求 |
| `ReqFromFutureToBankByFuture` | `int` | `CThostFtdcReqTransferField *pReqTransfer, int nRequestID` | 期货发起期货资金转银行请求 |
| `ReqQueryBankAccountMoneyByFuture` | `int` | `CThostFtdcReqQueryAccountField *pReqQueryAccount, int nRequestID` | 期货发起查询银行余额请求 |
| `ReqQryClassifiedInstrument` | `int` | `CThostFtdcQryClassifiedInstrumentField *pQryClassifiedInstrument, int nRequestID` | 请求查询分类合约 |
| `ReqQryCombPromotionParam` | `int` | `CThostFtdcQryCombPromotionParamField *pQryCombPromotionParam, int nRequestID` | 请求组合优惠比例 |
| `ReqQryRiskSettleInvstPosition` | `int` | `CThostFtdcQryRiskSettleInvstPositionField *pQryRiskSettleInvstPosition, int nRequestID` | 投资者风险结算持仓查询 |
| `ReqQryRiskSettleProductStatus` | `int` | `CThostFtdcQryRiskSettleProductStatusField *pQryRiskSettleProductStatus, int nRequestID` | 风险结算产品查询 |
| `ReqQrySPBMFutureParameter` | `int` | `CThostFtdcQrySPBMFutureParameterField *pQrySPBMFutureParameter, int nRequestID` | SPBM期货合约参数查询 |
| `ReqQrySPBMOptionParameter` | `int` | `CThostFtdcQrySPBMOptionParameterField *pQrySPBMOptionParameter, int nRequestID` | SPBM期权合约参数查询 |
| `ReqQrySPBMIntraParameter` | `int` | `CThostFtdcQrySPBMIntraParameterField *pQrySPBMIntraParameter, int nRequestID` | SPBM品种内对锁仓折扣参数查询 |
| `ReqQrySPBMInterParameter` | `int` | `CThostFtdcQrySPBMInterParameterField *pQrySPBMInterParameter, int nRequestID` | SPBM跨品种抵扣参数查询 |
| `ReqQrySPBMPortfDefinition` | `int` | `CThostFtdcQrySPBMPortfDefinitionField *pQrySPBMPortfDefinition, int nRequestID` | SPBM组合保证金套餐查询 |
| `ReqQrySPBMInvestorPortfDef` | `int` | `CThostFtdcQrySPBMInvestorPortfDefField *pQrySPBMInvestorPortfDef, int nRequestID` | 投资者SPBM套餐选择查询 |
| `ReqQryInvestorPortfMarginRatio` | `int` | `CThostFtdcQryInvestorPortfMarginRatioField *pQryInvestorPortfMarginRatio, int nRequestID` | 投资者新型组合保证金系数查询 |
| `ReqQryInvestorProdSPBMDetail` | `int` | `CThostFtdcQryInvestorProdSPBMDetailField *pQryInvestorProdSPBMDetail, int nRequestID` | 投资者产品SPBM明细查询 |
| `ReqQryInvestorCommoditySPMMMargin` | `int` | `CThostFtdcQryInvestorCommoditySPMMMarginField *pQryInvestorCommoditySPMMMargin, int nRequestID` | 投资者商品组SPMM记录查询 |
| `ReqQryInvestorCommodityGroupSPMMMargin` | `int` | `CThostFtdcQryInvestorCommodityGroupSPMMMarginField *pQryInvestorCommodityGroupSPMMMargin, int nRequestID` | 投资者商品群SPMM记录查询 |
| `ReqQrySPMMInstParam` | `int` | `CThostFtdcQrySPMMInstParamField *pQrySPMMInstParam, int nRequestID` | SPMM合约参数查询 |
| `ReqQrySPMMProductParam` | `int` | `CThostFtdcQrySPMMProductParamField *pQrySPMMProductParam, int nRequestID` | SPMM产品参数查询 |
| `ReqQrySPBMAddOnInterParameter` | `int` | `CThostFtdcQrySPBMAddOnInterParameterField *pQrySPBMAddOnInterParameter, int nRequestID` | SPBM附加跨品种抵扣参数查询 |
| `ReqQryRCAMSCombProductInfo` | `int` | `CThostFtdcQryRCAMSCombProductInfoField *pQryRCAMSCombProductInfo, int nRequestID` | RCAMS产品组合信息查询 |
| `ReqQryRCAMSInstrParameter` | `int` | `CThostFtdcQryRCAMSInstrParameterField *pQryRCAMSInstrParameter, int nRequestID` | RCAMS同合约风险对冲参数查询 |
| `ReqQryRCAMSIntraParameter` | `int` | `CThostFtdcQryRCAMSIntraParameterField *pQryRCAMSIntraParameter, int nRequestID` | RCAMS品种内风险对冲参数查询 |
| `ReqQryRCAMSInterParameter` | `int` | `CThostFtdcQryRCAMSInterParameterField *pQryRCAMSInterParameter, int nRequestID` | RCAMS跨品种风险折抵参数查询 |
| `ReqQryRCAMSShortOptAdjustParam` | `int` | `CThostFtdcQryRCAMSShortOptAdjustParamField *pQryRCAMSShortOptAdjustParam, int nRequestID` | RCAMS空头期权风险调整参数查询 |
| `ReqQryRCAMSInvestorCombPosition` | `int` | `CThostFtdcQryRCAMSInvestorCombPositionField *pQryRCAMSInvestorCombPosition, int nRequestID` | RCAMS策略组合持仓查询 |
| `ReqQryInvestorProdRCAMSMargin` | `int` | `CThostFtdcQryInvestorProdRCAMSMarginField *pQryInvestorProdRCAMSMargin, int nRequestID` | 投资者品种RCAMS保证金查询 |
| `ReqQryRULEInstrParameter` | `int` | `CThostFtdcQryRULEInstrParameterField *pQryRULEInstrParameter, int nRequestID` | RULE合约保证金参数查询 |
| `ReqQryRULEIntraParameter` | `int` | `CThostFtdcQryRULEIntraParameterField *pQryRULEIntraParameter, int nRequestID` | RULE品种内对锁仓折扣参数查询 |
| `ReqQryRULEInterParameter` | `int` | `CThostFtdcQryRULEInterParameterField *pQryRULEInterParameter, int nRequestID` | RULE跨品种抵扣参数查询 |
| `ReqQryInvestorProdRULEMargin` | `int` | `CThostFtdcQryInvestorProdRULEMarginField *pQryInvestorProdRULEMargin, int nRequestID` | 投资者产品RULE保证金查询 |
| `ReqQryInvestorPortfSetting` | `int` | `CThostFtdcQryInvestorPortfSettingField *pQryInvestorPortfSetting, int nRequestID` | 投资者新型组合保证金开关查询 |
| `ReqQryInvestorInfoCommRec` | `int` | `CThostFtdcQryInvestorInfoCommRecField *pQryInvestorInfoCommRec, int nRequestID` | 投资者申报费阶梯收取记录查询 |
| `ReqQryCombLeg` | `int` | `CThostFtdcQryCombLegField *pQryCombLeg, int nRequestID` | 组合腿信息查询 |
| `ReqOffsetSetting` | `int` | `CThostFtdcInputOffsetSettingField *pInputOffsetSetting, int nRequestID` | 对冲设置请求 |
| `ReqCancelOffsetSetting` | `int` | `CThostFtdcInputOffsetSettingField *pInputOffsetSetting, int nRequestID` | 对冲设置撤销请求 |
| `ReqQryOffsetSetting` | `int` | `CThostFtdcQryOffsetSettingField *pQryOffsetSetting, int nRequestID` | 投资者对冲设置查询 |
| `ReqGenSMSCode` | `int` | `CThostFtdcReqGenSMSCodeField *pReqGenSMSCode, int nRequestID` | 申请短信验证码请求 |
| `ReqSpdApply` | `int` | `CThostFtdcInputSpdApplyField *pInputSpdApply, int nRequestID` | 套利确认请求 |
| `ReqSpdApplyAction` | `int` | `CThostFtdcInputSpdApplyActionField *pInputSpdApplyAction, int nRequestID` | 套利确认撤销请求 |
| `ReqQrySpdApply` | `int` | `CThostFtdcQrySpdApplyField *pQrySpdApply, int nRequestID` | 套利确认查询请求 |
| `ReqHedgeCfm` | `int` | `CThostFtdcInputHedgeCfmField *pInputHedgeCfm, int nRequestID` | 套保确认请求 |
| `ReqHedgeCfmAction` | `int` | `CThostFtdcInputHedgeCfmActionField *pInputHedgeCfmAction, int nRequestID` | 套保确认撤销请求 |
| `ReqQryHedgeCfm` | `int` | `CThostFtdcQryHedgeCfmField *pQryHedgeCfm, int nRequestID` | 套保确认查询请求 |

## CThostFtdcTraderSpi（交易 SPI 回调）（178）

| 方法 | 返回 | 参数 | 说明 |
|---|---|---|---|
| `OnFrontConnected` | `void` | `` | 当客户端与交易后台建立起通信连接时（还未登录前），该方法被调用。 |
| `OnFrontDisconnected` | `void` | `int nReason` | 当客户端与交易后台通信连接断开时，该方法被调用。当发生这个情况后，API会自动重新连接，客户端可不做处理。 @param nReason 错误原因         0x1001 网络读失败         0x1002 网络写失败         0x2001 接收心跳超时         0x2002 发送心跳失败         0x2003 收到错误报文 |
| `OnHeartBeatWarning` | `void` | `int nTimeLapse` | 心跳超时警告。当长时间未收到报文时，该方法被调用。 @param nTimeLapse 距离上次接收报文的时间 |
| `OnRspAuthenticate` | `void` | `CThostFtdcRspAuthenticateField *pRspAuthenticateField, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 客户端认证响应 |
| `OnRtnPrivateSeqNo` | `void` | `int nSeqNo` | 该方法在处理私有流之前被调用 @param nSeqNo 即将被处理的私有流的序号 |
| `OnRspUserLogin` | `void` | `CThostFtdcRspUserLoginField *pRspUserLogin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 登录请求响应 |
| `OnRspUserLogout` | `void` | `CThostFtdcUserLogoutField *pUserLogout, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 登出请求响应 |
| `OnRspUserPasswordUpdate` | `void` | `CThostFtdcUserPasswordUpdateField *pUserPasswordUpdate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 用户口令更新请求响应 |
| `OnRspTradingAccountPasswordUpdate` | `void` | `CThostFtdcTradingAccountPasswordUpdateField *pTradingAccountPasswordUpdate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 资金账户口令更新请求响应 |
| `OnRspUserAuthMethod` | `void` | `CThostFtdcRspUserAuthMethodField *pRspUserAuthMethod, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 查询用户当前支持的认证模式的回复 |
| `OnRspGenUserCaptcha` | `void` | `CThostFtdcRspGenUserCaptchaField *pRspGenUserCaptcha, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 获取图形验证码请求的回复 |
| `OnRspGenUserText` | `void` | `CThostFtdcRspGenUserTextField *pRspGenUserText, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 获取短信验证码请求的回复 |
| `OnRspOrderInsert` | `void` | `CThostFtdcInputOrderField *pInputOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 报单录入请求响应 |
| `OnRspParkedOrderInsert` | `void` | `CThostFtdcParkedOrderField *pParkedOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 预埋单录入请求响应 |
| `OnRspParkedOrderAction` | `void` | `CThostFtdcParkedOrderActionField *pParkedOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 预埋撤单录入请求响应 |
| `OnRspOrderAction` | `void` | `CThostFtdcInputOrderActionField *pInputOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 报单操作请求响应 |
| `OnRspQryMaxOrderVolume` | `void` | `CThostFtdcQryMaxOrderVolumeField *pQryMaxOrderVolume, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 查询最大报单数量响应 |
| `OnRspSettlementInfoConfirm` | `void` | `CThostFtdcSettlementInfoConfirmField *pSettlementInfoConfirm, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者结算结果确认响应 |
| `OnRspRemoveParkedOrder` | `void` | `CThostFtdcRemoveParkedOrderField *pRemoveParkedOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 删除预埋单响应 |
| `OnRspRemoveParkedOrderAction` | `void` | `CThostFtdcRemoveParkedOrderActionField *pRemoveParkedOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 删除预埋撤单响应 |
| `OnRspExecOrderInsert` | `void` | `CThostFtdcInputExecOrderField *pInputExecOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 执行宣告录入请求响应 |
| `OnRspExecOrderAction` | `void` | `CThostFtdcInputExecOrderActionField *pInputExecOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 执行宣告操作请求响应 |
| `OnRspForQuoteInsert` | `void` | `CThostFtdcInputForQuoteField *pInputForQuote, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 询价录入请求响应 |
| `OnRspQuoteInsert` | `void` | `CThostFtdcInputQuoteField *pInputQuote, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 报价录入请求响应 |
| `OnRspQuoteAction` | `void` | `CThostFtdcInputQuoteActionField *pInputQuoteAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 报价操作请求响应 |
| `OnRspBatchOrderAction` | `void` | `CThostFtdcInputBatchOrderActionField *pInputBatchOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 批量报单操作请求响应 |
| `OnRspOptionSelfCloseInsert` | `void` | `CThostFtdcInputOptionSelfCloseField *pInputOptionSelfClose, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 期权自对冲录入请求响应 |
| `OnRspOptionSelfCloseAction` | `void` | `CThostFtdcInputOptionSelfCloseActionField *pInputOptionSelfCloseAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 期权自对冲操作请求响应 |
| `OnRspCombActionInsert` | `void` | `CThostFtdcInputCombActionField *pInputCombAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 申请组合录入请求响应 |
| `OnRspQryOrder` | `void` | `CThostFtdcOrderField *pOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询报单响应 |
| `OnRspQryTrade` | `void` | `CThostFtdcTradeField *pTrade, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询成交响应 |
| `OnRspQryInvestorPosition` | `void` | `CThostFtdcInvestorPositionField *pInvestorPosition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询投资者持仓响应 |
| `OnRspQryTradingAccount` | `void` | `CThostFtdcTradingAccountField *pTradingAccount, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询资金账户响应 |
| `OnRspQryInvestor` | `void` | `CThostFtdcInvestorField *pInvestor, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询投资者响应 |
| `OnRspQryTradingCode` | `void` | `CThostFtdcTradingCodeField *pTradingCode, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询交易编码响应 |
| `OnRspQryInstrumentMarginRate` | `void` | `CThostFtdcInstrumentMarginRateField *pInstrumentMarginRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询合约保证金率响应 |
| `OnRspQryInstrumentCommissionRate` | `void` | `CThostFtdcInstrumentCommissionRateField *pInstrumentCommissionRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询合约手续费率响应 |
| `OnRspQryUserSession` | `void` | `CThostFtdcUserSessionField *pUserSession, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询用户会话响应 |
| `OnRspQryExchange` | `void` | `CThostFtdcExchangeField *pExchange, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询交易所响应 |
| `OnRspQryProduct` | `void` | `CThostFtdcProductField *pProduct, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询产品响应 |
| `OnRspQryInstrument` | `void` | `CThostFtdcInstrumentField *pInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询合约响应 |
| `OnRspQryDepthMarketData` | `void` | `CThostFtdcDepthMarketDataField *pDepthMarketData, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询行情响应 |
| `OnRspQryTraderOffer` | `void` | `CThostFtdcTraderOfferField *pTraderOffer, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询交易员报盘机响应 |
| `OnRspQrySettlementInfo` | `void` | `CThostFtdcSettlementInfoField *pSettlementInfo, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询投资者结算结果响应 |
| `OnRspQryTransferBank` | `void` | `CThostFtdcTransferBankField *pTransferBank, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询转帐银行响应 |
| `OnRspQryInvestorPositionDetail` | `void` | `CThostFtdcInvestorPositionDetailField *pInvestorPositionDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询投资者持仓明细响应 |
| `OnRspQryNotice` | `void` | `CThostFtdcNoticeField *pNotice, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询客户通知响应 |
| `OnRspQrySettlementInfoConfirm` | `void` | `CThostFtdcSettlementInfoConfirmField *pSettlementInfoConfirm, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询结算信息确认响应 |
| `OnRspQryInvestorPositionCombineDetail` | `void` | `CThostFtdcInvestorPositionCombineDetailField *pInvestorPositionCombineDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询投资者持仓明细响应 |
| `OnRspQryCFMMCTradingAccountKey` | `void` | `CThostFtdcCFMMCTradingAccountKeyField *pCFMMCTradingAccountKey, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 查询保证金监管系统经纪公司资金账户密钥响应 |
| `OnRspQryEWarrantOffset` | `void` | `CThostFtdcEWarrantOffsetField *pEWarrantOffset, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询仓单折抵信息响应 |
| `OnRspQryInvestorProductGroupMargin` | `void` | `CThostFtdcInvestorProductGroupMarginField *pInvestorProductGroupMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询投资者品种/跨品种保证金响应 |
| `OnRspQryExchangeMarginRate` | `void` | `CThostFtdcExchangeMarginRateField *pExchangeMarginRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询交易所保证金率响应 |
| `OnRspQryExchangeMarginRateAdjust` | `void` | `CThostFtdcExchangeMarginRateAdjustField *pExchangeMarginRateAdjust, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询交易所调整保证金率响应 |
| `OnRspQryExchangeRate` | `void` | `CThostFtdcExchangeRateField *pExchangeRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询汇率响应 |
| `OnRspQrySecAgentACIDMap` | `void` | `CThostFtdcSecAgentACIDMapField *pSecAgentACIDMap, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询二级代理操作员银期权限响应 |
| `OnRspQryProductExchRate` | `void` | `CThostFtdcProductExchRateField *pProductExchRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询产品报价汇率 |
| `OnRspQryProductGroup` | `void` | `CThostFtdcProductGroupField *pProductGroup, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询产品组 |
| `OnRspQryMMInstrumentCommissionRate` | `void` | `CThostFtdcMMInstrumentCommissionRateField *pMMInstrumentCommissionRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询做市商合约手续费率响应 |
| `OnRspQryMMOptionInstrCommRate` | `void` | `CThostFtdcMMOptionInstrCommRateField *pMMOptionInstrCommRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询做市商期权合约手续费响应 |
| `OnRspQryInstrumentOrderCommRate` | `void` | `CThostFtdcInstrumentOrderCommRateField *pInstrumentOrderCommRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询报单手续费响应 |
| `OnRspQrySecAgentTradingAccount` | `void` | `CThostFtdcTradingAccountField *pTradingAccount, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询资金账户响应 |
| `OnRspQrySecAgentCheckMode` | `void` | `CThostFtdcSecAgentCheckModeField *pSecAgentCheckMode, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询二级代理商资金校验模式响应 |
| `OnRspQrySecAgentTradeInfo` | `void` | `CThostFtdcSecAgentTradeInfoField *pSecAgentTradeInfo, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询二级代理商信息响应 |
| `OnRspQryOptionInstrTradeCost` | `void` | `CThostFtdcOptionInstrTradeCostField *pOptionInstrTradeCost, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询期权交易成本响应 |
| `OnRspQryOptionInstrCommRate` | `void` | `CThostFtdcOptionInstrCommRateField *pOptionInstrCommRate, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询期权合约手续费响应 |
| `OnRspQryExecOrder` | `void` | `CThostFtdcExecOrderField *pExecOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询执行宣告响应 |
| `OnRspQryForQuote` | `void` | `CThostFtdcForQuoteField *pForQuote, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询询价响应 |
| `OnRspQryQuote` | `void` | `CThostFtdcQuoteField *pQuote, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询报价响应 |
| `OnRspQryOptionSelfClose` | `void` | `CThostFtdcOptionSelfCloseField *pOptionSelfClose, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询期权自对冲响应 |
| `OnRspQryInvestUnit` | `void` | `CThostFtdcInvestUnitField *pInvestUnit, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询投资单元响应 |
| `OnRspQryCombInstrumentGuard` | `void` | `CThostFtdcCombInstrumentGuardField *pCombInstrumentGuard, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询组合合约安全系数响应 |
| `OnRspQryCombAction` | `void` | `CThostFtdcCombActionField *pCombAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询申请组合响应 |
| `OnRspQryTransferSerial` | `void` | `CThostFtdcTransferSerialField *pTransferSerial, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询转帐流水响应 |
| `OnRspQryAccountregister` | `void` | `CThostFtdcAccountregisterField *pAccountregister, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询银期签约关系响应 |
| `OnRspError` | `void` | `CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 错误应答 |
| `OnRtnOrder` | `void` | `CThostFtdcOrderField *pOrder` | 报单通知 |
| `OnRtnTrade` | `void` | `CThostFtdcTradeField *pTrade` | 成交通知 |
| `OnErrRtnOrderInsert` | `void` | `CThostFtdcInputOrderField *pInputOrder, CThostFtdcRspInfoField *pRspInfo` | 报单录入错误回报 |
| `OnErrRtnOrderAction` | `void` | `CThostFtdcOrderActionField *pOrderAction, CThostFtdcRspInfoField *pRspInfo` | 报单操作错误回报 |
| `OnRtnInstrumentStatus` | `void` | `CThostFtdcInstrumentStatusField *pInstrumentStatus` | 合约交易状态通知 |
| `OnRtnBulletin` | `void` | `CThostFtdcBulletinField *pBulletin` | 交易所公告通知 |
| `OnRtnTradingNotice` | `void` | `CThostFtdcTradingNoticeInfoField *pTradingNoticeInfo` | 交易通知 |
| `OnRtnErrorConditionalOrder` | `void` | `CThostFtdcErrorConditionalOrderField *pErrorConditionalOrder` | 提示条件单校验错误 |
| `OnRtnExecOrder` | `void` | `CThostFtdcExecOrderField *pExecOrder` | 执行宣告通知 |
| `OnErrRtnExecOrderInsert` | `void` | `CThostFtdcInputExecOrderField *pInputExecOrder, CThostFtdcRspInfoField *pRspInfo` | 执行宣告录入错误回报 |
| `OnErrRtnExecOrderAction` | `void` | `CThostFtdcExecOrderActionField *pExecOrderAction, CThostFtdcRspInfoField *pRspInfo` | 执行宣告操作错误回报 |
| `OnErrRtnForQuoteInsert` | `void` | `CThostFtdcInputForQuoteField *pInputForQuote, CThostFtdcRspInfoField *pRspInfo` | 询价录入错误回报 |
| `OnRtnQuote` | `void` | `CThostFtdcQuoteField *pQuote` | 报价通知 |
| `OnErrRtnQuoteInsert` | `void` | `CThostFtdcInputQuoteField *pInputQuote, CThostFtdcRspInfoField *pRspInfo` | 报价录入错误回报 |
| `OnErrRtnQuoteAction` | `void` | `CThostFtdcQuoteActionField *pQuoteAction, CThostFtdcRspInfoField *pRspInfo` | 报价操作错误回报 |
| `OnRtnForQuoteRsp` | `void` | `CThostFtdcForQuoteRspField *pForQuoteRsp` | 询价通知 |
| `OnRtnCFMMCTradingAccountToken` | `void` | `CThostFtdcCFMMCTradingAccountTokenField *pCFMMCTradingAccountToken` | 保证金监控中心用户令牌 |
| `OnErrRtnBatchOrderAction` | `void` | `CThostFtdcBatchOrderActionField *pBatchOrderAction, CThostFtdcRspInfoField *pRspInfo` | 批量报单操作错误回报 |
| `OnRtnOptionSelfClose` | `void` | `CThostFtdcOptionSelfCloseField *pOptionSelfClose` | 期权自对冲通知 |
| `OnErrRtnOptionSelfCloseInsert` | `void` | `CThostFtdcInputOptionSelfCloseField *pInputOptionSelfClose, CThostFtdcRspInfoField *pRspInfo` | 期权自对冲录入错误回报 |
| `OnErrRtnOptionSelfCloseAction` | `void` | `CThostFtdcOptionSelfCloseActionField *pOptionSelfCloseAction, CThostFtdcRspInfoField *pRspInfo` | 期权自对冲操作错误回报 |
| `OnRtnCombAction` | `void` | `CThostFtdcCombActionField *pCombAction` | 申请组合通知 |
| `OnErrRtnCombActionInsert` | `void` | `CThostFtdcInputCombActionField *pInputCombAction, CThostFtdcRspInfoField *pRspInfo` | 申请组合录入错误回报 |
| `OnRspQryContractBank` | `void` | `CThostFtdcContractBankField *pContractBank, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询签约银行响应 |
| `OnRspQryParkedOrder` | `void` | `CThostFtdcParkedOrderField *pParkedOrder, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询预埋单响应 |
| `OnRspQryParkedOrderAction` | `void` | `CThostFtdcParkedOrderActionField *pParkedOrderAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询预埋撤单响应 |
| `OnRspQryTradingNotice` | `void` | `CThostFtdcTradingNoticeField *pTradingNotice, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询交易通知响应 |
| `OnRspQryBrokerTradingParams` | `void` | `CThostFtdcBrokerTradingParamsField *pBrokerTradingParams, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询经纪公司交易参数响应 |
| `OnRspQryBrokerTradingAlgos` | `void` | `CThostFtdcBrokerTradingAlgosField *pBrokerTradingAlgos, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询经纪公司交易算法响应 |
| `OnRspQueryCFMMCTradingAccountToken` | `void` | `CThostFtdcQueryCFMMCTradingAccountTokenField *pQueryCFMMCTradingAccountToken, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询监控中心用户令牌 |
| `OnRtnFromBankToFutureByBank` | `void` | `CThostFtdcRspTransferField *pRspTransfer` | 银行发起银行资金转期货通知 |
| `OnRtnFromFutureToBankByBank` | `void` | `CThostFtdcRspTransferField *pRspTransfer` | 银行发起期货资金转银行通知 |
| `OnRtnRepealFromBankToFutureByBank` | `void` | `CThostFtdcRspRepealField *pRspRepeal` | 银行发起冲正银行转期货通知 |
| `OnRtnRepealFromFutureToBankByBank` | `void` | `CThostFtdcRspRepealField *pRspRepeal` | 银行发起冲正期货转银行通知 |
| `OnRtnFromBankToFutureByFuture` | `void` | `CThostFtdcRspTransferField *pRspTransfer` | 期货发起银行资金转期货通知 |
| `OnRtnFromFutureToBankByFuture` | `void` | `CThostFtdcRspTransferField *pRspTransfer` | 期货发起期货资金转银行通知 |
| `OnRtnRepealFromBankToFutureByFutureManual` | `void` | `CThostFtdcRspRepealField *pRspRepeal` | 系统运行时期货端手工发起冲正银行转期货请求，银行处理完毕后报盘发回的通知 |
| `OnRtnRepealFromFutureToBankByFutureManual` | `void` | `CThostFtdcRspRepealField *pRspRepeal` | 系统运行时期货端手工发起冲正期货转银行请求，银行处理完毕后报盘发回的通知 |
| `OnRtnQueryBankBalanceByFuture` | `void` | `CThostFtdcNotifyQueryAccountField *pNotifyQueryAccount` | 期货发起查询银行余额通知 |
| `OnErrRtnBankToFutureByFuture` | `void` | `CThostFtdcReqTransferField *pReqTransfer, CThostFtdcRspInfoField *pRspInfo` | 期货发起银行资金转期货错误回报 |
| `OnErrRtnFutureToBankByFuture` | `void` | `CThostFtdcReqTransferField *pReqTransfer, CThostFtdcRspInfoField *pRspInfo` | 期货发起期货资金转银行错误回报 |
| `OnErrRtnRepealBankToFutureByFutureManual` | `void` | `CThostFtdcReqRepealField *pReqRepeal, CThostFtdcRspInfoField *pRspInfo` | 系统运行时期货端手工发起冲正银行转期货错误回报 |
| `OnErrRtnRepealFutureToBankByFutureManual` | `void` | `CThostFtdcReqRepealField *pReqRepeal, CThostFtdcRspInfoField *pRspInfo` | 系统运行时期货端手工发起冲正期货转银行错误回报 |
| `OnErrRtnQueryBankBalanceByFuture` | `void` | `CThostFtdcReqQueryAccountField *pReqQueryAccount, CThostFtdcRspInfoField *pRspInfo` | 期货发起查询银行余额错误回报 |
| `OnRtnRepealFromBankToFutureByFuture` | `void` | `CThostFtdcRspRepealField *pRspRepeal` | 期货发起冲正银行转期货请求，银行处理完毕后报盘发回的通知 |
| `OnRtnRepealFromFutureToBankByFuture` | `void` | `CThostFtdcRspRepealField *pRspRepeal` | 期货发起冲正期货转银行请求，银行处理完毕后报盘发回的通知 |
| `OnRspFromBankToFutureByFuture` | `void` | `CThostFtdcReqTransferField *pReqTransfer, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 期货发起银行资金转期货应答 |
| `OnRspFromFutureToBankByFuture` | `void` | `CThostFtdcReqTransferField *pReqTransfer, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 期货发起期货资金转银行应答 |
| `OnRspQueryBankAccountMoneyByFuture` | `void` | `CThostFtdcReqQueryAccountField *pReqQueryAccount, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 期货发起查询银行余额应答 |
| `OnRtnOpenAccountByBank` | `void` | `CThostFtdcOpenAccountField *pOpenAccount` | 银行发起银期开户通知 |
| `OnRtnCancelAccountByBank` | `void` | `CThostFtdcCancelAccountField *pCancelAccount` | 银行发起银期销户通知 |
| `OnRtnChangeAccountByBank` | `void` | `CThostFtdcChangeAccountField *pChangeAccount` | 银行发起变更银行账号通知 |
| `OnRspQryClassifiedInstrument` | `void` | `CThostFtdcInstrumentField *pInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询分类合约响应 |
| `OnRspQryCombPromotionParam` | `void` | `CThostFtdcCombPromotionParamField *pCombPromotionParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求组合优惠比例响应 |
| `OnRspQryRiskSettleInvstPosition` | `void` | `CThostFtdcRiskSettleInvstPositionField *pRiskSettleInvstPosition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者风险结算持仓查询响应 |
| `OnRspQryRiskSettleProductStatus` | `void` | `CThostFtdcRiskSettleProductStatusField *pRiskSettleProductStatus, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 风险结算产品查询响应 |
| `OnRspQrySPBMFutureParameter` | `void` | `CThostFtdcSPBMFutureParameterField *pSPBMFutureParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | SPBM期货合约参数查询响应 |
| `OnRspQrySPBMOptionParameter` | `void` | `CThostFtdcSPBMOptionParameterField *pSPBMOptionParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | SPBM期权合约参数查询响应 |
| `OnRspQrySPBMIntraParameter` | `void` | `CThostFtdcSPBMIntraParameterField *pSPBMIntraParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | SPBM品种内对锁仓折扣参数查询响应 |
| `OnRspQrySPBMInterParameter` | `void` | `CThostFtdcSPBMInterParameterField *pSPBMInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | SPBM跨品种抵扣参数查询响应 |
| `OnRspQrySPBMPortfDefinition` | `void` | `CThostFtdcSPBMPortfDefinitionField *pSPBMPortfDefinition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | SPBM组合保证金套餐查询响应 |
| `OnRspQrySPBMInvestorPortfDef` | `void` | `CThostFtdcSPBMInvestorPortfDefField *pSPBMInvestorPortfDef, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者SPBM套餐选择查询响应 |
| `OnRspQryInvestorPortfMarginRatio` | `void` | `CThostFtdcInvestorPortfMarginRatioField *pInvestorPortfMarginRatio, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者新型组合保证金系数查询响应 |
| `OnRspQryInvestorProdSPBMDetail` | `void` | `CThostFtdcInvestorProdSPBMDetailField *pInvestorProdSPBMDetail, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者产品SPBM明细查询响应 |
| `OnRspQryInvestorCommoditySPMMMargin` | `void` | `CThostFtdcInvestorCommoditySPMMMarginField *pInvestorCommoditySPMMMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者商品组SPMM记录查询响应 |
| `OnRspQryInvestorCommodityGroupSPMMMargin` | `void` | `CThostFtdcInvestorCommodityGroupSPMMMarginField *pInvestorCommodityGroupSPMMMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者商品群SPMM记录查询响应 |
| `OnRspQrySPMMInstParam` | `void` | `CThostFtdcSPMMInstParamField *pSPMMInstParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | SPMM合约参数查询响应 |
| `OnRspQrySPMMProductParam` | `void` | `CThostFtdcSPMMProductParamField *pSPMMProductParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | SPMM产品参数查询响应 |
| `OnRspQrySPBMAddOnInterParameter` | `void` | `CThostFtdcSPBMAddOnInterParameterField *pSPBMAddOnInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | SPBM附加跨品种抵扣参数查询响应 |
| `OnRspQryRCAMSCombProductInfo` | `void` | `CThostFtdcRCAMSCombProductInfoField *pRCAMSCombProductInfo, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | RCAMS产品组合信息查询响应 |
| `OnRspQryRCAMSInstrParameter` | `void` | `CThostFtdcRCAMSInstrParameterField *pRCAMSInstrParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | RCAMS同合约风险对冲参数查询响应 |
| `OnRspQryRCAMSIntraParameter` | `void` | `CThostFtdcRCAMSIntraParameterField *pRCAMSIntraParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | RCAMS品种内风险对冲参数查询响应 |
| `OnRspQryRCAMSInterParameter` | `void` | `CThostFtdcRCAMSInterParameterField *pRCAMSInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | RCAMS跨品种风险折抵参数查询响应 |
| `OnRspQryRCAMSShortOptAdjustParam` | `void` | `CThostFtdcRCAMSShortOptAdjustParamField *pRCAMSShortOptAdjustParam, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | RCAMS空头期权风险调整参数查询响应 |
| `OnRspQryRCAMSInvestorCombPosition` | `void` | `CThostFtdcRCAMSInvestorCombPositionField *pRCAMSInvestorCombPosition, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | RCAMS策略组合持仓查询响应 |
| `OnRspQryInvestorProdRCAMSMargin` | `void` | `CThostFtdcInvestorProdRCAMSMarginField *pInvestorProdRCAMSMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者品种RCAMS保证金查询响应 |
| `OnRspQryRULEInstrParameter` | `void` | `CThostFtdcRULEInstrParameterField *pRULEInstrParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | RULE合约保证金参数查询响应 |
| `OnRspQryRULEIntraParameter` | `void` | `CThostFtdcRULEIntraParameterField *pRULEIntraParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | RULE品种内对锁仓折扣参数查询响应 |
| `OnRspQryRULEInterParameter` | `void` | `CThostFtdcRULEInterParameterField *pRULEInterParameter, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | RULE跨品种抵扣参数查询响应 |
| `OnRspQryInvestorProdRULEMargin` | `void` | `CThostFtdcInvestorProdRULEMarginField *pInvestorProdRULEMargin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者产品RULE保证金查询响应 |
| `OnRspQryInvestorPortfSetting` | `void` | `CThostFtdcInvestorPortfSettingField *pInvestorPortfSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者新型组合保证金开关查询响应 |
| `OnRspQryInvestorInfoCommRec` | `void` | `CThostFtdcInvestorInfoCommRecField *pInvestorInfoCommRec, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者申报费阶梯收取记录查询响应 |
| `OnRspQryCombLeg` | `void` | `CThostFtdcCombLegField *pCombLeg, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 组合腿信息查询响应 |
| `OnRspOffsetSetting` | `void` | `CThostFtdcInputOffsetSettingField *pInputOffsetSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 对冲设置请求响应 |
| `OnRspCancelOffsetSetting` | `void` | `CThostFtdcInputOffsetSettingField *pInputOffsetSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 对冲设置撤销请求响应 |
| `OnRtnOffsetSetting` | `void` | `CThostFtdcOffsetSettingField *pOffsetSetting` | 对冲设置通知 |
| `OnErrRtnOffsetSetting` | `void` | `CThostFtdcInputOffsetSettingField *pInputOffsetSetting, CThostFtdcRspInfoField *pRspInfo` | 对冲设置错误回报 |
| `OnErrRtnCancelOffsetSetting` | `void` | `CThostFtdcCancelOffsetSettingField *pCancelOffsetSetting, CThostFtdcRspInfoField *pRspInfo` | 对冲设置撤销错误回报 |
| `OnRspQryOffsetSetting` | `void` | `CThostFtdcOffsetSettingField *pOffsetSetting, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 投资者对冲设置查询响应 |
| `OnRspGenSMSCode` | `void` | `CThostFtdcRspGenSMSCodeField *pRspGenSMSCode, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 申请短信验证码响应 |
| `OnRspSpdApply` | `void` | `CThostFtdcInputSpdApplyField *pInputSpdApply, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 套利确认回复 |
| `OnRspSpdApplyAction` | `void` | `CThostFtdcInputSpdApplyActionField *pInputSpdApplyAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 套利确认撤销回复 |
| `OnRspQrySpdApply` | `void` | `CThostFtdcSpdApplyField *pSpdApply, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 套利确认查询回复 |
| `OnRtnSpdApply` | `void` | `CThostFtdcSpdApplyField *pSpdApply` | 套利确认通知 |
| `OnErrRtnSpdApply` | `void` | `CThostFtdcInputSpdApplyField *pInputSpdApply, CThostFtdcRspInfoField *pRspInfo` | 套利申请录入错误回报 |
| `OnErrRtnSpdApplyAction` | `void` | `CThostFtdcSpdApplyActionField *pSpdApplyAction, CThostFtdcRspInfoField *pRspInfo` | 套利确认撤销通知 |
| `OnRspHedgeCfm` | `void` | `CThostFtdcInputHedgeCfmField *pInputHedgeCfm, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 套保确认回复 |
| `OnRspHedgeCfmAction` | `void` | `CThostFtdcInputHedgeCfmActionField *pInputHedgeCfmAction, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 套保确认撤销回复 |
| `OnRspQryHedgeCfm` | `void` | `CThostFtdcHedgeCfmField *pHedgeCfm, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 套保确认查询回复 |
| `OnRtnHedgeCfm` | `void` | `CThostFtdcHedgeCfmField *pHedgeCfm` | 套保确认通知 |
| `OnErrRtnHedgeCfm` | `void` | `CThostFtdcInputHedgeCfmField *pInputHedgeCfm, CThostFtdcRspInfoField *pRspInfo` | 套保额度录入错误回报 |
| `OnErrRtnHedgeCfmAction` | `void` | `CThostFtdcHedgeCfmActionField *pHedgeCfmAction, CThostFtdcRspInfoField *pRspInfo` | 套保确认撤销通知 |

## CThostFtdcMdApi（行情 API 主动调用）（15）

| 方法 | 返回 | 参数 | 说明 |
|---|---|---|---|
| `CreateFtdcMdApi` | `CThostFtdcMdApi *` | `const char *pszFlowPath = "", const bool bIsUsingUdp=false, const bool bIsMulticast=false, bool bIsProductionMode=true` | 创建MdApi @param pszFlowPath 存贮订阅信息文件的目录，默认为当前目录 @param bIsProductionMode true:使用生产版本的API  false:使用测评版本API @return 创建出的UserApi modify for udp marketdata |
| `Release` | `void` | `` | 删除接口对象本身 @remark 不再使用本接口对象时,调用该函数删除接口对象 |
| `Init` | `void` | `` | 初始化 @remark 初始化运行环境,只有调用后,接口才开始工作 |
| `Join` | `int` | `` | 等待接口线程结束运行 @return 线程退出代码 |
| `RegisterFront` | `void` | `char *pszFrontAddress` | 注册前置机网络地址 @param pszFrontAddress：前置机网络地址。 @remark 网络地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:17001”。 @remark “tcp”代表传输协议，“127.0.0.1”代表服务器地址。”17001”代表服务器端口号。 |
| `RegisterNameServer` | `void` | `char *pszNsAddress` | 注册名字服务器网络地址 @param pszNsAddress：名字服务器网络地址。 @remark 网络地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:12001”。 @remark “tcp”代表传输协议，“127.0.0.1”代表服务器地址。”12001”代表服务器端口号。 @remark RegisterNameServer优先于RegisterFront |
| `RegisterFensUserInfo` | `void` | `CThostFtdcFensUserInfoField * pFensUserInfo` | 注册名字服务器用户信息 @param pFensUserInfo：用户信息。 |
| `RegisterSpi` | `void` | `CThostFtdcMdSpi *pSpi` | 注册回调接口 @param pSpi 派生自回调接口类的实例 |
| `SubscribeMarketData` | `int` | `char *ppInstrumentID[], int nCount` | 订阅行情。 @param ppInstrumentID 合约ID @param nCount 要订阅/退订行情的合约个数 @remark |
| `UnSubscribeMarketData` | `int` | `char *ppInstrumentID[], int nCount` | 退订行情。 @param ppInstrumentID 合约ID @param nCount 要订阅/退订行情的合约个数 @remark |
| `SubscribeForQuoteRsp` | `int` | `char *ppInstrumentID[], int nCount` | 订阅询价。 @param ppInstrumentID 合约ID @param nCount 要订阅/退订行情的合约个数 @remark |
| `UnSubscribeForQuoteRsp` | `int` | `char *ppInstrumentID[], int nCount` | 退订询价。 @param ppInstrumentID 合约ID @param nCount 要订阅/退订行情的合约个数 @remark |
| `ReqUserLogin` | `int` | `CThostFtdcReqUserLoginField *pReqUserLoginField, int nRequestID` | 用户登录请求 |
| `ReqUserLogout` | `int` | `CThostFtdcUserLogoutField *pUserLogout, int nRequestID` | 登出请求 |
| `ReqQryMulticastInstrument` | `int` | `CThostFtdcQryMulticastInstrumentField *pQryMulticastInstrument, int nRequestID` | 请求查询组播合约 |

## CThostFtdcMdSpi（行情 SPI 回调）（13）

| 方法 | 返回 | 参数 | 说明 |
|---|---|---|---|
| `OnFrontConnected` | `void` | `` | 当客户端与交易后台建立起通信连接时（还未登录前），该方法被调用。 |
| `OnFrontDisconnected` | `void` | `int nReason` | 当客户端与交易后台通信连接断开时，该方法被调用。当发生这个情况后，API会自动重新连接，客户端可不做处理。 @param nReason 错误原因         0x1001 网络读失败         0x1002 网络写失败         0x2001 接收心跳超时         0x2002 发送心跳失败         0x2003 收到错误报文 |
| `OnHeartBeatWarning` | `void` | `int nTimeLapse` | 心跳超时警告。当长时间未收到报文时，该方法被调用。 @param nTimeLapse 距离上次接收报文的时间 |
| `OnRspUserLogin` | `void` | `CThostFtdcRspUserLoginField *pRspUserLogin, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 登录请求响应 |
| `OnRspUserLogout` | `void` | `CThostFtdcUserLogoutField *pUserLogout, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 登出请求响应 |
| `OnRspQryMulticastInstrument` | `void` | `CThostFtdcMulticastInstrumentField *pMulticastInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 请求查询组播合约响应 |
| `OnRspError` | `void` | `CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 错误应答 |
| `OnRspSubMarketData` | `void` | `CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 订阅行情应答 |
| `OnRspUnSubMarketData` | `void` | `CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 取消订阅行情应答 |
| `OnRspSubForQuoteRsp` | `void` | `CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 订阅询价应答 |
| `OnRspUnSubForQuoteRsp` | `void` | `CThostFtdcSpecificInstrumentField *pSpecificInstrument, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast` | 取消订阅询价应答 |
| `OnRtnDepthMarketData` | `void` | `CThostFtdcDepthMarketDataField *pDepthMarketData` | 深度行情通知 |
| `OnRtnForQuoteRsp` | `void` | `CThostFtdcForQuoteRspField *pForQuoteRsp` | 询价通知 |
