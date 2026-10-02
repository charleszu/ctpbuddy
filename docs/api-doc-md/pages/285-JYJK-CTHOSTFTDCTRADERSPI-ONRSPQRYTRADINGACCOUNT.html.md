# OnRspQryTradingAccount

<a id="__TOP_4E4ABC53-B143-46FF-93CF-F9381EAD8E14__"></a>

<a id="printArea"></a>

<a id="file_header"></a>

OnRspQryTradingAccount<a id="content"></a>

<a id="left_menu"></a>

  ** **

请求查询资金账户响应，当执行[ReqQryTradingAccount](pages/134-JYJK-CTHOSTFTDCTRADERAPI-REQQRYTRADINGACCOUNT.html.md)后，该方法被调用。
<a id="0e358b54-3463-46da-a629-28b5ab35c92a"></a><a id="title1"></a>

<a id="header_span1"></a>◇ 1. 函数原型
<a id="panel1"></a>

virtual void OnRspQryTradingAccount(CThostFtdcTradingAccountField *pTradingAccount, CThostFtdcRspInfoField *pRspInfo, int nRequestID, bool bIsLast) {};

<a id="983017d4-0d39-46c1-bb22-7889c629f475"></a><a id="title2"></a>

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

<a id="anchor-id-01"></a>

ReserveBalance：最低权益标准，对应柜台菜单“投资者最低权益设置”

Reserve：基础保证金，对应柜台菜单“投资者基础保证金设置”

PreCredit：上日资金冻结，为负值

Credit：当日资金冻结，为负值。交易系统初始化后会将PreCredit赋值给Credit，因此Credit包含了上日资金冻结和当日资金冻结。

CurrMargin：当前保证金总额。仅当期权权利金算法使用[max(昨结算价,最新价)]时，盘中此字段根据期权持仓行情实时计算，其他情况以及期货持仓均不随行情发生变动。

PreBalance：昨权益

Balance：权益

FrozenCash：冻结权利金

CashIn：权利金

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

<a id="61194f3b-9e9e-48eb-8537-b72a7d55049f"></a><a id="title3"></a>

<a id="header_span3"></a>◇ 3. 返回
<a id="panel3"></a>

当查询无记录时，指针返回为null

<a id="96d43ddd-dd9c-4aed-bf34-ad7fe3a668e3"></a><a id="title4"></a>

<a id="header_span4"></a>◇ 4. FAQ
<a id="panel4"></a>

<a id="region_header_1"></a>

期货品种的套保仓、投机仓、套利仓的单向大边是单独计算，还是一起计算的？对于能源中心INE目前的交易细则是不是也有单向大边的规则？<a id="region_panel_1"></a>

| 单向大边目前只有上期所和中金所有此项业务。对于上期所，同品种的买卖双边分别计算，取买卖双边中保证金金额较大的一边单向实时收取。
对于中金所，套利、套保和投机的交易编码不同，计算的时候首先是区分交易编码的，每个交易编码下同品种的买卖双边再分别计算取大边。
对于能源中心，sc目前是单向大边的规则。 |
|---|

<a id="region_tail_1"></a>

<a id="region_header_2"></a>

关于冻结手续费计算的问题：比如我有大商所的持仓 i1709 10手昨仓，这个时候我平仓2手挂单了，请问此时的平仓手续费率是采用平昨手续费还是平今手续费，还是这两个费率取较大的？<a id="region_panel_2"></a>

| 短线开平仓合约的平仓（不管平仓还是平今）在成交的时候是按平今手续费收取的，在冻结的时候是按平仓手续费收取。 |
|---|

<a id="region_tail_2"></a>

<a id="region_header_3"></a>

如何理解大商所短线开平仓，手续费的收取规则如何？<a id="region_panel_3"></a>

-

-

-

-

| 1.关于短线开平。目前只有大商所有短线开平。跟其他交易所相比，短线开平的业务体现在开仓手续费的收取上。当有短线开平业务的时候，结算时会判断开仓是否平仓，如果是则收取【短线开仓手续费】；但是交易时有所不同，由于交易无法在你开仓时预知是否会平仓，所以交易里并没有【短线开仓手续费】，并且开仓时收取的是【开仓手续费】。
2.持仓的计算仍然是先开先平；
3.所有交易所的手续费的计算都是按照【开仓后平仓收平今手续费，今仓平完后收平仓手续费】的规则计算。举例说明如下：
场景一：历史仓3手，平仓3手，收平仓手续费；
场景二：历史仓3手，开仓1手，平仓3手，平仓时收1手平今，2手平仓手续费。
4.根据第3点，目前中金所的平今手续费高（15倍于平仓），而平仓手续费低，盘中开仓后平仓的手续费就会高收，这部分在结算时恢复正常。 |
|---|

<a id="region_tail_3"></a>

<a id="author"></a>

<a id="theme_switcher"></a>
