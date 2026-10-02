# CThostFtdcTraderApi

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

CThostFtdcTraderApi<a id="content"></a>

<a id="left_menu"></a>

  ** **

|  | ■ 6.7.13_API接口说明
└△ 交易接口
　└◆ CThostFtdcTraderApi |  |
|---|---|---|

CThostFtdcTraderApi类提供了交易api的初始化、登录、报单和查询等功能。
<a id="1a7a059d-2b72-40a1-ac95-5cf73bfd2eb3"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 接口
<a id="panel1"></a>

```
class TRADER_API_EXPORT CThostFtdcTraderApi
{
public:
    ///创建TraderApi
    ///@param pszFlowPath 存贮订阅信息文件的目录，默认为当前目录
    ///@return 创建出的UserApi
    static CThostFtdcTraderApi *CreateFtdcTraderApi(const char *pszFlowPath = "", bool bIsProductionMode = true);
    ///获取API的版本信息
    ///@retrun 获取到的版本号
    static const char *GetApiVersion();
    ///删除接口对象本身
    ///@remark 不再使用本接口对象时,调用该函数删除接口对象
    virtual void Release() = 0;
    ///初始化
    ///@remark 初始化运行环境,只有调用后,接口才开始工作
    virtual void Init() = 0;
    ///等待接口线程结束运行
    ///@return 线程退出代码
    virtual int Join() = 0;
    ///获取当前交易日
    ///@retrun 获取到的交易日
    ///@remark 只有登录成功后,才能得到正确的交易日
    virtual const char *GetTradingDay() = 0;
    ///获取已连接的前置的信息
    /// @param pFrontInfo：输入输出参数，用于存储获取到的前置信息，不能为空
    /// @remark 连接成功后，可获取正确的前置地址信息
    /// @remark 登录成功后，可获取正确的前置流控信息
    virtual void GetFrontInfo(CThostFtdcFrontInfoField* pFrontInfo) =0;
    ///注册前置机网络地址
    ///@param pszFrontAddress：前置机网络地址。
    ///@remark 网络地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:17001”。
    ///@remark “tcp”代表传输协议，“127.0.0.1”代表服务器地址。”17001”代表服务器端口号。
    virtual void RegisterFront(char *pszFrontAddress) = 0;
    ///注册名字服务器网络地址
    ///@param pszNsAddress：名字服务器网络地址。
    ///@remark 网络地址的格式为：“protocol://ipaddress:port”，如：”tcp://127.0.0.1:12001”。
    ///@remark “tcp”代表传输协议，“127.0.0.1”代表服务器地址。”12001”代表服务器端口号。
    ///@remark RegisterNameServer优先于RegisterFront
    virtual void RegisterNameServer(char *pszNsAddress) = 0;
    ///注册名字服务器用户信息
    ///@param pFensUserInfo：用户信息。
    virtual void RegisterFensUserInfo(CThostFtdcFensUserInfoField * pFensUserInfo) = 0;
    ///注册回调接口
    ///@param pSpi 派生自回调接口类的实例
    virtual void RegisterSpi(CThostFtdcTraderSpi *pSpi) = 0;
    ///订阅私有流。
    ///@param nResumeType 私有流重传方式
    ///        THOST_TERT_RESTART:从本交易日开始重传
    ///        THOST_TERT_RESUME:从上次收到的续传
    ///        THOST_TERT_QUICK:只传送登录后私有流的内容
    ///@remark 该方法要在Init方法前调用。若不调用则不会收到私有流的数据。
    virtual void SubscribePrivateTopic(THOST_TE_RESUME_TYPE nResumeType) = 0;
    ///订阅公共流。
    ///@param nResumeType 公共流重传方式
    ///        THOST_TERT_RESTART:从本交易日开始重传
    ///        THOST_TERT_RESUME:从上次收到的续传
    ///        THOST_TERT_QUICK:只传送登录后公共流的内容
    ///        THOST_TERT_NONE:取消订阅公共流
    ///@remark 该方法要在Init方法前调用。若不调用则不会收到公共流的数据。
    virtual void SubscribePublicTopic(THOST_TE_RESUME_TYPE nResumeType) = 0;
    ///客户端认证请求
    virtual int ReqAuthenticate(CThostFtdcReqAuthenticateField *pReqAuthenticateField, int nRequestID) = 0;
    ///注册用户终端信息，用于中继服务器多连接模式
    ///需要在终端认证成功后，用户登录前调用该接口
    virtual int RegisterUserSystemInfo(CThostFtdcUserSystemInfoField *pUserSystemInfo) = 0;
    ///上报用户终端信息，用于中继服务器操作员登录模式
    ///操作员登录后，可以多次调用该接口上报客户信息
    virtual int SubmitUserSystemInfo(CThostFtdcUserSystemInfoField *pUserSystemInfo) = 0;
    ///注册用户终端信息，用于中继服务器多连接模式.用于微信小程序等应用上报信息.
    virtual int RegisterWechatUserSystemInfo(CThostFtdcWechatUserSystemInfoField *pUserSystemInfo) = 0;
    ///上报用户终端信息，用于中继服务器操作员登录模式.用于微信小程序等应用上报信息.
    virtual int SubmitWechatUserSystemInfo(CThostFtdcWechatUserSystemInfoField *pUserSystemInfo) = 0;
    ///用户登录请求
    virtual int ReqUserLogin(CThostFtdcReqUserLoginField *pReqUserLoginField, int nRequestID) = 0;
    ///登出请求
    virtual int ReqUserLogout(CThostFtdcUserLogoutField *pUserLogout, int nRequestID) = 0;
    ///用户口令更新请求
    virtual int ReqUserPasswordUpdate(CThostFtdcUserPasswordUpdateField *pUserPasswordUpdate, int nRequestID) = 0;
    ///资金账户口令更新请求
    virtual int ReqTradingAccountPasswordUpdate(CThostFtdcTradingAccountPasswordUpdateField *pTradingAccountPasswordUpdate, int nRequestID) = 0;
    ///查询用户当前支持的认证模式
    virtual int ReqUserAuthMethod(CThostFtdcReqUserAuthMethodField *pReqUserAuthMethod, int nRequestID) = 0;
    ///用户发出获取图形验证码请求
    virtual int ReqGenUserCaptcha(CThostFtdcReqGenUserCaptchaField *pReqGenUserCaptcha, int nRequestID) = 0;
    ///用户发出获取短信验证码请求
    virtual int ReqGenUserText(CThostFtdcReqGenUserTextField *pReqGenUserText, int nRequestID) = 0;
    ///用户发出带有图片验证码的登陆请求
    virtual int ReqUserLoginWithCaptcha(CThostFtdcReqUserLoginWithCaptchaField *pReqUserLoginWithCaptcha, int nRequestID) = 0;
    ///用户发出带有短信验证码的登陆请求
    virtual int ReqUserLoginWithText(CThostFtdcReqUserLoginWithTextField *pReqUserLoginWithText, int nRequestID) = 0;
    ///用户发出带有动态口令的登陆请求
    virtual int ReqUserLoginWithOTP(CThostFtdcReqUserLoginWithOTPField *pReqUserLoginWithOTP, int nRequestID) = 0;
    ///报单录入请求
    virtual int ReqOrderInsert(CThostFtdcInputOrderField *pInputOrder, int nRequestID) = 0;
    ///预埋单录入请求
    virtual int ReqParkedOrderInsert(CThostFtdcParkedOrderField *pParkedOrder, int nRequestID) = 0;
    ///预埋撤单录入请求
    virtual int ReqParkedOrderAction(CThostFtdcParkedOrderActionField *pParkedOrderAction, int nRequestID) = 0;
    ///报单操作请求
    virtual int ReqOrderAction(CThostFtdcInputOrderActionField *pInputOrderAction, int nRequestID) = 0;
    ///查询最大报单数量请求
    virtual int ReqQryMaxOrderVolume(CThostFtdcQryMaxOrderVolumeField *pQryMaxOrderVolume, int nRequestID) = 0;
    ///投资者结算结果确认
    virtual int ReqSettlementInfoConfirm(CThostFtdcSettlementInfoConfirmField *pSettlementInfoConfirm, int nRequestID) = 0;
    ///请求删除预埋单
    virtual int ReqRemoveParkedOrder(CThostFtdcRemoveParkedOrderField *pRemoveParkedOrder, int nRequestID) = 0;
    ///请求删除预埋撤单
    virtual int ReqRemoveParkedOrderAction(CThostFtdcRemoveParkedOrderActionField *pRemoveParkedOrderAction, int nRequestID) = 0;
    ///执行宣告录入请求
    virtual int ReqExecOrderInsert(CThostFtdcInputExecOrderField *pInputExecOrder, int nRequestID) = 0;
    ///执行宣告操作请求
    virtual int ReqExecOrderAction(CThostFtdcInputExecOrderActionField *pInputExecOrderAction, int nRequestID) = 0;
    ///询价录入请求
    virtual int ReqForQuoteInsert(CThostFtdcInputForQuoteField *pInputForQuote, int nRequestID) = 0;
    ///报价录入请求
    virtual int ReqQuoteInsert(CThostFtdcInputQuoteField *pInputQuote, int nRequestID) = 0;
    ///报价操作请求
    virtual int ReqQuoteAction(CThostFtdcInputQuoteActionField *pInputQuoteAction, int nRequestID) = 0;
    ///批量报单操作请求
    virtual int ReqBatchOrderAction(CThostFtdcInputBatchOrderActionField *pInputBatchOrderAction, int nRequestID) = 0;
    ///期权自对冲录入请求
    virtual int ReqOptionSelfCloseInsert(CThostFtdcInputOptionSelfCloseField *pInputOptionSelfClose, int nRequestID) = 0;
    ///期权自对冲操作请求
    virtual int ReqOptionSelfCloseAction(CThostFtdcInputOptionSelfCloseActionField *pInputOptionSelfCloseAction, int nRequestID) = 0;
    ///申请组合录入请求
    virtual int ReqCombActionInsert(CThostFtdcInputCombActionField *pInputCombAction, int nRequestID) = 0;
    ///请求查询报单
    virtual int ReqQryOrder(CThostFtdcQryOrderField *pQryOrder, int nRequestID) = 0;
    ///请求查询成交
    virtual int ReqQryTrade(CThostFtdcQryTradeField *pQryTrade, int nRequestID) = 0;
    ///请求查询投资者持仓
    virtual int ReqQryInvestorPosition(CThostFtdcQryInvestorPositionField *pQryInvestorPosition, int nRequestID) = 0;
    ///请求查询资金账户
    virtual int ReqQryTradingAccount(CThostFtdcQryTradingAccountField *pQryTradingAccount, int nRequestID) = 0;
    ///请求查询投资者
    virtual int ReqQryInvestor(CThostFtdcQryInvestorField *pQryInvestor, int nRequestID) = 0;
    ///请求查询交易编码
    virtual int ReqQryTradingCode(CThostFtdcQryTradingCodeField *pQryTradingCode, int nRequestID) = 0;
    ///请求查询合约保证金率
    virtual int ReqQryInstrumentMarginRate(CThostFtdcQryInstrumentMarginRateField *pQryInstrumentMarginRate, int nRequestID) = 0;
    ///请求查询合约手续费率
    virtual int ReqQryInstrumentCommissionRate(CThostFtdcQryInstrumentCommissionRateField *pQryInstrumentCommissionRate, int nRequestID) = 0;
    ///请求查询用户会话
    virtual int ReqQryUserSession(CThostFtdcQryUserSessionField *pQryUserSession, int nRequestID) = 0;
    ///请求查询交易所
    virtual int ReqQryExchange(CThostFtdcQryExchangeField *pQryExchange, int nRequestID) = 0;
    ///请求查询产品
    virtual int ReqQryProduct(CThostFtdcQryProductField *pQryProduct, int nRequestID) = 0;
    ///请求查询合约
    virtual int ReqQryInstrument(CThostFtdcQryInstrumentField *pQryInstrument, int nRequestID) = 0;
    ///请求查询行情
    virtual int ReqQryDepthMarketData(CThostFtdcQryDepthMarketDataField *pQryDepthMarketData, int nRequestID) = 0;
    ///请求查询投资者结算结果
    virtual int ReqQrySettlementInfo(CThostFtdcQrySettlementInfoField *pQrySettlementInfo, int nRequestID) = 0;
    ///请求查询转帐银行
    virtual int ReqQryTransferBank(CThostFtdcQryTransferBankField *pQryTransferBank, int nRequestID) = 0;
    ///请求查询投资者持仓明细
    virtual int ReqQryInvestorPositionDetail(CThostFtdcQryInvestorPositionDetailField *pQryInvestorPositionDetail, int nRequestID) = 0;
    ///请求查询客户通知
    virtual int ReqQryNotice(CThostFtdcQryNoticeField *pQryNotice, int nRequestID) = 0;
    ///请求查询结算信息确认
    virtual int ReqQrySettlementInfoConfirm(CThostFtdcQrySettlementInfoConfirmField *pQrySettlementInfoConfirm, int nRequestID) = 0;
    ///请求查询投资者持仓明细
    virtual int ReqQryInvestorPositionCombineDetail(CThostFtdcQryInvestorPositionCombineDetailField *pQryInvestorPositionCombineDetail, int nRequestID) = 0;
    ///请求查询保证金监管系统经纪公司资金账户密钥
    virtual int ReqQryCFMMCTradingAccountKey(CThostFtdcQryCFMMCTradingAccountKeyField *pQryCFMMCTradingAccountKey, int nRequestID) = 0;
    ///请求查询仓单折抵信息
    virtual int ReqQryEWarrantOffset(CThostFtdcQryEWarrantOffsetField *pQryEWarrantOffset, int nRequestID) = 0;
    ///请求查询投资者品种/跨品种保证金
    virtual int ReqQryInvestorProductGroupMargin(CThostFtdcQryInvestorProductGroupMarginField *pQryInvestorProductGroupMargin, int nRequestID) = 0;
    ///请求查询交易所保证金率
    virtual int ReqQryExchangeMarginRate(CThostFtdcQryExchangeMarginRateField *pQryExchangeMarginRate, int nRequestID) = 0;
    ///请求查询交易所调整保证金率
    virtual int ReqQryExchangeMarginRateAdjust(CThostFtdcQryExchangeMarginRateAdjustField *pQryExchangeMarginRateAdjust, int nRequestID) = 0;
    ///请求查询汇率
    virtual int ReqQryExchangeRate(CThostFtdcQryExchangeRateField *pQryExchangeRate, int nRequestID) = 0;
    ///请求查询二级代理操作员银期权限
    virtual int ReqQrySecAgentACIDMap(CThostFtdcQrySecAgentACIDMapField *pQrySecAgentACIDMap, int nRequestID) = 0;
    ///请求查询产品报价汇率
    virtual int ReqQryProductExchRate(CThostFtdcQryProductExchRateField *pQryProductExchRate, int nRequestID) = 0;
    ///请求查询产品组
    virtual int ReqQryProductGroup(CThostFtdcQryProductGroupField *pQryProductGroup, int nRequestID) = 0;
    ///请求查询做市商合约手续费率
    virtual int ReqQryMMInstrumentCommissionRate(CThostFtdcQryMMInstrumentCommissionRateField *pQryMMInstrumentCommissionRate, int nRequestID) = 0;
    ///请求查询做市商期权合约手续费
    virtual int ReqQryMMOptionInstrCommRate(CThostFtdcQryMMOptionInstrCommRateField *pQryMMOptionInstrCommRate, int nRequestID) = 0;
    ///请求查询报单手续费
    virtual int ReqQryInstrumentOrderCommRate(CThostFtdcQryInstrumentOrderCommRateField *pQryInstrumentOrderCommRate, int nRequestID) = 0;
    ///请求查询资金账户
    virtual int ReqQrySecAgentTradingAccount(CThostFtdcQryTradingAccountField *pQryTradingAccount, int nRequestID) = 0;
    ///请求查询二级代理商资金校验模式
    virtual int ReqQrySecAgentCheckMode(CThostFtdcQrySecAgentCheckModeField *pQrySecAgentCheckMode, int nRequestID) = 0;
    ///请求查询二级代理商信息
    virtual int ReqQrySecAgentTradeInfo(CThostFtdcQrySecAgentTradeInfoField *pQrySecAgentTradeInfo, int nRequestID) = 0;
    ///请求查询期权交易成本
    virtual int ReqQryOptionInstrTradeCost(CThostFtdcQryOptionInstrTradeCostField *pQryOptionInstrTradeCost, int nRequestID) = 0;
    ///请求查询期权合约手续费
    virtual int ReqQryOptionInstrCommRate(CThostFtdcQryOptionInstrCommRateField *pQryOptionInstrCommRate, int nRequestID) = 0;
    ///请求查询执行宣告
    virtual int ReqQryExecOrder(CThostFtdcQryExecOrderField *pQryExecOrder, int nRequestID) = 0;
    ///请求查询询价
    virtual int ReqQryForQuote(CThostFtdcQryForQuoteField *pQryForQuote, int nRequestID) = 0;
    ///请求查询报价
    virtual int ReqQryQuote(CThostFtdcQryQuoteField *pQryQuote, int nRequestID) = 0;
    ///请求查询期权自对冲
    virtual int ReqQryOptionSelfClose(CThostFtdcQryOptionSelfCloseField *pQryOptionSelfClose, int nRequestID) = 0;
    ///请求查询投资单元
    virtual int ReqQryInvestUnit(CThostFtdcQryInvestUnitField *pQryInvestUnit, int nRequestID) = 0;
    ///请求查询组合合约安全系数
    virtual int ReqQryCombInstrumentGuard(CThostFtdcQryCombInstrumentGuardField *pQryCombInstrumentGuard, int nRequestID) = 0;
    ///请求查询申请组合
    virtual int ReqQryCombAction(CThostFtdcQryCombActionField *pQryCombAction, int nRequestID) = 0;
    ///请求查询转帐流水
    virtual int ReqQryTransferSerial(CThostFtdcQryTransferSerialField *pQryTransferSerial, int nRequestID) = 0;
    ///请求查询银期签约关系
    virtual int ReqQryAccountregister(CThostFtdcQryAccountregisterField *pQryAccountregister, int nRequestID) = 0;
    ///请求查询签约银行
    virtual int ReqQryContractBank(CThostFtdcQryContractBankField *pQryContractBank, int nRequestID) = 0;
    ///请求查询预埋单
    virtual int ReqQryParkedOrder(CThostFtdcQryParkedOrderField *pQryParkedOrder, int nRequestID) = 0;
    ///请求查询预埋撤单
    virtual int ReqQryParkedOrderAction(CThostFtdcQryParkedOrderActionField *pQryParkedOrderAction, int nRequestID) = 0;
    ///请求查询交易通知
    virtual int ReqQryTradingNotice(CThostFtdcQryTradingNoticeField *pQryTradingNotice, int nRequestID) = 0;
    ///请求查询经纪公司交易参数
    virtual int ReqQryBrokerTradingParams(CThostFtdcQryBrokerTradingParamsField *pQryBrokerTradingParams, int nRequestID) = 0;
    ///请求查询经纪公司交易算法
    virtual int ReqQryBrokerTradingAlgos(CThostFtdcQryBrokerTradingAlgosField *pQryBrokerTradingAlgos, int nRequestID) = 0;
    ///请求查询监控中心用户令牌
    virtual int ReqQueryCFMMCTradingAccountToken(CThostFtdcQueryCFMMCTradingAccountTokenField *pQueryCFMMCTradingAccountToken, int nRequestID) = 0;
    ///期货发起银行资金转期货请求
    virtual int ReqFromBankToFutureByFuture(CThostFtdcReqTransferField *pReqTransfer, int nRequestID) = 0;
    ///期货发起期货资金转银行请求
    virtual int ReqFromFutureToBankByFuture(CThostFtdcReqTransferField *pReqTransfer, int nRequestID) = 0;
    ///期货发起查询银行余额请求
    virtual int ReqQueryBankAccountMoneyByFuture(CThostFtdcReqQueryAccountField *pReqQueryAccount, int nRequestID) = 0;
    ///请求查询分类合约
    virtual int ReqQryClassifiedInstrument(CThostFtdcQryClassifiedInstrumentField *pQryClassifiedInstrument, int nRequestID) = 0;
    ///请求组合优惠比例
    virtual int ReqQryCombPromotionParam(CThostFtdcQryCombPromotionParamField *pQryCombPromotionParam, int nRequestID) = 0;
    ///SPBM期货合约参数查询
    virtual int ReqQrySPBMFutureParameter(CThostFtdcQrySPBMFutureParameterField *pQrySPBMFutureParameter, int nRequestID) = 0;
    ///SPBM期权合约参数查询
    virtual int ReqQrySPBMOptionParameter(CThostFtdcQrySPBMOptionParameterField *pQrySPBMOptionParameter, int nRequestID) = 0;
    ///SPBM品种内对锁仓折扣参数查询
    virtual int ReqQrySPBMIntraParameter(CThostFtdcQrySPBMIntraParameterField *pQrySPBMIntraParameter, int nRequestID) = 0;
    ///SPBM跨品种抵扣参数查询
    virtual int ReqQrySPBMInterParameter(CThostFtdcQrySPBMInterParameterField *pQrySPBMInterParameter, int nRequestID) = 0;
    ///SPBM组合保证金套餐查询
    virtual int ReqQrySPBMPortfDefinition(CThostFtdcQrySPBMPortfDefinitionField *pQrySPBMPortfDefinition, int nRequestID) = 0;
    ///投资者SPBM套餐选择查询
    virtual int ReqQrySPBMInvestorPortfDef(CThostFtdcQrySPBMInvestorPortfDefField *pQrySPBMInvestorPortfDef, int nRequestID) = 0;
    ///投资者新型组合保证金系数查询
    virtual int ReqQryInvestorPortfMarginRatio(CThostFtdcQryInvestorPortfMarginRatioField *pQryInvestorPortfMarginRatio, int nRequestID) = 0;
    ///投资者产品SPBM明细查询
    virtual int ReqQryInvestorProdSPBMDetail(CThostFtdcQryInvestorProdSPBMDetailField *pQryInvestorProdSPBMDetail, int nRequestID) = 0;
    ///投资者商品组SPMM记录查询
    virtual int ReqQryInvestorCommoditySPMMMargin(CThostFtdcQryInvestorCommoditySPMMMarginField *pQryInvestorCommoditySPMMMargin, int nRequestID) = 0;
    ///投资者商品群SPMM记录查询
    virtual int ReqQryInvestorCommodityGroupSPMMMargin(CThostFtdcQryInvestorCommodityGroupSPMMMarginField *pQryInvestorCommodityGroupSPMMMargin, int nRequestID) = 0;
    ///SPMM合约参数查询
    virtual int ReqQrySPMMInstParam(CThostFtdcQrySPMMInstParamField *pQrySPMMInstParam, int nRequestID) = 0;
    ///SPMM产品参数查询
    virtual int ReqQrySPMMProductParam(CThostFtdcQrySPMMProductParamField *pQrySPMMProductParam, int nRequestID) = 0;
    ///SPBM附加跨品种抵扣参数查询
    virtual int ReqQrySPBMAddOnInterParameter(CThostFtdcQrySPBMAddOnInterParameterField *pQrySPBMAddOnInterParameter, int nRequestID) = 0;
    ///RCAMS产品组合信息查询
    virtual int ReqQryRCAMSCombProductInfo(CThostFtdcQryRCAMSCombProductInfoField *pQryRCAMSCombProductInfo, int nRequestID) = 0;
    ///RCAMS同合约风险对冲参数查询
    virtual int ReqQryRCAMSInstrParameter(CThostFtdcQryRCAMSInstrParameterField *pQryRCAMSInstrParameter, int nRequestID) = 0;
    ///RCAMS品种内风险对冲参数查询
    virtual int ReqQryRCAMSIntraParameter(CThostFtdcQryRCAMSIntraParameterField *pQryRCAMSIntraParameter, int nRequestID) = 0;
    ///RCAMS跨品种风险折抵参数查询
    virtual int ReqQryRCAMSInterParameter(CThostFtdcQryRCAMSInterParameterField *pQryRCAMSInterParameter, int nRequestID) = 0;
    ///RCAMS空头期权风险调整参数查询
    virtual int ReqQryRCAMSShortOptAdjustParam(CThostFtdcQryRCAMSShortOptAdjustParamField *pQryRCAMSShortOptAdjustParam, int nRequestID) = 0;
    ///RCAMS策略组合持仓查询
    virtual int ReqQryRCAMSInvestorCombPosition(CThostFtdcQryRCAMSInvestorCombPositionField *pQryRCAMSInvestorCombPosition, int nRequestID) = 0;
    ///投资者品种RCAMS保证金查询
    virtual int ReqQryInvestorProdRCAMSMargin(CThostFtdcQryInvestorProdRCAMSMarginField *pQryInvestorProdRCAMSMargin, int nRequestID) = 0;
    ///RULE合约保证金参数查询
    virtual int ReqQryRULEInstrParameter(CThostFtdcQryRULEInstrParameterField *pQryRULEInstrParameter, int nRequestID) = 0;
    ///RULE品种内对锁仓折扣参数查询
    virtual int ReqQryRULEIntraParameter(CThostFtdcQryRULEIntraParameterField *pQryRULEIntraParameter, int nRequestID) = 0;
    ///RULE跨品种抵扣参数查询
    virtual int ReqQryRULEInterParameter(CThostFtdcQryRULEInterParameterField *pQryRULEInterParameter, int nRequestID) = 0;
    ///投资者产品RULE保证金查询
    virtual int ReqQryInvestorProdRULEMargin(CThostFtdcQryInvestorProdRULEMarginField *pQryInvestorProdRULEMargin, int nRequestID) = 0;
    ///投资者投资者新组保设置查询
    virtual int ReqQryInvestorPortfSetting(CThostFtdcQryInvestorPortfSettingField *pQryInvestorPortfSetting, int nRequestID) = 0;
    ///投资者申报费阶梯收取记录查询
    virtual int ReqQryInvestorInfoCommRec(CThostFtdcQryInvestorInfoCommRecField *pQryInvestorInfoCommRec, int nRequestID) = 0;
    ///组合腿信息查询
    virtual int ReqQryCombLeg(CThostFtdcQryCombLegField *pQryCombLeg, int nRequestID) = 0;
    ///对冲设置请求
    virtual int ReqOffsetSetting(CThostFtdcInputOffsetSettingField *pInputOffsetSetting, int nRequestID) = 0;
    ///对冲设置撤销请求
    virtual int ReqCancelOffsetSetting(CThostFtdcInputOffsetSettingField *pInputOffsetSetting, int nRequestID) = 0;
    ///投资者对冲设置查询
    virtual int ReqQryOffsetSetting(CThostFtdcQryOffsetSettingField *pQryOffsetSetting, int nRequestID) = 0;
protected:
    ~CThostFtdcTraderApi(){};
};

```

<a id="f5d93d48-fb48-4d90-a16f-50261a962d0d"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 示例代码
<a id="panel2"></a>

```
class CTraderHandler : public CThostFtdcTraderSpi
{
private:
  CThostFtdcTraderApi *m_ptraderapi;
public:
  void connect()
  {
      //创建API实例，按照以下顺序初始化
      m_ptraderapi = CThostFtdcTraderApi::CreateFtdcTraderApi("flow/"); //必须提前创建好flow目录
      m_ptraderapi->RegisterSpi(this);
      m_ptraderapi->SubscribePublicTopic(THOST_TERT_QUICK);
      m_ptraderapi->SubscribePrivateTopic(THOST_TERT_QUICK); //设置私有流订阅模式
      m_ptraderapi->RegisterFront("tcp://127.0.0.1:41205");
      m_ptraderapi->Init();
      //输出API版本信息
      printf("%s\n", m_ptraderapi->GetApiVersion());
  }
}

```

请参阅：

    [CreateFtdcTraderApi](pages/061-JYJK-CTHOSTFTDCTRADERAPI-CREATEFTDCTRADERAPI.html.md)

    [GetApiVersion](pages/062-JYJK-CTHOSTFTDCTRADERAPI-GETAPIVERSION.html.md)

    [GetTradingDay](pages/063-JYJK-CTHOSTFTDCTRADERAPI-GETTRADINGDAY.html.md)

    [GetFrontInfo](pages/064-JYJK-CTHOSTFTDCTRADERAPI-GETFRONTINFO.html.md)

    [Init](pages/065-JYJK-CTHOSTFTDCTRADERAPI-INIT.html.md)

    [Join](pages/066-JYJK-CTHOSTFTDCTRADERAPI-JOIN.html.md)

    [RegisterFensUserInfo](pages/067-JYJK-CTHOSTFTDCTRADERAPI-REGISTERFENSUSERINFO.html.md)

    [RegisterFront](pages/068-JYJK-CTHOSTFTDCTRADERAPI-REGISTERFRONT.html.md)

    [RegisterNameServer](pages/069-JYJK-CTHOSTFTDCTRADERAPI-REGISTERNAMESERVER.html.md)

    [RegisterSpi](pages/070-JYJK-CTHOSTFTDCTRADERAPI-REGISTERSPI.html.md)

    [RegisterUserSystemInfo](pages/071-JYJK-CTHOSTFTDCTRADERAPI-REGISTERUSERSYSTEMINFO.html.md)

    [Release](pages/072-JYJK-CTHOSTFTDCTRADERAPI-RELEASE.html.md)

    [ReqAuthenticate](pages/073-JYJK-CTHOSTFTDCTRADERAPI-REQAUTHENTICATE.html.md)

    [ReqBatchOrderAction](pages/074-JYJK-CTHOSTFTDCTRADERAPI-REQBATCHORDERACTION.html.md)

    [ReqCombActionInsert](pages/075-JYJK-CTHOSTFTDCTRADERAPI-REQCOMBACTIONINSERT.html.md)

    [ReqExecOrderAction](pages/076-JYJK-CTHOSTFTDCTRADERAPI-REQEXECORDERACTION.html.md)

    [ReqExecOrderInsert](pages/077-JYJK-CTHOSTFTDCTRADERAPI-REQEXECORDERINSERT.html.md)

    [ReqForQuoteInsert](pages/078-JYJK-CTHOSTFTDCTRADERAPI-REQFORQUOTEINSERT.html.md)

    [ReqFromBankToFutureByFuture](pages/079-JYJK-CTHOSTFTDCTRADERAPI-REQFROMBANKTOFUTUREBYFUTURE.html.md)

    [ReqFromFutureToBankByFuture](pages/080-JYJK-CTHOSTFTDCTRADERAPI-REQFROMFUTURETOBANKBYFUTURE.html.md)

    [ReqGenUserCaptcha](pages/081-JYJK-CTHOSTFTDCTRADERAPI-REQGENUSERCAPTCHA.html.md)

    [ReqGenUserText](pages/082-JYJK-CTHOSTFTDCTRADERAPI-REQGENUSERTEXT.html.md)

    [ReqOptionSelfCloseAction](pages/083-JYJK-CTHOSTFTDCTRADERAPI-REQOPTIONSELFCLOSEACTION.html.md)

    [ReqOptionSelfCloseInsert](pages/084-JYJK-CTHOSTFTDCTRADERAPI-REQOPTIONSELFCLOSEINSERT.html.md)

    [ReqOrderAction](pages/085-JYJK-CTHOSTFTDCTRADERAPI-REQORDERACTION.html.md)

    [ReqOrderInsert](pages/086-JYJK-CTHOSTFTDCTRADERAPI-REQORDERINSERT.html.md)

    [ReqParkedOrderAction](pages/087-JYJK-CTHOSTFTDCTRADERAPI-REQPARKEDORDERACTION.html.md)

    [ReqParkedOrderInsert](pages/088-JYJK-CTHOSTFTDCTRADERAPI-REQPARKEDORDERINSERT.html.md)

    [ReqQryAccountregister](pages/089-JYJK-CTHOSTFTDCTRADERAPI-REQQRYACCOUNTREGISTER.html.md)

    [ReqQryBrokerTradingAlgos](pages/090-JYJK-CTHOSTFTDCTRADERAPI-REQQRYBROKERTRADINGALGOS.html.md)

    [ReqQryBrokerTradingParams](pages/091-JYJK-CTHOSTFTDCTRADERAPI-REQQRYBROKERTRADINGPARAMS.html.md)

    [ReqQryCFMMCTradingAccountKey](pages/092-JYJK-CTHOSTFTDCTRADERAPI-REQQRYCFMMCTRADINGACCOUNTKEY.html.md)

    [ReqQryCombAction](pages/093-JYJK-CTHOSTFTDCTRADERAPI-REQQRYCOMBACTION.html.md)

    [ReqQryCombInstrumentGuard](pages/094-JYJK-CTHOSTFTDCTRADERAPI-REQQRYCOMBINSTRUMENTGUARD.html.md)

    [ReqQryContractBank](pages/095-JYJK-CTHOSTFTDCTRADERAPI-REQQRYCONTRACTBANK.html.md)

    [ReqQryDepthMarketData](pages/096-JYJK-CTHOSTFTDCTRADERAPI-REQQRYDEPTHMARKETDATA.html.md)

    [ReqQryEWarrantOffset](pages/097-JYJK-CTHOSTFTDCTRADERAPI-REQQRYEWARRANTOFFSET.html.md)

    [ReqQryExchange](pages/098-JYJK-CTHOSTFTDCTRADERAPI-REQQRYEXCHANGE.html.md)

    [ReqQryExchangeMarginRate](pages/099-JYJK-CTHOSTFTDCTRADERAPI-REQQRYEXCHANGEMARGINRATE.html.md)

    [ReqQryExchangeMarginRateAdjust](pages/100-JYJK-CTHOSTFTDCTRADERAPI-REQQRYEXCHANGEMARGINRATEADJUST.html.md)

    [ReqQryExchangeRate](pages/101-JYJK-CTHOSTFTDCTRADERAPI-REQQRYEXCHANGERATE.html.md)

    [ReqQryExecOrder](pages/102-JYJK-CTHOSTFTDCTRADERAPI-REQQRYEXECORDER.html.md)

    [ReqQryForQuote](pages/103-JYJK-CTHOSTFTDCTRADERAPI-REQQRYFORQUOTE.html.md)

    [ReqQryInstrument](pages/104-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINSTRUMENT.html.md)

    [ReqQryInstrumentCommissionRate](pages/105-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINSTRUMENTCOMMISSIONRATE.html.md)

    [ReqQryInstrumentMarginRate](pages/106-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINSTRUMENTMARGINRATE.html.md)

    [ReqQryInstrumentOrderCommRate](pages/107-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINSTRUMENTORDERCOMMRATE.html.md)

    [ReqQryInvestor](pages/108-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTOR.html.md)

    [ReqQryInvestorPosition](pages/109-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPOSITION.html.md)

    [ReqQryInvestorPositionCombineDetail](pages/110-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPOSITIONCOMBINEDETAIL.html.md)

    [ReqQryInvestorPositionDetail](pages/111-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPOSITIONDETAIL.html.md)

    [ReqQryInvestorProductGroupMargin](pages/112-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPRODUCTGROUPMARGIN.html.md)

    [ReqQryInvestUnit](pages/113-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTUNIT.html.md)

    [ReqQryMMInstrumentCommissionRate](pages/114-JYJK-CTHOSTFTDCTRADERAPI-REQQRYMMINSTRUMENTCOMMISSIONRATE.html.md)

    [ReqQryMMOptionInstrCommRate](pages/115-JYJK-CTHOSTFTDCTRADERAPI-REQQRYMMOPTIONINSTRCOMMRATE.html.md)

    [ReqQryNotice](pages/116-JYJK-CTHOSTFTDCTRADERAPI-REQQRYNOTICE.html.md)

    [ReqQryOptionInstrCommRate](pages/117-JYJK-CTHOSTFTDCTRADERAPI-REQQRYOPTIONINSTRCOMMRATE.html.md)

    [ReqQryOptionInstrTradeCost](pages/118-JYJK-CTHOSTFTDCTRADERAPI-REQQRYOPTIONINSTRTRADECOST.html.md)

    [ReqQryOptionSelfClose](pages/119-JYJK-CTHOSTFTDCTRADERAPI-REQQRYOPTIONSELFCLOSE.html.md)

    [ReqQryOrder](pages/120-JYJK-CTHOSTFTDCTRADERAPI-REQQRYORDER.html.md)

    [ReqQryParkedOrder](pages/121-JYJK-CTHOSTFTDCTRADERAPI-REQQRYPARKEDORDER.html.md)

    [ReqQryParkedOrderAction](pages/122-JYJK-CTHOSTFTDCTRADERAPI-REQQRYPARKEDORDERACTION.html.md)

    [ReqQryProduct](pages/123-JYJK-CTHOSTFTDCTRADERAPI-REQQRYPRODUCT.html.md)

    [ReqQryProductExchRate](pages/124-JYJK-CTHOSTFTDCTRADERAPI-REQQRYPRODUCTEXCHRATE.html.md)

    [ReqQryProductGroup](pages/125-JYJK-CTHOSTFTDCTRADERAPI-REQQRYPRODUCTGROUP.html.md)

    [ReqQryQuote](pages/126-JYJK-CTHOSTFTDCTRADERAPI-REQQRYQUOTE.html.md)

    [ReqQrySecAgentACIDMap](pages/127-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSECAGENTACIDMAP.html.md)

    [ReqQrySecAgentCheckMode](pages/128-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSECAGENTCHECKMODE.html.md)

    [ReqQrySecAgentTradeInfo](pages/129-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSECAGENTTRADEINFO.html.md)

    [ReqQrySecAgentTradingAccount](pages/130-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSECAGENTTRADINGACCOUNT.html.md)

    [ReqQrySettlementInfo](pages/131-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSETTLEMENTINFO.html.md)

    [ReqQrySettlementInfoConfirm](pages/132-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSETTLEMENTINFOCONFIRM.html.md)

    [ReqQryTrade](pages/133-JYJK-CTHOSTFTDCTRADERAPI-REQQRYTRADE.html.md)

    [ReqQryTradingAccount](pages/134-JYJK-CTHOSTFTDCTRADERAPI-REQQRYTRADINGACCOUNT.html.md)

    [ReqQryTradingCode](pages/135-JYJK-CTHOSTFTDCTRADERAPI-REQQRYTRADINGCODE.html.md)

    [ReqQryTradingNotice](pages/136-JYJK-CTHOSTFTDCTRADERAPI-REQQRYTRADINGNOTICE.html.md)

    [ReqQryTransferBank](pages/137-JYJK-CTHOSTFTDCTRADERAPI-REQQRYTRANSFERBANK.html.md)

    [ReqQryTransferSerial](pages/138-JYJK-CTHOSTFTDCTRADERAPI-REQQRYTRANSFERSERIAL.html.md)

    [ReqQueryBankAccountMoneyByFuture](pages/139-JYJK-CTHOSTFTDCTRADERAPI-REQQUERYBANKACCOUNTMONEYBYFUTURE.html.md)

    [ReqQueryCFMMCTradingAccountToken](pages/140-JYJK-CTHOSTFTDCTRADERAPI-REQQUERYCFMMCTRADINGACCOUNTTOKEN.html.md)

    [ReqQryMaxOrderVolume](pages/141-JYJK-CTHOSTFTDCTRADERAPI-REQQUERYMAXORDERVOLUME.html.md)

    [ReqQuoteAction](pages/142-JYJK-CTHOSTFTDCTRADERAPI-REQQUOTEACTION.html.md)

    [ReqQuoteInsert](pages/143-JYJK-CTHOSTFTDCTRADERAPI-REQQUOTEINSERT.html.md)

    [ReqRemoveParkedOrder](pages/144-JYJK-CTHOSTFTDCTRADERAPI-REQREMOVEPARKEDORDER.html.md)

    [ReqRemoveParkedOrderAction](pages/145-JYJK-CTHOSTFTDCTRADERAPI-REQREMOVEPARKEDORDERACTION.html.md)

    [ReqSettlementInfoConfirm](pages/146-JYJK-CTHOSTFTDCTRADERAPI-REQSETTLEMENTINFOCONFIRM.html.md)

    [ReqTradingAccountPasswordUpdate](pages/147-JYJK-CTHOSTFTDCTRADERAPI-REQTRADINGACCOUNTPASSWORDUPDATE.html.md)

    [ReqUserAuthMethod](pages/148-JYJK-CTHOSTFTDCTRADERAPI-REQUSERAUTHMETHOD.html.md)

    [ReqUserLogin](pages/149-JYJK-CTHOSTFTDCTRADERAPI-REQUSERLOGIN.html.md)

    [ReqUserLoginWithCaptcha](pages/150-JYJK-CTHOSTFTDCTRADERAPI-REQUSERLOGINWITHCAPTCHA.html.md)

    [ReqUserLoginWithOTP](pages/151-JYJK-CTHOSTFTDCTRADERAPI-REQUSERLOGINWITHOTP.html.md)

    [ReqUserLoginWithText](pages/152-JYJK-CTHOSTFTDCTRADERAPI-REQUSERLOGINWITHTEXT.html.md)

    [ReqUserLogout](pages/153-JYJK-CTHOSTFTDCTRADERAPI-REQUSERLOGOUT.html.md)

    [ReqUserPasswordUpdate](pages/154-JYJK-CTHOSTFTDCTRADERAPI-REQUSERPASSWORDUPDATE.html.md)

    [SubmitUserSystemInfo](pages/155-JYJK-CTHOSTFTDCTRADERAPI-SUBMITUSERSYSTEMINFO.html.md)

    [SubscribePrivateTopic](pages/156-JYJK-CTHOSTFTDCTRADERAPI-SUBSCRIBEPRIVATETOPIC.html.md)

    [SubscribePublicTopic](pages/157-JYJK-CTHOSTFTDCTRADERAPI-SUBSCRIBEPUBLICTOPIC.html.md)

    [ReqQryClassifiedInstrument](pages/158-JYJK-CTHOSTFTDCTRADERAPI-REQQRYCLASSIFIEDINSTRUMENT.html.md)

    [ReqQryCombPromotionParam](pages/159-JYJK-CTHOSTFTDCTRADERAPI-REQQRYCOMBPROMOTIONPARAM.html.md)

    [ReqQryRiskSettleInvstPosition](pages/160-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRISKSETTLEINVSTPOSITION.html.md)

    [ReqQryRiskSettleProductStatus](pages/161-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRISKSETTLEPRODUCTSTATUS.html.md)

    [ReqQryTraderOffer](pages/162-JYJK-CTHOSTFTDCTRADERAPI-REQQRYTRADEROFFER.html.md)

    [ReqQrySPBMFutureParameter](pages/163-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPBMFUTUREPARAMETER.html.md)

    [ReqQrySPBMOptionParameter](pages/164-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPBMOPTIONPARAMETER.html.md)

    [ReqQrySPBMIntraParameter](pages/165-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPBMINTRAPARAMETER.html.md)

    [ReqQrySPBMInterParameter](pages/166-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPBMINTERPARAMETER.html.md)

    [ReqQrySPBMPortfDefinition](pages/167-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPBMPORTFDEFINITION.html.md)

    [ReqQrySPBMInvestorPortfDef](pages/168-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPBMINVESTORPORTFDEF.html.md)

    [ReqQryInvestorPortfMarginRatio](pages/169-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPORTFMARGINRATIO.html.md)

    [ReqQryInvestorProdSPBMDetail](pages/170-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPRODSPBMDETAIL.html.md)

    [ReqQryInvestorCommoditySPMMMargin](pages/171-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORCOMMODITYSPMMMARGIN.html.md)

    [ReqQryInvestorCommodityGroupSPMMMargin](pages/172-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORCOMMODITYGROUPSPMMMARGIN.html.md)

    [ReqQrySPMMInstParam](pages/173-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPMMINSTPARAM.html.md)

    [ReqQrySPMMProductParam](pages/174-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPMMPRODUCTPARAM.html.md)

    [ReqQrySPBMAddOnInterParameter](pages/175-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPBMADDONINTERPARAMETER.html.md)

    [ReqQryRCAMSCombProductInfo](pages/176-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSCOMBPRODUCTINFO.html.md)

    [ReqQryRCAMSInstrParameter](pages/177-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSINSTRPARAMETER.html.md)

    [ReqQryRCAMSIntraParameter](pages/178-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSINTRAPARAMETER.html.md)

    [ReqQryRCAMSInterParameter](pages/179-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSINTERPARAMETER.html.md)

    [ReqQryRCAMSShortOptAdjustParam](pages/180-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSSHORTOPTADJUSTPARAM.html.md)

    [ReqQryRCAMSInvestorCombPosition](pages/181-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRCAMSINVESTORCOMBPOSITION.html.md)

    [ReqQryInvestorProdRCAMSMargin](pages/182-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPRODRCAMSMARGIN.html.md)

    [ReqQryRULEInstrParameter](pages/183-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRULEINSTRPARAMETER.html.md)

    [ReqQryRULEIntraParameter](pages/184-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRULEINTRAPARAMETER.html.md)

    [ReqQryRULEInterParameter](pages/185-JYJK-CTHOSTFTDCTRADERAPI-REQQRYRULEINTERPARAMETER.html.md)

    [ReqQryInvestorProdRULEMargin](pages/186-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPRODRULEMARGIN.html.md)

    [ReqQryInvestorPortfSetting](pages/187-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORPORTFSETTING.html.md)

    [ReqQryInvestorInfoCommRec](pages/188-JYJK-CTHOSTFTDCTRADERAPI-REQQRYINVESTORINFOCOMMREC.html.md)

    [ReqQryCombLeg](pages/189-JYJK-CTHOSTFTDCTRADERAPI-REQQRYCOMBLEG.html.md)

    [ReqOffsetSetting](pages/190-JYJK-CTHOSTFTDCTRADERAPI-REQOFFSETSETTING.html.md)

    [ReqCancelOffsetSetting](pages/191-JYJK-CTHOSTFTDCTRADERAPI-REQCANCELOFFSETSETTING.html.md)

    [ReqQryOffsetSetting](pages/192-JYJK-CTHOSTFTDCTRADERAPI-REQQRYOFFSETSETTING.html.md)

    [RegisterWechatUserSystemInfo](pages/193-JYJK-CTHOSTFTDCTRADERAPI-REGISTERWECHATUSERSYSTEMINFO.html.md)

    [SubmitWechatUserSystemInfo](pages/194-JYJK-CTHOSTFTDCTRADERAPI-SUBMITWECHATUSERSYSTEMINFO.html.md)

    [ReqQryUserSession](pages/195-JYJK-CTHOSTFTDCTRADERAPI-REQQRYUSERSESSION.html.md)

    [ReqGenSMSCode](pages/196-JYJK-CTHOSTFTDCTRADERAPI-REQGENSMSCODE.html.md)

    [ReqSpdApply](pages/197-JYJK-CTHOSTFTDCTRADERAPI-REQSPDAPPLY.html.md)

    [ReqSpdApplyAction](pages/198-JYJK-CTHOSTFTDCTRADERAPI-REQSPDAPPLYACTION.html.md)

    [ReqQrySpdApply](pages/199-JYJK-CTHOSTFTDCTRADERAPI-REQQRYSPDAPPLY.html.md)

    [ReqHedgeCfm](pages/200-JYJK-CTHOSTFTDCTRADERAPI-REQHEDGECFM.html.md)

    [ReqHedgeCfmAction](pages/201-JYJK-CTHOSTFTDCTRADERAPI-REQHEDGECFMACTION.html.md)

    [ReqQryHedgeCfm](pages/202-JYJK-CTHOSTFTDCTRADERAPI-REQQRYHEDGECFM.html.md)

<a id="author"></a>

<a id="theme_switcher"></a>
