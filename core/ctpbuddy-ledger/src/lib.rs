//! Account ledger: funds, positions, margin and commission (DESIGN.md §8.6).
//!
//! Accounting model (M1 simplification, documented):
//! - `balance`        realized equity (2,000,000 CNY per auto-opened account);
//! - `position_profit` unrealized PnL, recomputed on every mark-to-market;
//! - `available = balance + position_profit - used_margin - frozen_margin
//!   - frozen_commission` (mirrors `CThostFtdcTradingAccountField`);
//! - opening an order freezes estimated margin + commission; the freeze is
//!   released pro-rata on fills and replaced by actuals at the fill price;
//! - closing realizes PnL into `balance` and releases margin;
//! - closing orders additionally reserve position volume (today/yd split) at
//!   insert time so two concurrent close orders cannot over-close;
//! - ByAmount margin and a flat fee rate via `Catalog`; per-exchange rule
//!   tables (close-today rates, 平今/平昨 order preference) are a TODO.
//!
//! The ledger is a single-writer state machine driven by the world loop; it
//! never performs IO.

use std::collections::HashMap;

use ctpbuddy_wire::generated::{cstr, set_cstr, CThostFtdcInvestorPositionField, CThostFtdcTradingAccountField};

use ctpbuddy_matching::{Catalog, Direction, Fill, OffsetFlag};

pub const INITIAL_FUNDS: f64 = 2_000_000.0;

pub const ERR_FUNDS: i32 = 50; // 可用资金不足
pub const ERR_POSITION: i32 = 30; // 持仓不足
pub const ERR_NO_CLOSE_TODAY_LEDGER: i32 = 31; // 可平今仓不足

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
    pub pre_balance: f64,
    pub deposit: f64,
    pub withdraw: f64,
    /// Realized equity (excludes unrealized position profit).
    pub balance: f64,
    pub position_profit: f64,
    pub close_profit: f64,
    pub commission: f64,
    pub used_margin: f64,
    pub frozen_margin: f64,
    pub frozen_commission: f64,
    pub currency_id: String,
}

impl Account {
    pub fn new(broker_id: &str, investor_id: &str, initial: f64) -> Self {
        Account {
            broker_id: broker_id.to_string(),
            investor_id: investor_id.to_string(),
            pre_balance: initial,
            deposit: 0.0,
            withdraw: 0.0,
            balance: initial,
            position_profit: 0.0,
            close_profit: 0.0,
            commission: 0.0,
            used_margin: 0.0,
            frozen_margin: 0.0,
            frozen_commission: 0.0,
            currency_id: "CNY".to_string(),
        }
    }

    /// Dynamic equity: realized balance + unrealized position profit.
    pub fn dynamic_equity(&self) -> f64 {
        self.balance + self.position_profit
    }

    /// `CThostFtdcTradingAccountField::Available` semantics.
    pub fn available(&self) -> f64 {
        self.dynamic_equity() - self.used_margin - self.frozen_margin - self.frozen_commission
    }

    /// Risk ratio `CurMargin / Balance` (0 when equity is non-positive).
    pub fn risk(&self) -> f64 {
        let eq = self.dynamic_equity();
        if eq > 0.0 {
            self.used_margin / eq
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
        f.PreBalance = self.pre_balance;
        f.PreMortgage = 0.0;
        f.PreCredit = 0.0;
        f.PreDeposit = 0.0;
        f.PreMargin = 0.0;
        f.Deposit = self.deposit;
        f.Withdraw = self.withdraw;
        f.Balance = self.dynamic_equity();
        f.Available = self.available();
        f.CurrMargin = self.used_margin;
        f.FrozenMargin = self.frozen_margin;
        f.FrozenCommission = self.frozen_commission;
        f.Commission = self.commission;
        f.CloseProfit = self.close_profit;
        f.PositionProfit = self.position_profit;
        f.CashIn = 0.0;
        f.WithdrawQuota = self.available();
        f.Reserve = 0.0;
        f.Credit = 0.0;
        f.Mortgage = 0.0;
        f.SettlementID = 1;
        f
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PositionSide {
    Long,
    Short,
}

impl PositionSide {
    pub fn of(direction: Direction) -> Self {
        match direction {
            Direction::Buy => PositionSide::Long,
            Direction::Sell => PositionSide::Short,
        }
    }

    /// The side a close order reduces: a sell closes a long, a buy closes a short.
    pub fn opposite(self) -> Self {
        match self {
            PositionSide::Long => PositionSide::Short,
            PositionSide::Short => PositionSide::Long,
        }
    }

    pub fn posi_direction(&self) -> u8 {
        match self {
            PositionSide::Long => b'2',
            PositionSide::Short => b'3',
        }
    }
}

#[derive(Clone, Debug)]
pub struct Position {
    pub instrument_id: String,
    pub side: PositionSide,
    /// 今仓 (opened today).
    pub today_position: i32,
    /// 昨仓 (carried from yesterday).
    pub yd_position: i32,
    /// Open turnover basis (sum of `price * volume * multiple`); average cost
    /// is `open_amount / (open_volume * multiple)`.
    pub open_amount: f64,
    pub open_volume: i32,
    pub position_cost: f64,
    pub open_cost: f64,
    pub margin: f64,
    /// Volume reserved by outstanding close orders (today / yd split).
    pub frozen_today: i32,
    pub frozen_yd: i32,
    pub commission: f64,
    pub close_profit: f64,
    pub position_profit: f64,
    pub pre_settlement_price: f64,
    pub settlement_price: f64,
}

impl Position {
    pub fn new(instrument_id: &str, side: PositionSide) -> Self {
        Position {
            instrument_id: instrument_id.to_string(),
            side,
            today_position: 0,
            yd_position: 0,
            open_amount: 0.0,
            open_volume: 0,
            position_cost: 0.0,
            open_cost: 0.0,
            margin: 0.0,
            frozen_today: 0,
            frozen_yd: 0,
            commission: 0.0,
            close_profit: 0.0,
            position_profit: 0.0,
            pre_settlement_price: 0.0,
            settlement_price: 0.0,
        }
    }

    pub fn volume(&self) -> i32 {
        self.today_position + self.yd_position
    }

    pub fn avg_cost(&self, volume_multiple: i32) -> f64 {
        let denom = self.open_volume as f64 * volume_multiple as f64;
        if denom > 0.0 {
            self.open_amount / denom
        } else {
            0.0
        }
    }

    /// `CThostFtdcInvestorPositionField` mirror for query responses.
    #[allow(clippy::too_many_arguments)]
    pub fn to_field(
        &self,
        broker_id: &str,
        investor_id: &str,
        exchange_id: &str,
        trading_day: &str,
        mult: i32,
    ) -> CThostFtdcInvestorPositionField {
        let mut f = CThostFtdcInvestorPositionField::zeroed();
        set_cstr(&mut f.BrokerID, broker_id);
        set_cstr(&mut f.InvestorID, investor_id);
        set_cstr(&mut f.ExchangeID, exchange_id);
        set_cstr(&mut f.InstrumentID, &self.instrument_id);
        set_cstr(&mut f.TradingDay, trading_day);
        f.PosiDirection = self.side.posi_direction();
        f.HedgeFlag = b'1';
        f.PositionDate = b'2'; // M1 never crosses a settlement boundary
        f.YdPosition = self.yd_position;
        f.Position = self.volume();
        f.TodayPosition = self.today_position;
        let frozen = self.frozen_today + self.frozen_yd;
        if self.side == PositionSide::Long {
            f.LongFrozen = frozen;
        } else {
            f.ShortFrozen = frozen;
        }
        f.OpenVolume = self.open_volume;
        f.OpenAmount = self.open_amount;
        f.OpenCost = self.open_cost;
        f.PositionCost = self.position_cost;
        f.UseMargin = self.margin;
        f.Commission = self.commission;
        f.CloseProfit = self.close_profit;
        f.PositionProfit = self.position_profit;
        f.PreSettlementPrice = self.pre_settlement_price;
        f.SettlementPrice = self.settlement_price;
        f.CloseProfitByDate = self.close_profit;
        f.CloseProfitByTrade = self.close_profit;
        let cost = self.avg_cost(mult);
        f.PositionCostOffset = cost;
        f.SettlementID = 1;
        f
    }
}

/// Frozen funds estimate attached to one order, released pro-rata on fills and
/// released in full on cancel.
#[derive(Clone, Debug)]
struct FrozenEst {
    key: AccountKey,
    margin: f64,
    commission: f64,
}

/// Position volume reserved by an outstanding close order (today/yd split),
/// released on fill or cancel.
#[derive(Clone, Debug)]
struct FrozenPos {
    key: AccountKey,
    instrument_id: String,
    side: PositionSide,
    today: i32,
    yd: i32,
}

pub struct Ledger {
    accounts: HashMap<AccountKey, Account>,
    positions: HashMap<(AccountKey, String, PositionSide), Position>,
    /// order_key -> remaining frozen funds estimate.
    frozen: HashMap<String, FrozenEst>,
    /// order_key -> position volume reservation.
    frozen_pos: HashMap<String, FrozenPos>,
    initial_funds: f64,
}

impl Ledger {
    pub fn new(initial_funds: f64) -> Self {
        Ledger {
            accounts: HashMap::new(),
            positions: HashMap::new(),
            frozen: HashMap::new(),
            frozen_pos: HashMap::new(),
            initial_funds,
        }
    }

    /// Auto-open an account on first login (SimNow-style convenience): every
    /// investor starts with `initial_funds`.
    pub fn ensure_account(&mut self, broker_id: &str, investor_id: &str) -> &mut Account {
        let initial = self.initial_funds;
        self.accounts
            .entry(AccountKey::new(broker_id, investor_id))
            .or_insert_with(|| Account::new(broker_id, investor_id, initial))
    }

    pub fn account(&self, broker_id: &str, investor_id: &str) -> Option<&Account> {
        self.accounts.get(&AccountKey::new(broker_id, investor_id))
    }

    pub fn account_mut(&mut self, broker_id: &str, investor_id: &str) -> Option<&mut Account> {
        self.accounts.get_mut(&AccountKey::new(broker_id, investor_id))
    }

    pub fn accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }

    /// Reset an account to its initial state (admin reset): wipes positions and
    /// any outstanding freezes belonging to the account.
    pub fn reset_account(&mut self, broker_id: &str, investor_id: &str) {
        let key = AccountKey::new(broker_id, investor_id);
        let fresh = Account::new(broker_id, investor_id, self.initial_funds);
        self.accounts.insert(key.clone(), fresh);
        self.positions.retain(|(k, _, _), _| *k != key);
        self.frozen.retain(|_, est| est.key != key);
        self.frozen_pos.retain(|_, est| est.key != key);
    }

    pub fn position(
        &self,
        broker_id: &str,
        investor_id: &str,
        instrument_id: &str,
        side: PositionSide,
    ) -> Option<&Position> {
        let key = AccountKey::new(broker_id, investor_id);
        self.positions.get(&(key, instrument_id.to_string(), side))
    }

    pub fn positions_of(&self, broker_id: &str, investor_id: &str) -> Vec<&Position> {
        let key = AccountKey::new(broker_id, investor_id);
        self.positions
            .iter()
            .filter(|((k, _, _), _)| *k == key)
            .map(|(_, p)| p)
            .collect()
    }

    /// Freeze estimated margin + commission for a new order. Fails with
    /// [`ERR_FUNDS`] when availability is insufficient.
    pub fn freeze(
        &mut self,
        order_key: &str,
        broker_id: &str,
        investor_id: &str,
        est_margin: f64,
        est_commission: f64,
    ) -> Result<(), i32> {
        let a = self.ensure_account(broker_id, investor_id);
        if a.available() + 1e-6 < est_margin + est_commission {
            return Err(ERR_FUNDS);
        }
        a.frozen_margin += est_margin;
        a.frozen_commission += est_commission;
        self.frozen.insert(
            order_key.to_string(),
            FrozenEst {
                key: AccountKey::new(broker_id, investor_id),
                margin: est_margin,
                commission: est_commission,
            },
        );
        Ok(())
    }

    /// Reserve closeable volume for a close order (today/yd split preference:
    /// `Close` takes today first, matching the fill accounting in `on_fill`).
    pub fn freeze_close_position(
        &mut self,
        order_key: &str,
        broker_id: &str,
        investor_id: &str,
        instrument_id: &str,
        side: PositionSide,
        offset: OffsetFlag,
        volume: i32,
    ) -> Result<(), i32> {
        let key = AccountKey::new(broker_id, investor_id);
        let pos = match self.positions.get_mut(&(key, instrument_id.to_string(), side)) {
            Some(p) => p,
            None => return Err(ERR_POSITION),
        };
        let avail_today = pos.today_position - pos.frozen_today;
        let avail_yd = pos.yd_position - pos.frozen_yd;
        let (from_today, from_yd) = match offset {
            OffsetFlag::CloseToday => {
                if avail_today < volume {
                    return Err(ERR_NO_CLOSE_TODAY_LEDGER);
                }
                (volume, 0)
            }
            OffsetFlag::CloseYesterday => {
                if avail_yd < volume {
                    return Err(ERR_POSITION);
                }
                (0, volume)
            }
            _ => {
                let t = volume.min(avail_today);
                let rest = volume - t;
                if rest > avail_yd {
                    return Err(ERR_POSITION);
                }
                (t, rest)
            }
        };
        pos.frozen_today += from_today;
        pos.frozen_yd += from_yd;
        self.frozen_pos.insert(
            order_key.to_string(),
            FrozenPos {
                key: AccountKey::new(broker_id, investor_id),
                instrument_id: instrument_id.to_string(),
                side,
                today: from_today,
                yd: from_yd,
            },
        );
        Ok(())
    }

    /// Release whatever remains frozen for an order (cancel path): funds
    /// estimate + position reservation.
    pub fn unfreeze_order(&mut self, order_key: &str) {
        if let Some(est) = self.frozen.remove(order_key) {
            if let Some(a) = self.accounts.get_mut(&est.key) {
                a.frozen_margin = (a.frozen_margin - est.margin).max(0.0);
                a.frozen_commission = (a.frozen_commission - est.commission).max(0.0);
            }
        }
        if let Some(res) = self.frozen_pos.remove(order_key) {
            if let Some(pos) = self
                .positions
                .get_mut(&(res.key, res.instrument_id.clone(), res.side))
            {
                pos.frozen_today = (pos.frozen_today - res.today).max(0);
                pos.frozen_yd = (pos.frozen_yd - res.yd).max(0);
            }
        }
    }

    /// Settle a fill: release the pro-rata freeze, pay commission, move margin
    /// and volume, realize close PnL.
    pub fn on_fill(&mut self, fill: &Fill, catalog: &Catalog) {
        let instr = catalog.get(&fill.instrument_id);
        let mult = instr.map(|i| i.volume_multiple).unwrap_or(1);
        let commission = instr
            .map(|i| i.commission(fill.price, fill.volume))
            .unwrap_or(fill.volume as f64);
        let margin_ratio = |d: Direction| {
            instr.map(|i| i.margin_ratio(d)).unwrap_or(0.10)
        };

        // ---- release pro-rata frozen estimate for this order ----
        let (rel_margin, rel_comm) = {
            let est = self.frozen.get_mut(&fill.order_key);
            match est {
                Some(est) => {
                    let total = fill.volume_total_original.max(1) as f64;
                    let frac = (fill.volume as f64 / total).clamp(0.0, 1.0);
                    let rm = est.margin * frac;
                    let rc = est.commission * frac;
                    est.margin -= rm;
                    est.commission -= rc;
                    (rm, rc)
                }
                None => (0.0, 0.0),
            }
        };

        let key = AccountKey::new(&cstr(&fill.broker_id), &cstr(&fill.investor_id));
        {
            let a = self
                .accounts
                .get_mut(&key)
                .expect("account must exist: login auto-opens it");
            a.frozen_margin = (a.frozen_margin - rel_margin).max(0.0);
            a.frozen_commission = (a.frozen_commission - rel_comm).max(0.0);
            a.balance -= commission;
            a.commission += commission;
        }

        // An open fill creates/increases a position on the order's side; a
        // close fill reduces the position on the opposite side (sell closes long).
        let side = if fill.offset == OffsetFlag::Open {
            PositionSide::of(fill.direction)
        } else {
            PositionSide::of(fill.direction).opposite()
        };
        let pos = self
            .positions
            .entry((key.clone(), fill.instrument_id.clone(), side))
            .or_insert_with(|| Position::new(&fill.instrument_id, side));

        pos.commission += commission;
        let turnover = fill.price * fill.volume as f64 * mult as f64;
        if fill.offset == OffsetFlag::Open {
            let margin_actual =
                fill.price * fill.volume as f64 * mult as f64 * margin_ratio(fill.direction);
            pos.today_position += fill.volume;
            pos.open_amount += turnover;
            pos.open_volume += fill.volume;
            pos.position_cost += turnover;
            pos.open_cost += commission;
            pos.margin += margin_actual;
            let a = self.accounts.get_mut(&key).expect("account exists");
            a.used_margin += margin_actual;
        } else {
            // ---- close: today first, then yd (consumes reservations) ----
            let mut to_close = fill.volume;
            let today_take = if matches!(fill.offset, OffsetFlag::CloseYesterday) {
                0
            } else {
                to_close.min(pos.today_position)
            };
            pos.today_position -= today_take;
            pos.frozen_today = (pos.frozen_today - today_take).max(0);
            to_close -= today_take;
            let yd_take = to_close.min(pos.yd_position);
            pos.yd_position -= yd_take;
            pos.frozen_yd = (pos.frozen_yd - yd_take).max(0);
            to_close -= yd_take;
            let closed = fill.volume - to_close;
            // `closed < fill.volume` is unreachable: freeze_close_position
            // reserved the volume at insert time (single-writer ledger).
            let _ = closed;

            let cost = pos.avg_cost(mult);
            let pnl = match side {
                PositionSide::Long => (fill.price - cost) * closed as f64 * mult as f64,
                PositionSide::Short => (cost - fill.price) * closed as f64 * mult as f64,
            };
            let closed_amount = cost * closed as f64 * mult as f64;
            pos.open_amount = (pos.open_amount - closed_amount).max(0.0);
            pos.open_volume = (pos.open_volume - closed).max(0);
            pos.position_cost = (pos.position_cost - closed_amount).max(0.0);
            pos.open_cost += commission;
            let margin_released =
                cost * closed as f64 * mult as f64 * margin_ratio(fill.direction);
            pos.margin = (pos.margin - margin_released).max(0.0);
            pos.close_profit += pnl;

            let a = self.accounts.get_mut(&key).expect("account exists");
            a.used_margin = (a.used_margin - margin_released).max(0.0);
            a.balance += pnl;
            a.close_profit += pnl;

            // drop flat positions from the query surface (CTP does not return them)
            if pos.volume() == 0 && pos.frozen_today == 0 && pos.frozen_yd == 0 {
                let k = (key.clone(), fill.instrument_id.clone(), side);
                self.positions.remove(&k);
            }
        }
    }

    /// Recompute unrealized PnL for every position from `prices`
    /// (`instrument -> last price`).
    pub fn mark_to_market(&mut self, catalog: &Catalog, prices: &HashMap<String, f64>) {
        for pos in self.positions.values_mut() {
            let mult = catalog
                .get(&pos.instrument_id)
                .map(|i| i.volume_multiple)
                .unwrap_or(1);
            let p = prices
                .get(&pos.instrument_id)
                .copied()
                .unwrap_or(pos.settlement_price);
            let cost = pos.avg_cost(mult);
            let vol = pos.volume();
            pos.position_profit = match pos.side {
                PositionSide::Long => (p - cost) * vol as f64 * mult as f64,
                PositionSide::Short => (cost - p) * vol as f64 * mult as f64,
            };
            pos.settlement_price = p;
        }
        let mut sums: HashMap<AccountKey, f64> = HashMap::new();
        for ((key, _, _), pos) in &self.positions {
            *sums.entry(key.clone()).or_insert(0.0) += pos.position_profit;
        }
        // accounts with positions take the recomputed sum; the rest reset to 0.
        for a in self.accounts.values_mut() {
            let key = AccountKey::new(&a.broker_id, &a.investor_id);
            a.position_profit = sums.get(&key).copied().unwrap_or(0.0);
        }
    }
}
