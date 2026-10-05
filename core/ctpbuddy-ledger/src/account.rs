//! Account key and the cash-account record.

use ctpbuddy_wire::generated::{set_cstr, CThostFtdcTradingAccountField};

use crate::Money;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AccountKey {
    pub broker_id: String,
    pub investor_id: String,
}

impl AccountKey {
    pub fn new(broker_id: &str, investor_id: &str) -> Self {
        AccountKey {
            broker_id: broker_id.to_string(),
            investor_id: investor_id.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Account {
    pub broker_id: String,
    pub investor_id: String,
    pub pre_balance: Money,
    pub deposit: Money,
    pub withdraw: Money,
    /// Realized equity (excludes unrealized position profit).
    pub balance: Money,
    pub position_profit: Money,
    pub close_profit: Money,
    pub commission: Money,
    pub used_margin: Money,
    pub frozen_margin: Money,
    pub frozen_commission: Money,
    pub currency_id: String,
}

impl Account {
    pub fn new(broker_id: &str, investor_id: &str, initial: Money) -> Self {
        Account {
            broker_id: broker_id.to_string(),
            investor_id: investor_id.to_string(),
            pre_balance: initial,
            deposit: Money::ZERO,
            withdraw: Money::ZERO,
            balance: initial,
            position_profit: Money::ZERO,
            close_profit: Money::ZERO,
            commission: Money::ZERO,
            used_margin: Money::ZERO,
            frozen_margin: Money::ZERO,
            frozen_commission: Money::ZERO,
            currency_id: "CNY".to_string(),
        }
    }

    /// Dynamic equity: static equity plus today's mark-to-market effects.
    /// `balance` already contains realized close PnL and commission; deposits
    /// and withdrawals remain separate account-day fields.
    pub fn dynamic_equity(&self) -> Money {
        self.balance + self.deposit - self.withdraw + self.position_profit
    }

    /// `CThostFtdcTradingAccountField::Available` semantics.
    pub fn available(&self) -> Money {
        self.dynamic_equity() - self.used_margin - self.frozen_margin - self.frozen_commission
    }

    /// Risk ratio `CurMargin / Balance` (0 when equity is non-positive).
    pub fn risk(&self) -> f64 {
        let eq = self.dynamic_equity();
        if eq.is_positive() {
            self.used_margin.to_f64() / eq.to_f64()
        } else {
            0.0
        }
    }

    /// `CThostFtdcTradingAccountField` mirror for query responses.
    pub fn to_field(&self, trading_day: &str) -> CThostFtdcTradingAccountField {
        let mut f = CThostFtdcTradingAccountField::zeroed();
        set_cstr(&mut f.BrokerID, &self.broker_id);
        set_cstr(&mut f.AccountID, &self.investor_id);
        set_cstr(&mut f.CurrencyID, &self.currency_id);
        set_cstr(&mut f.TradingDay, trading_day);
        f.PreBalance = self.pre_balance.to_f64();
        f.PreMortgage = 0.0;
        f.PreCredit = 0.0;
        f.PreDeposit = 0.0;
        f.PreMargin = 0.0;
        f.Deposit = self.deposit.to_f64();
        f.Withdraw = self.withdraw.to_f64();
        f.Balance = self.dynamic_equity().to_f64();
        f.Available = self.available().to_f64();
        f.CurrMargin = self.used_margin.to_f64();
        f.FrozenMargin = self.frozen_margin.to_f64();
        f.FrozenCommission = self.frozen_commission.to_f64();
        f.Commission = self.commission.to_f64();
        f.CloseProfit = self.close_profit.to_f64();
        f.PositionProfit = self.position_profit.to_f64();
        f.CashIn = 0.0;
        f.WithdrawQuota = self.available().to_f64();
        f.Reserve = 0.0;
        f.Credit = 0.0;
        f.Mortgage = 0.0;
        f.SettlementID = 1;
        f
    }
}
