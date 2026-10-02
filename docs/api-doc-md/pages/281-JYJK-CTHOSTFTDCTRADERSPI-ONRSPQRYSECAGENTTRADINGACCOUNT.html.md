# OnRspQrySecAgentTradingAccount

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQrySecAgentTradingAccount<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询资金账户响应
<a id="57bf5f30-9488-42de-b4b1-277e8b8a42d6"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQrySecAgentTradingAccount(CThostFtdcTradingAccountField *pTradingAccount, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="248a891c-c080-4d6d-bca5-dc2b05322835"></a><a id="title2"></a>

<a id="header_span2"></a>◇ 2. 参数
<a id="panel2"></a>

pTradingAccount：资金账户

```
struct CThostFtdcTradingAccountField
{
    ///经纪公司代码
    TThostFtdcBrokerIDType BrokerID;
    ///投资者帐号
    TThostFtdcAccountIDType AccountID;
    ///上次质押金额
    TThostFtdcMoneyType PreMortgage;
    ///上次信用额度
    TThostFtdcMoneyType PreCredit;
    ///上次存款额
    TThostFtdcMoneyType PreDeposit;
    ///上次结算准备金
    TThostFtdcMoneyType PreBalance;
    ///上次占用的保证金
    TThostFtdcMoneyType PreMargin;
    ///利息基数
    TThostFtdcMoneyType InterestBase;
    ///利息收入
    TThostFtdcMoneyType Interest;
    ///入金金额
    TThostFtdcMoneyType Deposit;
    ///出金金额
    TThostFtdcMoneyType Withdraw;
    ///冻结的保证金
    TThostFtdcMoneyType FrozenMargin;
    ///冻结的资金
    TThostFtdcMoneyType FrozenCash;
    ///冻结的手续费
    TThostFtdcMoneyType FrozenCommission;
    ///当前保证金总额
    TThostFtdcMoneyType CurrMargin;
    ///资金差额
    TThostFtdcMoneyType CashIn;
    ///手续费
    TThostFtdcMoneyType Commission;
    ///平仓盈亏
    TThostFtdcMoneyType CloseProfit;
    ///持仓盈亏
    TThostFtdcMoneyType PositionProfit;
    ///期货结算准备金
    TThostFtdcMoneyType Balance;
    ///可用资金
    TThostFtdcMoneyType Available;
    ///可取资金
    TThostFtdcMoneyType WithdrawQuota;
    ///基本准备金
    TThostFtdcMoneyType Reserve;
    ///交易日
    TThostFtdcDateType TradingDay;
    ///结算编号
    TThostFtdcSettlementIDType SettlementID;
    ///信用额度
    TThostFtdcMoneyType Credit;
    ///质押金额
    TThostFtdcMoneyType Mortgage;
    ///交易所保证金
    TThostFtdcMoneyType ExchangeMargin;
    ///投资者交割保证金
    TThostFtdcMoneyType DeliveryMargin;
    ///交易所交割保证金
    TThostFtdcMoneyType ExchangeDeliveryMargin;
    ///保底期货结算准备金
    TThostFtdcMoneyType ReserveBalance;
    ///币种代码
    TThostFtdcCurrencyIDType CurrencyID;
    ///上次货币质入金额
    TThostFtdcMoneyType PreFundMortgageIn;
    ///上次货币质出金额
    TThostFtdcMoneyType PreFundMortgageOut;
    ///货币质入金额
    TThostFtdcMoneyType FundMortgageIn;
    ///货币质出金额
    TThostFtdcMoneyType FundMortgageOut;
    ///货币质押余额
    TThostFtdcMoneyType FundMortgageAvailable;
    ///可质押货币金额--已废弃
    TThostFtdcMoneyType MortgageableFund;
    ///特殊产品占用保证金--已废弃
    TThostFtdcMoneyType SpecProductMargin;
    ///特殊产品冻结保证金--已废弃
    TThostFtdcMoneyType SpecProductFrozenMargin;
    ///特殊产品手续费--已废弃
    TThostFtdcMoneyType SpecProductCommission;
    ///特殊产品冻结手续费--已废弃
    TThostFtdcMoneyType SpecProductFrozenCommission;
    ///特殊产品持仓盈亏--已废弃
    TThostFtdcMoneyType SpecProductPositionProfit;
    ///特殊产品平仓盈亏--已废弃
    TThostFtdcMoneyType SpecProductCloseProfit;
    ///根据持仓盈亏算法计算的特殊产品持仓盈亏--已废弃
    TThostFtdcMoneyType SpecProductPositionProfitByAlg;
    ///特殊产品交易所保证金--已废弃
    TThostFtdcMoneyType SpecProductExchangeMargin;
    ///业务类型
    TThostFtdcBizTypeType BizType;
    ///延时换汇冻结金额
    TThostFtdcMoneyType FrozenSwap;
    ///剩余换汇额度
    TThostFtdcMoneyType RemainSwap;
    ///期权市值
    TThostFtdcMoneyType OptionValue;
};

```

OptionValue：期权市值=（多头期权手数-空头期权手数）*期权合约乘数*期权最新价。终端厂商可通过期权市值计算市值权益：市值权益=期权市值+权益。

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

<a id="32af95f1-bd78-42d4-876b-a52d4f7fa2ad"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="7391b7b7-ecfe-45bf-911c-e1402496aa90"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

无

<a id="author"></a>

<a id="theme_switcher"></a>
