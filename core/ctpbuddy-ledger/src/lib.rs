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

use ctpbuddy_wire::generated::CThostFtdcInvestorProductGroupMarginField;

use ctpbuddy_wire::generated::{
    cstr, set_cstr, CThostFtdcInvestorPositionDetailField, CThostFtdcInvestorPositionField,
    CThostFtdcTradingAccountField,
};

use ctpbuddy_matching::{Catalog, CommissionKind, Direction, Fill, OffsetFlag};

pub const INITIAL_FUNDS: f64 = 2_000_000.0;

// Official CTP error.xml codes (docs/错误码全集.md): ids and prompts verbatim —
// downstream clients key off these numbers.
pub const ERR_FUNDS: i32 = 31; //          INSUFFICIENT_MONEY       CTP:资金不足
pub const ERR_POSITION: i32 = 30; //       OVER_CLOSE_POSITION      CTP:平仓量超过持仓量
pub const ERR_NO_CLOSE_TODAY_LEDGER: i32 = 50; // OVER_CLOSETODAY_POSITION CTP:平今仓位不足
pub const ERR_NO_CLOSE_YD_LEDGER: i32 = 51; // OVER_CLOSEYESTERDAY_POSITION CTP:平昨仓位不足

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

/// One open-lot record — the unit CTP actually settles.
///
/// notes/04 B2: a detail appears on every opening fill and is keyed by
/// `(OpenDate, TradeID)`. It is not decoration: under 逐日盯市 the **close PnL
/// of a lot is priced off its own basis**, which differs by age (E2):
///
/// - 昨仓 detail (`OpenDate != today`) → basis = **昨结算价**
/// - 今仓 detail (`OpenDate == today`) → basis = **开仓价**
///
/// Getting the order of consumption wrong therefore changes the PnL number a
/// client sees, which is exactly the failure mode notes/04 E2 warns about
/// ("如果顺序错了，那么算出来的平仓盈亏也就会和实际值不相符了").
///
/// Verified against real broker statements (`ctp_settlement/`, 山金期货
/// 盯市单): IH2501 买 1 手 opened 20250102, closed 20250103 at 2616.4 with
/// 昨结算 2607.2 → (2607.2 − 2616.4) × 300 = −2760.00, matching the
/// statement's 平仓盈亏 exactly. The open price 2626.2 would have given
/// −2940 — the detail's age, not its entry, sets the basis.
#[derive(Clone, Debug)]
pub struct PositionDetail {
    /// 开仓日期 — the trading day this lot was opened.
    pub open_date: String,
    /// 成交编号 — within a day, orders time in sequence; the tiebreaker that
    /// makes 先开先平 total.
    pub trade_id: String,
    pub open_price: f64,
    pub bootstrap: bool,
    /// Remaining lots on this detail.
    pub volume: i32,
    /// Margin charged to this detail when it was opened.
    pub margin: f64,
    /// Price basis used when the opening margin was booked. Queries reuse this
    /// settled basis; it is never refreshed from later market data.
    pub margin_price: f64,
    /// Lots this detail originally had. Margin release is pro-rata against
    /// this, not against the (shrinking) remainder, so closing the last lot
    /// releases exactly what was charged.
    pub open_volume: i32,
    /// 昨结算价 of the day this lot became 昨仓 (0 while it is 今仓).
    pub last_settlement_price: f64,
    /// Realized PnL attributed to this detail so far.
    pub close_profit: f64,
    pub close_profit_trade: f64,
    pub commission: f64,
    /// Lots of this detail already closed.
    pub close_volume: i32,
    /// Turnover of the closes attributed to this detail.
    pub close_amount: f64,
}

impl PositionDetail {
    pub fn new(open_date: &str, trade_id: &str, open_price: f64, volume: i32, margin: f64, margin_price: f64) -> Self {
        PositionDetail {
            open_date: open_date.to_string(),
            trade_id: trade_id.to_string(),
            open_price,
            bootstrap: false,
            volume,
            margin,
            margin_price,
            open_volume: volume,
            last_settlement_price: 0.0,
            close_profit: 0.0,
            close_profit_trade: 0.0,
            commission: 0.0,
            close_volume: 0,
            close_amount: 0.0,
        }
    }

    pub fn is_today(&self, trading_day: &str) -> bool {
        self.open_date == trading_day
    }

    /// 逐日盯市 basis for this detail (notes/04 E2). A lot opened today is
    /// marked against its entry; anything older against 昨结算.
    pub fn mark_basis(&self, trading_day: &str) -> f64 {
        if self.is_today(trading_day) {
            self.open_price
        } else {
            self.last_settlement_price
        }
    }

    /// 浮动盈亏 of this detail at `price`, per notes/04 E1.
    pub fn profit(&self, side: PositionSide, price: f64, multiple: i32) -> f64 {
        let sign = match side {
            PositionSide::Long => 1.0,
            PositionSide::Short => -1.0,
        };
        (price - self.open_price) * sign * self.volume as f64 * multiple as f64
    }

    /// `CThostFtdcInvestorPositionDetailField` mirror.
    #[allow(clippy::too_many_arguments)]
    pub fn to_field(
        &self,
        broker_id: &str,
        investor_id: &str,
        exchange_id: &str,
        trading_day: &str,
        side: PositionSide,
        instrument_id: &str,
        settlement_price: f64,
        last_price: f64,
        multiple: i32,
    ) -> CThostFtdcInvestorPositionDetailField {
        let mut f = CThostFtdcInvestorPositionDetailField::zeroed();
        set_cstr(&mut f.BrokerID, broker_id);
        set_cstr(&mut f.InvestorID, investor_id);
        set_cstr(&mut f.InstrumentID, instrument_id);
        set_cstr(&mut f.OpenDate, &self.open_date);
        set_cstr(&mut f.TradeID, &self.trade_id);
        set_cstr(&mut f.TradingDay, trading_day);
        f.ExchangeID.fill(0);
        set_cstr(&mut f.ExchangeID, exchange_id);
        f.HedgeFlag = b'1';
        f.Direction = match side {
            PositionSide::Long => b'0',
            PositionSide::Short => b'1',
        };
        f.TradeType = b'0';
        f.Volume = self.volume;
        f.OpenPrice = self.open_price;
        f.SettlementID = 1;
        f.CloseVolume = self.close_volume;
        f.CloseAmount = self.close_amount;
        f.Margin = self.margin * self.volume as f64 / self.open_volume.max(1) as f64;
        f.LastSettlementPrice = self.last_settlement_price;
        f.SettlementPrice = settlement_price;
        let basis = if self.open_date == trading_day { self.open_price } else { self.last_settlement_price };
        f.PositionProfitByDate = Position::detail_pnl(side, basis, last_price, self.volume, multiple);
        f.PositionProfitByTrade = Position::detail_pnl(side, self.open_price, last_price, self.volume, multiple);
        f.CloseProfitByDate = self.close_profit;
        f.CloseProfitByTrade = self.close_profit_trade;
        // 先开先平剩余数量: the lots of this detail still open, which for a
        // detail consumed from the front equals its own remainder.
        f.TimeFirstVolume = self.volume;
        f
    }
}

#[derive(Clone, Debug)]
pub struct Position {
    pub instrument_id: String,
    pub side: PositionSide,
    /// 今仓 (opened today).
    pub today_position: i32,
    /// 昨仓 (currently remaining carried lots).
    pub yd_position: i32,
    /// Static trading-day-start yesterday position (`YdPosition`).
    pub yd_initial: i32,
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
    /// Per-lot records backing this aggregate, ordered oldest-first so
    /// 先开先平 is a straight walk (notes/04 B2).
    pub details: Vec<PositionDetail>,
}

impl Position {
    pub fn new(instrument_id: &str, side: PositionSide) -> Self {
        Position {
            instrument_id: instrument_id.to_string(),
            side,
            today_position: 0,
            yd_position: 0,
            yd_initial: 0,
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
            details: Vec::new(),
        }
    }

    /// Record an opening fill as its own detail (notes/04 B2: a detail is
    /// created per opening fill, keyed by `(OpenDate, TradeID)`).
    pub fn add_detail(&mut self, open_date: &str, trade_id: &str, price: f64, volume: i32, margin: f64, margin_price: f64) {
        self.details
            .push(PositionDetail::new(open_date, trade_id, price, volume, margin, margin_price));
    }

    pub fn add_bootstrap_detail(
        &mut self,
        open_date: &str,
        trade_id: &str,
        price: f64,
        volume: i32,
        margin: f64,
        pre_settlement: f64,
        multiple: i32,
    ) {
        self.yd_position += volume;
        self.yd_initial += volume;
        self.open_cost += price * volume as f64 * multiple as f64;
        self.position_cost += pre_settlement * volume as f64 * multiple as f64;
        self.margin += margin;
        self.settlement_price = pre_settlement;
        let mut detail = PositionDetail::new(open_date, trade_id, price, volume, margin, pre_settlement);
        detail.bootstrap = true;
        detail.last_settlement_price = pre_settlement;
        self.details.push(detail);
        self.details.sort_by(|a, b| a.open_date.cmp(&b.open_date).then(a.trade_id.cmp(&b.trade_id)));
    }

    /// 先开先平: consume `volume` lots from the **oldest** details, returning
    /// the per-detail breakdown the caller needs to price PnL and commission.
    ///
    /// 先开先平 and 今/昨 are two different axes and must not be conflated:
    /// the walk order is by opening time, full stop. `today_only` /
    /// `yd_only` say *which age bucket* may be touched (上期所/能源中心 split
    /// the offset flags), never that the walk may skip ahead to the newest
    /// lot — that would realize PnL against the wrong basis and break the
    /// 先开先平 guarantee clients rely on to reproduce a desk's positions.
    pub fn take_details_filtered(
        &mut self,
        volume: i32,
        today_only: bool,
        yd_only: bool,
        trading_day: &str,
    ) -> Vec<(usize, i32)> {
        let mut taken = Vec::new();
        let mut left = volume;
        for idx in 0..self.details.len() {
            if left <= 0 {
                break;
            }
            let is_today = self.details[idx].is_today(trading_day);
            if (today_only && !is_today) || (yd_only && is_today) {
                continue;
            }
            let n = left.min(self.details[idx].volume);
            if n <= 0 {
                continue;
            }
            self.details[idx].volume -= n;
            self.details[idx].close_volume += n;
            taken.push((idx, n));
            left -= n;
        }
        taken
    }

    /// 逐日盯市 close PnL for one consumed detail (notes/04 E2).
    ///
    /// `basis` comes from [`PositionDetail::mark_basis`] — 昨结算 for 昨仓,
    /// 开仓价 for 今仓. Pricing every lot off the position's average cost
    /// (what this ledger did before details existed) is wrong whenever a
    /// position mixes lots of different ages, which is the normal case after
    /// the first partial close.
    pub fn detail_pnl(side: PositionSide, basis: f64, price: f64, n: i32, multiple: i32) -> f64 {
        let sign = match side {
            PositionSide::Long => 1.0,
            PositionSide::Short => -1.0,
        };
        (price - basis) * sign * n as f64 * multiple as f64
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

    /// Query projection for `CThostFtdcInvestorPositionField`.
    ///
    /// CTP exposes SHFE/INE today and yesterday buckets as separate rows. The
    /// ledger keeps one position aggregate, so this projection splits only the
    /// query row and allocates aggregate fields once by the live bucket volume.
    /// It never derives the official static `YdPosition` initial value from the
    /// current position: this minimal model has no settlement/bootstrap input,
    /// therefore that value remains zero until such an input exists.
    pub fn to_query_fields(
        &self,
        broker_id: &str,
        investor_id: &str,
        exchange_id: &str,
        trading_day: &str,
        mult: i32,
    ) -> Vec<CThostFtdcInvestorPositionField> {
        let base = self.to_field(broker_id, investor_id, exchange_id, trading_day, mult);
        if !matches!(exchange_id, "SHFE" | "INE") {
            return vec![base];
        }
        let yd = self.detail_bucket_field(base.clone(), trading_day, mult, false);
        let today = self.detail_bucket_field(base, trading_day, mult, true);
        if (self.yd_position > 0 || self.yd_initial > 0) && self.today_position > 0 {
            vec![yd, today]
        } else if self.yd_position > 0 {
            vec![yd]
        } else if self.today_position > 0 {
            vec![today]
        } else {
            // CTP may retain a zero Position row with static YdPosition until
            // settlement; preserve that row, but never expose zero-volume detail.
            vec![yd]
        }
    }

    fn detail_bucket_field(
        &self,
        mut field: CThostFtdcInvestorPositionField,
        trading_day: &str,
        mult: i32,
        today: bool,
    ) -> CThostFtdcInvestorPositionField {
        field.PositionDate = if today { b'1' } else { b'2' };
        field.Position = 0;
        field.TodayPosition = 0;
        field.YdPosition = if today { 0 } else { self.yd_initial };
        field.OpenVolume = 0;
        field.CloseVolume = 0;
        field.CloseAmount = 0.0;
        field.OpenAmount = 0.0;
        field.OpenCost = 0.0;
        field.PositionCost = 0.0;
        field.UseMargin = 0.0;
        field.ExchangeMargin = 0.0;
        field.Commission = 0.0;
        field.CloseProfit = 0.0;
        field.PositionProfit = 0.0;
        field.CloseProfitByDate = 0.0;
        field.CloseProfitByTrade = 0.0;
        field.PositionCostOffset = 0.0;
        field.LongFrozen = 0;
        field.ShortFrozen = 0;
        field.LongFrozenAmount = 0.0;
        field.ShortFrozenAmount = 0.0;
        field.FrozenMargin = 0.0;
        field.FrozenCash = 0.0;
        field.FrozenCommission = 0.0;
        field.CashIn = 0.0;
        let frozen = if today { self.frozen_today } else { self.frozen_yd };
        if self.side == PositionSide::Long {
            field.ShortFrozen = frozen;
        } else {
            field.LongFrozen = frozen;
        }
        for detail in &self.details {
            if detail.is_today(trading_day) != today {
                continue;
            }
            let volume = detail.volume;
            let basis = detail.mark_basis(trading_day);
            field.CloseProfit += detail.close_profit;
            field.CloseProfitByDate += detail.close_profit;
            field.CloseProfitByTrade += detail.close_profit_trade;
            field.Commission += detail.commission;
            field.CloseVolume += detail.close_volume;
            field.CloseAmount += detail.close_amount;
            if today && !detail.bootstrap {
                field.OpenVolume += detail.open_volume;
                field.OpenAmount += detail.open_price * detail.open_volume as f64 * mult as f64;
            }
            if volume > 0 {
                let share = volume as f64 / detail.open_volume.max(1) as f64;
                field.Position += volume;
                field.OpenCost += detail.open_price * volume as f64 * mult as f64;
                field.PositionCost += basis * volume as f64 * mult as f64;
                field.UseMargin += detail.margin * share;
                field.PositionProfit += Position::detail_pnl(self.side, basis, self.settlement_price, volume, mult);
                if today {
                    field.TodayPosition += volume;
                }
            }
        }
        field
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
        // `YdPosition` is the official static trading-day-start value. The
        // ledger has no settlement/bootstrap source for that value, so it must
        // not be fabricated from the current yesterday bucket.
        f.PositionDate = if !matches!(exchange_id, "SHFE" | "INE") || (self.today_position > 0 && self.yd_position == 0) {
            b'1'
        } else {
            b'2'
        };
        f.YdPosition = self.yd_initial;
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

/// Frozen funds estimate attached to one order. `margin` / `commission` keep
/// the **original** estimate; each fill releases `original * filled_fraction`
/// and the released part is tracked, so a fully-filled order (no terminal
/// '5', hence no `unfreeze_order`) still ends at exactly zero frozen.
#[derive(Clone, Debug)]
struct FrozenEst {
    key: AccountKey,
    instrument_id: String,
    side: PositionSide,
    margin: f64,
    commission: f64,
    released_margin: f64,
    released_commission: f64,
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

#[derive(Clone)]
pub struct Ledger {
    accounts: HashMap<AccountKey, Account>,
    positions: HashMap<(AccountKey, String, PositionSide), Position>,
    /// order_key -> remaining frozen funds estimate.
    frozen: HashMap<String, FrozenEst>,
    /// order_key -> position volume reservation.
    frozen_pos: HashMap<String, FrozenPos>,
    initial_funds: f64,
    group_activity: HashMap<(AccountKey, String, String), (f64, f64)>,
}

impl Ledger {
    pub fn new(initial_funds: f64) -> Self {
        Ledger {
            accounts: HashMap::new(),
            positions: HashMap::new(),
            frozen: HashMap::new(),
            frozen_pos: HashMap::new(),
            initial_funds,
            group_activity: HashMap::new(),
        }
    }

    /// Auto-open an account on first login (SimNow-style convenience): every
    /// investor starts with `initial_funds`.
    pub fn set_initial_funds(&mut self, initial_funds: f64) {
        self.initial_funds = initial_funds;
    }

    pub fn ensure_account(&mut self, broker_id: &str, investor_id: &str) -> &mut Account {
        let initial = self.initial_funds;
        self.accounts
            .entry(AccountKey::new(broker_id, investor_id))
            .or_insert_with(|| Account::new(broker_id, investor_id, initial))
    }

    /// Ensure an account exists with scenario-authored initial funds
    /// (DESIGN.md §7.4 `accounts`). Existing accounts keep their state; a
    /// non-positive `funds` falls back to the server default.
    pub fn ensure_account_with(&mut self, broker_id: &str, investor_id: &str, funds: f64) -> &mut Account {
        let initial = self.initial_funds;
        self.accounts
            .entry(AccountKey::new(broker_id, investor_id))
            .or_insert_with(|| {
                Account::new(broker_id, investor_id, if funds > 0.0 { funds } else { initial })
            })
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
        self.group_activity.retain(|(k, _, _), _| *k != key);
    }

    pub fn position_mut_or_create(
        &mut self,
        broker_id: &str,
        investor_id: &str,
        instrument_id: &str,
        side: PositionSide,
    ) -> &mut Position {
        let key = AccountKey::new(broker_id, investor_id);
        self.positions
            .entry((key, instrument_id.to_string(), side))
            .or_insert_with(|| Position::new(instrument_id, side))
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

    /// Positions in a **stable order** — by instrument, then by side.
    ///
    /// The query surfaces must be reproducible: the backing store is a
    /// `HashMap`, so iterating it directly hands clients a different row order
    /// on every run. That is not cosmetic — a client diffing two snapshots, or
    /// a journal hash covering a query stream, would see spurious diffs.
    pub fn positions_of_ordered(
        &self,
        broker_id: &str,
        investor_id: &str,
    ) -> Vec<(&Position, &PositionDetail)> {
        let key = AccountKey::new(broker_id, investor_id);
        let mut out: Vec<(&Position, &PositionDetail)> = self
            .positions
            .iter()
            .filter(|((k, _, _), _)| *k == key)
            .flat_map(|(_, p)| p.details.iter().filter(|d| d.volume > 0).map(move |d| (p, d)))
            .collect();
        out.sort_by(|a, b| {
            a.0.instrument_id
                .cmp(&b.0.instrument_id)
                .then(a.0.side.cmp(&b.0.side))
                // Within a position, 先开先平 order is the meaningful one: it
                // is the order a client will consume them in.
                .then(a.1.open_date.cmp(&b.1.open_date))
                .then(a.1.trade_id.cmp(&b.1.trade_id))
        });
        out
    }

    /// Freeze estimated margin + commission for a new order. Fails with
    /// [`ERR_FUNDS`] when availability is insufficient.
    pub fn freeze(
        &mut self,
        order_key: &str,
        broker_id: &str,
        investor_id: &str,
        instrument_id: &str,
        side: PositionSide,
        est_margin: f64,
        est_commission: f64,
        catalog: &Catalog,
    ) -> Result<(), i32> {
        self.ensure_account(broker_id, investor_id);
        self.frozen.insert(
            order_key.to_string(),
            FrozenEst {
                key: AccountKey::new(broker_id, investor_id),
                instrument_id: instrument_id.to_string(),
                side,
                margin: est_margin,
                commission: est_commission,
                released_margin: 0.0,
                released_commission: 0.0,
            },
        );
        self.refresh_margin(catalog);
        if self.account(broker_id, investor_id).unwrap().available() < -1e-6 {
            self.frozen.remove(order_key);
            self.refresh_margin(catalog);
            return Err(ERR_FUNDS);
        }
        Ok(())
    }

    /// 按账户、交易所、品种聚合；目前账本仅支持投机、空投资单元。
    /// 未启用优惠的合约独立求和，不能被同品种优惠合约抵消。
    pub fn product_group_margin(
        &self, broker: &str, investor: &str, catalog: &Catalog, day: &str,
    ) -> Vec<CThostFtdcInvestorProductGroupMarginField> {
        let key = AccountKey::new(broker, investor);
        let mut groups: std::collections::BTreeMap<(String, String),
            (CThostFtdcInvestorProductGroupMarginField, [[f64; 2]; 4], [[f64; 2]; 4])>
            = std::collections::BTreeMap::new();
        for ((k, id, side), pos) in &self.positions {
            if *k != key { continue; }
            let Some(i) = catalog.get(id) else { continue; };
            let product_group = i.product_id.clone();
            let (f, used, _) = groups.entry((i.exchange_id.clone(), product_group))
                .or_insert_with(|| (CThostFtdcInvestorProductGroupMarginField::zeroed(), [[0.0; 2]; 4], [[0.0; 2]; 4]));
            let s = if *side == PositionSide::Long { 0 } else { 1 };
            let bucket = if catalog.refdata().product_margin_algorithm(id) { 1 } else { 0 };
            used[bucket][s] += pos.margin;
            let ratio = if s == 0 { i.long_margin_ratio } else { i.short_margin_ratio };
            used[2 + bucket][s] += pos.details.iter().map(|d| {
                // ExchMargin is query-only: use the detail's booked price
                // baseline, never a later market quote.
                d.margin_price * d.volume as f64 * i.volume_multiple as f64 * ratio
            }).sum::<f64>();
            f.PositionProfit += pos.position_profit;
        }
        for est in self.frozen.values().filter(|e| e.key == key) {
            let Some(i) = catalog.get(&est.instrument_id) else { continue; };
            let product_group = i.product_id.clone();
            let (f, _, frozen) = groups.entry((i.exchange_id.clone(), product_group))
                .or_insert_with(|| (CThostFtdcInvestorProductGroupMarginField::zeroed(), [[0.0; 2]; 4], [[0.0; 2]; 4]));
            let s = if est.side == PositionSide::Long { 0 } else { 1 };
            let bucket = if catalog.refdata().product_margin_algorithm(&est.instrument_id) { 1 } else { 0 };
            frozen[bucket][s] += (est.margin - est.released_margin).max(0.0);
            f.FrozenCommission += (est.commission - est.released_commission).max(0.0);
        }
        for ((k, ex, prod), (commission, profit)) in &self.group_activity {
            if *k != key { continue; }
            let (f, _, _) = groups.entry((ex.clone(), prod.clone()))
                .or_insert_with(|| (CThostFtdcInvestorProductGroupMarginField::zeroed(), [[0.0; 2]; 4], [[0.0; 2]; 4]));
            f.Commission = *commission;
            f.CloseProfit = *profit;
        }
        groups.into_iter().map(|((ex, prod), (mut f, used, frozen))| {
            set_cstr(&mut f.BrokerID, broker);
            set_cstr(&mut f.InvestorID, investor);
            set_cstr(&mut f.ExchangeID, &ex);
            set_cstr(&mut f.ProductGroupID, &prod);
            set_cstr(&mut f.TradingDay, day);
            f.SettlementID = 1;
            f.HedgeFlag = b'1';
            f.LongUseMargin = used[0][0] + used[1][0];
            f.ShortUseMargin = used[0][1] + used[1][1];
            f.UseMargin = used[0][0] + used[0][1] + used[1][0].max(used[1][1]);
            f.LongFrozenMargin = frozen[0][0] + frozen[1][0];
            f.ShortFrozenMargin = frozen[0][1] + frozen[1][1];
            let current_max = used[1][0].max(used[1][1]);
            let projected_max = (used[1][0] + frozen[1][0]).max(used[1][1] + frozen[1][1]);
            f.FrozenMargin = frozen[0][0] + frozen[0][1] + (projected_max - current_max).max(0.0);
            // 交易所费率独立展示；没有跨品种映射或仓单折抵规则，不推导折抵金额。
            f.LongExchMargin = used[2][0] + used[3][0];
            f.ShortExchMargin = used[2][1] + used[3][1];
            f.ExchMargin = used[2][0] + used[2][1] + used[3][0].max(used[3][1]);
            f
        }).collect()
    }

    pub fn refresh(&mut self, catalog: &Catalog) {
        self.refresh_margin(catalog);
    }

    fn refresh_margin(&mut self, catalog: &Catalog) {
        let keys: Vec<_> = self.accounts.keys().cloned().collect();
        for key in keys {
            let rows = self.product_group_margin(&key.broker_id, &key.investor_id, catalog, "");
            let a = self.accounts.get_mut(&key).unwrap();
            a.used_margin = rows.iter().map(|f| f.UseMargin).sum();
            a.frozen_margin = rows.iter().map(|f| f.FrozenMargin).sum();
            a.frozen_commission = rows.iter().map(|f| f.FrozenCommission).sum();
        }
    }

    /// Reserve closeable volume for a close order.
    ///
    /// A plain `Close` reserves **昨仓 first**, i.e. 先开先平: notes/04 B3
    /// ("平仓一般是按先开先平来处理，即先平昨仓再平今仓") and the P1 worked
    /// example (2 昨 + 2 今, close 3 → 2 昨 + 1 今). Taking 今仓 first would
    /// price those three lots off a different basis and misstate the PnL.
    /// `CloseToday` / `CloseYesterday` are the client saying which one it
    /// means — that is the whole point of 上期所 splitting the flags — and are
    /// honoured literally.
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
                return Err(ERR_NO_CLOSE_YD_LEDGER);
            }
            (0, volume)
        }
            _ => {
                let y = volume.min(avail_yd);
                let rest = volume - y;
                if rest > avail_today {
                    return Err(ERR_POSITION);
                }
                (rest, y)
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

    /// Release whatever remains frozen for an order (cancel path): the
    /// unreleased part of the funds estimate + the position reservation.
    pub fn unfreeze_order(&mut self, order_key: &str, catalog: &Catalog) {
        if let Some(est) = self.frozen.remove(order_key) {
            let rem_margin = (est.margin - est.released_margin).max(0.0);
            let rem_commission = (est.commission - est.released_commission).max(0.0);
            if let Some(a) = self.accounts.get_mut(&est.key) {
                a.frozen_margin = (a.frozen_margin - rem_margin).max(0.0);
                a.frozen_commission = (a.frozen_commission - rem_commission).max(0.0);
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
        self.refresh_margin(catalog);
    }

    /// Settle a fill: release the pro-rata freeze, pay commission, move margin
    /// and volume, realize close PnL.
    ///
    /// Commission is priced **per position leg**, not per fill: a `Close` that
    /// eats 2 lots of yesterday and 1 of today pays 平昨 for two and 平今 for
    /// one (notes/04 B3/D1). notes/04 B3 is explicit that even 大商所 — which
    /// does not split positions into 今/昨 — still prices the two with
    /// different rates, so a single blended rate is visibly wrong.
    ///
    /// `pre_settlement` is the instrument's 昨结算价 **as the engine currently
    /// sees it**. It is a parameter rather than a lookup because the first
    /// fill of a session lands before the ledger has ever been marked to
    /// market, and `MarginPriceType == '1'` (the default) charges 今仓 against
    /// 昨结算 — reading the position's own copy there would margin the opening
    /// fill at zero.
    ///
    /// `trading_day` decides each lot's 逐日盯市 basis (notes/04 E2): a detail
    /// opened today is marked against its entry, anything older against 昨结算.
    /// Passing it in rather than storing a day on the ledger keeps a single
    /// source of truth — the world loop's virtual clock, which scenario replay
    /// rewinds.
    pub fn on_fill(
        &mut self,
        fill: &Fill,
        catalog: &Catalog,
        pre_settlement: f64,
        trading_day: &str,
    ) {
        let instr = catalog.get(&fill.instrument_id);
        let before_commission = self.account(&cstr(&fill.broker_id), &cstr(&fill.investor_id)).unwrap().commission;
        let before_profit = self.account(&cstr(&fill.broker_id), &cstr(&fill.investor_id)).unwrap().close_profit;
        let mult = instr.map(|i| i.volume_multiple).unwrap_or(1);

        // ---- release pro-rata frozen estimate for this order ----
        // The fraction is of the ORIGINAL estimate (not of what is left), so
        // a multi-tranche fill that ends up fully filled releases exactly
        // 100%: 2/3 + 1/3 of the original, not of the shrinking remainder.
        let (rel_margin, rel_comm) = {
            let est = self.frozen.get_mut(&fill.order_key);
            match est {
                Some(est) => {
                    let total = fill.volume_total_original.max(1) as f64;
                    let frac = (fill.volume as f64 / total).clamp(0.0, 1.0);
                    let rm = est.margin * frac;
                    let rc = est.commission * frac;
                    est.released_margin += rm;
                    est.released_commission += rc;
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

        let turnover = fill.price * fill.volume as f64 * mult as f64;
        if fill.offset == OffsetFlag::Open {
            // 昨结算价 comes from the tick stream, not from the position: on
            // the very first fill of a session the position has none yet, and
            // `MarginPriceType == '1'` prices 今仓 against it.
            if pos.pre_settlement_price <= 0.0 && pre_settlement > 0.0 {
                pos.pre_settlement_price = pre_settlement;
            }
            // 今仓保证金 follows `MarginPriceType`; the fill price is the
            // 开仓价 leg of that choice (notes/04 C2).
            let price = catalog.margin_price_for(true, pos.pre_settlement_price, fill.price);
            let margin_actual = catalog.margin(
                &fill.instrument_id,
                fill.direction,
                price,
                fill.volume,
            );
            let commission =
                catalog.commission(&fill.instrument_id, CommissionKind::Open, fill.price, fill.volume);
            pos.commission += commission;
            pos.today_position += fill.volume;
            pos.open_amount += turnover;
            pos.open_volume += fill.volume;
            pos.position_cost += turnover;
            pos.open_cost += commission;
            pos.margin += margin_actual;
            // notes/04 B2: an opening fill creates its own detail, keyed by
            // (OpenDate, TradeID). The margin travels with the lot so a later
            // close can release exactly what was charged, and so
            // `ReqQryInvestorPositionDetail` has rows to return.
            pos.add_detail(
                trading_day,
                &cstr(&fill.trade_id),
                fill.price,
                fill.volume,
                margin_actual,
                price.value(),
            );
            pos.details.last_mut().unwrap().commission = commission;
            let a = self.accounts.get_mut(&key).expect("account exists");
            a.balance -= commission;
            a.commission += commission;
            // 账户占用由共享聚合重算，不按成交保证金直接相加。
        } else {
            // ---- close: consume details oldest-first (先开先平) ----
            //
            // An explicit 平今/平昨 says *which age bucket* to touch; a plain
            // `Close` takes whatever comes first. Either way the walk order is
            // by opening time, so the lots and their PnL stay consistent.
            let want_today_only = matches!(fill.offset, OffsetFlag::CloseToday);
            let want_yd_only = matches!(fill.offset, OffsetFlag::CloseYesterday);

            // 逐日盯市 PnL, priced per detail (notes/04 E2). 昨仓 lots are
            // marked against 昨结算, 今仓 lots against their own entry — so
            // one fill can span two bases, and pricing every lot off the
            // aggregate average cost (what this ledger did before details
            // existed) is wrong by exactly the age mix.
            let mut pnl = 0.0;
            let mut closed_amount = 0.0;
            let mut margin_released = 0.0;
            let mut today_take = 0;
            let mut yd_take = 0;
            for (idx, n) in pos.take_details_filtered(fill.volume, want_today_only, want_yd_only, trading_day) {
                let d = &mut pos.details[idx];
                let is_today = d.is_today(trading_day);
                if is_today {
                    today_take += n;
                } else {
                    yd_take += n;
                }
                let basis = d.mark_basis(trading_day);
                let leg_pnl = Position::detail_pnl(side, basis, fill.price, n, mult);
                let leg_trade_pnl = Position::detail_pnl(side, d.open_price, fill.price, n, mult);
                pnl += leg_pnl;
                closed_amount += d.open_price * n as f64 * mult as f64;
                d.close_amount += fill.price * n as f64 * mult as f64;
                // Margin travels with the lot: release the pro-rata share of
                // what this detail was charged, not a recomputed figure.
                margin_released += d.margin * n as f64 / d.open_volume.max(1) as f64;
                d.close_profit += leg_pnl;
                d.close_profit_trade += leg_trade_pnl;
                let kind = if is_today { CommissionKind::CloseToday } else { CommissionKind::CloseYesterday };
                d.commission += catalog.commission(&fill.instrument_id, kind, fill.price, n);
            }
            let closed = today_take + yd_take;
            pos.today_position = (pos.today_position - today_take).max(0);
            pos.yd_position = (pos.yd_position - yd_take).max(0);
            pos.frozen_today = (pos.frozen_today - today_take).max(0);
            pos.frozen_yd = (pos.frozen_yd - yd_take).max(0);
            // `closed < fill.volume` is unreachable: freeze_close_position
            // reserved the volume at insert time (single-writer ledger).
            debug_assert_eq!(closed, fill.volume);

            pos.open_amount = (pos.open_amount - closed_amount).max(0.0);
            pos.open_volume = (pos.open_volume - closed).max(0);
            // 持仓成本 follows the **持仓价** basis (notes/04 B1), not the open
            // price: a 昨仓 lot is carried at 昨结算 until it is closed. Sum it
            // off the surviving details rather than subtracting, so a carried
            // lot keeps being valued at settlement instead of at its entry.
            pos.position_cost = pos
                .details
                .iter()
                .map(|d| d.mark_basis(trading_day) * d.volume as f64 * mult as f64)
                .sum();

            // Commission is charged per leg: the 平今 portion at today's rate,
            // the 平昨 portion at yesterday's. Summing two legs is the whole
            // point — a blended single rate is what notes/04 B3 warns about.
            let comm_today = catalog.commission(
                &fill.instrument_id,
                CommissionKind::CloseToday,
                fill.price,
                today_take,
            );
            let comm_yd = catalog.commission(
                &fill.instrument_id,
                CommissionKind::CloseYesterday,
                fill.price,
                yd_take,
            );
            let commission = comm_today + comm_yd;
            pos.commission += commission;
            pos.open_cost += commission;

            pos.margin = (pos.margin - margin_released).max(0.0);
            pos.close_profit += pnl;

            let a = self.accounts.get_mut(&key).expect("account exists");
            a.balance -= commission;
            a.commission += commission;
            // 平仓后大边可能切换，账户保证金交给共享聚合重算。
            a.balance += pnl;
            a.close_profit += pnl;

            // Exhausted details leave the query surface; CTP does not return
            // a 明细 with zero remaining volume.
            // 当天累计归属依赖已平明细；仅查询入口过滤零余量，不在盘中删除。
            // Keep a static bootstrap row after a full close. CTP can expose the
            // zero Position row until settlement; zero-volume details remain
            // hidden by the detail query projection.
            if pos.volume() == 0 && pos.frozen_today == 0 && pos.frozen_yd == 0 && pos.yd_initial == 0 {
                let k = (key.clone(), fill.instrument_id.clone(), side);
                self.positions.remove(&k);
            }
        }
        if let Some(i) = instr {
            let a = self.accounts.get(&key).unwrap();
            let activity = self.group_activity.entry((key, i.exchange_id.clone(), i.product_id.clone())).or_default();
            activity.0 += a.commission - before_commission;
            activity.1 += a.close_profit - before_profit;
        }
        self.refresh_margin(catalog);
    }

    /// Recompute unrealized PnL for every position from `prices`.
    ///
    /// Market data updates settlement/PnL only. Position margin is booked at
    /// fill time and remains unchanged until a fill releases its detail share.
    pub fn mark_to_market(
        &mut self,
        catalog: &Catalog,
        prices: &HashMap<String, f64>,
        pre_settlements: &HashMap<String, f64>,
        trading_day: &str,
    ) {
        for pos in self.positions.values_mut() {
            let mult = catalog
                .get(&pos.instrument_id)
                .map(|i| i.volume_multiple)
                .unwrap_or(1);
            let p = prices
                .get(&pos.instrument_id)
                .copied()
                .unwrap_or(pos.settlement_price);
            if let Some(ps) = pre_settlements.get(&pos.instrument_id) {
                if *ps > 0.0 && pos.yd_initial == 0 {
                    pos.pre_settlement_price = *ps;
                    for d in pos.details.iter_mut().filter(|d| !d.bootstrap) {
                        d.last_settlement_price = *ps;
                    }
                }
            }
            pos.position_profit = pos.details.iter().map(|d| {
                let basis = d.mark_basis(trading_day);
                Position::detail_pnl(pos.side, basis, p, d.volume, mult)
            }).sum();
            pos.settlement_price = p;
        }
        let mut sums: HashMap<AccountKey, f64> = HashMap::new();
        for ((key, _, _), pos) in &self.positions {
            *sums.entry(key.clone()).or_insert(0.0) += pos.position_profit;
        }
        for a in self.accounts.values_mut() {
            let key = AccountKey::new(&a.broker_id, &a.investor_id);
            a.position_profit = sums.get(&key).copied().unwrap_or(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctpbuddy_matching::{Instrument, MarginRate, TradingParams, to_fixed};

    fn fixture(enabled: bool) -> Catalog {
        // 明确虚构的测试合约和费率，不代表生产默认值。
        let mut catalog = Catalog::new();
        for (id, product) in [("TESTA01", "TESTA"), ("TESTA02", "TESTA"), ("TESTB01", "TESTB")] {
            let mut i = Instrument::new(id, "TEST");
            i.product_id = product.into();
            i.max_margin_side_algorithm = if enabled { b'1' } else { b'0' };
            catalog.insert(i);
            catalog.insert_margin_rate(MarginRate {
                instrument_id: id.into(), long_margin_ratio_by_volume: 100.0,
                short_margin_ratio_by_volume: 150.0, ..Default::default()
            });
        }
        let mut params = TradingParams::default();
        params.margin_price_type = b'4';
        catalog.set_trading_params(params);
        catalog
    }

    fn fill(id: &str, direction: Direction, offset: OffsetFlag, volume: i32) -> Fill {
        Fill { broker_id: to_fixed("TEST"), investor_id: to_fixed("alice"), user_id: to_fixed("alice"),
            instrument_id: id.into(), exchange_id: "TEST".into(), direction, offset, hedge_flag: b'1',
            price: 10.0, volume, volume_total_original: volume, order_sys_id: to_fixed("1"),
            order_ref: to_fixed("1"), trade_id: to_fixed("1"), order_key: "1".into() }
    }

    #[test]
    fn shfe_query_projection_splits_age_buckets_without_double_counting() {
        let mut p = Position::new("rb", PositionSide::Long);
        p.add_bootstrap_detail("20261002", "YD1", 10.0, 2, 40.0, 9.0, 1);
        p.details.push(PositionDetail::new("20261003", "TD1", 11.0, 3, 60.0, 10.0));
        p.today_position = 3;
        p.open_volume = 5;
        p.open_amount = 53.0;
        p.position_cost = 49.0;
        p.margin = 100.0;
        p.commission = 10.0;
        p.frozen_today = 1;
        let rows = p.to_query_fields("B", "I", "SHFE", "20261003", 1);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].PositionDate, b'2');
        assert_eq!(rows[0].Position, 2);
        assert_eq!(rows[1].PositionDate, b'1');
        assert_eq!(rows[1].Position, 3);
        assert_eq!(rows.iter().map(|r| r.Position).sum::<i32>(), p.volume());
        assert_eq!(rows.iter().map(|r| r.OpenVolume).sum::<i32>(), 3);
        assert!((rows.iter().map(|r| r.OpenAmount).sum::<f64>() - 33.0).abs() < 1e-9);
        assert!((rows.iter().map(|r| r.UseMargin).sum::<f64>() - p.margin).abs() < 1e-9);
        assert_eq!(rows.iter().map(|r| r.ShortFrozen).sum::<i32>(), p.frozen_today);
        assert_eq!(rows[0].YdPosition, 2);
        assert_eq!(rows[1].YdPosition, 0);
    }

    #[test]
    fn non_shfe_query_projection_keeps_one_row() {
        let mut p = Position::new("a", PositionSide::Short);
        p.yd_position = 2;
        p.today_position = 3;
        let rows = p.to_query_fields("B", "I", "DCE", "20261003", 1);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].Position, 5);
        assert_eq!(rows[0].TodayPosition, 3);
        assert_eq!(rows[0].YdPosition, 0);
    }

    #[test]
    fn same_product_cross_contract_and_different_product_isolation() {
        let catalog = fixture(true);
        let mut ledger = Ledger::new(10000.0);
        ledger.ensure_account("TEST", "alice");
        ledger.on_fill(&fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3), &catalog, 10.0, "20261003");
        ledger.on_fill(&fill("TESTA02", Direction::Sell, OffsetFlag::Open, 1), &catalog, 10.0, "20261003");
        assert_eq!(ledger.account("TEST", "alice").unwrap().used_margin, 300.0);
        ledger.on_fill(&fill("TESTB01", Direction::Sell, OffsetFlag::Open, 1), &catalog, 10.0, "20261003");
        assert_eq!(ledger.account("TEST", "alice").unwrap().used_margin, 450.0);
        let rows = ledger.product_group_margin("TEST", "alice", &catalog, "20261003");
        assert_eq!(rows.iter().map(|r| r.UseMargin).sum::<f64>(), 450.0);
    }

    #[test]
    fn closing_larger_side_switches_margin_side() {
        let catalog = fixture(true);
        let mut ledger = Ledger::new(10000.0);
        ledger.ensure_account("TEST", "alice");
        ledger.on_fill(&fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3), &catalog, 10.0, "20261003");
        ledger.on_fill(&fill("TESTA02", Direction::Sell, OffsetFlag::Open, 1), &catalog, 10.0, "20261003");
        ledger.on_fill(&fill("TESTA01", Direction::Sell, OffsetFlag::Close, 2), &catalog, 10.0, "20261003");
        assert_eq!(ledger.account("TEST", "alice").unwrap().used_margin, 150.0);
    }

    #[test]
    fn pending_orders_partial_fill_and_cancel_share_incremental_risk() {
        let catalog = fixture(true);
        let mut ledger = Ledger::new(350.0);
        ledger.ensure_account("TEST", "alice");
        ledger.on_fill(&fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3), &catalog, 10.0, "20261003");
        ledger.freeze("pending", "TEST", "alice", "TESTA02", PositionSide::Short, 300.0, 0.0, &catalog).unwrap();
        assert_eq!(ledger.account("TEST", "alice").unwrap().frozen_margin, 0.0);
        assert_eq!(ledger.freeze("extra", "TEST", "alice", "TESTA02", PositionSide::Short, 150.0, 0.0, &catalog), Err(ERR_FUNDS));
        let mut f = fill("TESTA02", Direction::Sell, OffsetFlag::Open, 1);
        f.order_key = "pending".into();
        f.volume_total_original = 2;
        ledger.on_fill(&f, &catalog, 10.0, "20261003");
        assert_eq!(ledger.account("TEST", "alice").unwrap().used_margin, 300.0);
        assert_eq!(ledger.account("TEST", "alice").unwrap().frozen_margin, 0.0);
        ledger.unfreeze_order("pending", &catalog);
        assert_eq!(ledger.account("TEST", "alice").unwrap().frozen_margin, 0.0);
        ledger.on_fill(&fill("TESTA01", Direction::Sell, OffsetFlag::Close, 2), &catalog, 10.0, "20261003");
        assert_eq!(ledger.account("TEST", "alice").unwrap().used_margin, 150.0);
    }

    #[test]
    fn broker_investor_exchange_and_mixed_rules_do_not_offset() {
        let mut catalog = fixture(true);
        let mut mixed = catalog.get("TESTA02").unwrap().clone();
        mixed.max_margin_side_algorithm = b'0';
        catalog.insert(mixed);
        let mut other = catalog.get("TESTA01").unwrap().clone();
        other.instrument_id = "OTHER".into();
        other.exchange_id = "OTHER".into();
        catalog.insert(other);
        catalog.insert_margin_rate(MarginRate { instrument_id: "OTHER".into(),
            short_margin_ratio_by_volume: 150.0, ..Default::default() });
        let mut ledger = Ledger::new(10000.0);
        ledger.ensure_account("TEST", "alice");
        ledger.ensure_account("OTHER", "alice");
        ledger.ensure_account("TEST", "bob");
        ledger.on_fill(&fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3), &catalog, 10.0, "20261003");
        ledger.on_fill(&fill("TESTA02", Direction::Sell, OffsetFlag::Open, 1), &catalog, 10.0, "20261003");
        ledger.on_fill(&fill("OTHER", Direction::Sell, OffsetFlag::Open, 1), &catalog, 10.0, "20261003");
        let mut f = fill("TESTA01", Direction::Sell, OffsetFlag::Open, 1);
        f.broker_id = to_fixed("OTHER");
        ledger.on_fill(&f, &catalog, 10.0, "20261003");
        f.broker_id = to_fixed("TEST");
        f.investor_id = to_fixed("bob");
        ledger.on_fill(&f, &catalog, 10.0, "20261003");
        assert_eq!(ledger.account("TEST", "alice").unwrap().used_margin, 600.0);
        assert_eq!(ledger.account("OTHER", "alice").unwrap().used_margin, 150.0);
        assert_eq!(ledger.account("TEST", "bob").unwrap().used_margin, 150.0);
    }

    #[test]
    fn mark_to_market_does_not_reprice_booked_margin_after_market_move() {
        let mut catalog = fixture(true);
        catalog.insert_margin_rate(MarginRate {
            instrument_id: "TESTA01".into(),
            long_margin_ratio_by_money: 0.1,
            ..Default::default()
        });
        let mut ledger = Ledger::new(10000.0);
        ledger.ensure_account("TEST", "alice");
        ledger.on_fill(&fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3), &catalog, 10.0, "20261003");
        let before = ledger.account("TEST", "alice").unwrap().used_margin;
        let prices = HashMap::from([("TESTA01".into(), 20.0)]);
        let previous = HashMap::from([("TESTA01".into(), 10.0)]);
        ledger.mark_to_market(&catalog, &prices, &previous, "20261003");
        assert_eq!(ledger.account("TEST", "alice").unwrap().used_margin, before);
        ledger.on_fill(&fill("TESTA01", Direction::Sell, OffsetFlag::Close, 1), &catalog, 10.0, "20261003");
        let after_partial_close = ledger.account("TEST", "alice").unwrap().used_margin;
        assert!((after_partial_close - before * 2.0 / 3.0).abs() < 1e-9);
        ledger.mark_to_market(&catalog, &HashMap::from([("TESTA01".into(), 30.0)]), &previous, "20261003");
        assert_eq!(ledger.account("TEST", "alice").unwrap().used_margin, after_partial_close);
        ledger.on_fill(&fill("TESTA01", Direction::Sell, OffsetFlag::Close, 1), &catalog, 10.0, "20261003");
        let rows = ledger.product_group_margin("TEST", "alice", &catalog, "20261003");
        assert_eq!(rows.iter().map(|r| r.UseMargin).sum::<f64>(), before / 3.0);
        ledger.on_fill(&fill("TESTA01", Direction::Sell, OffsetFlag::Close, 1), &catalog, 10.0, "20261003");
        let rows = ledger.product_group_margin("TEST", "alice", &catalog, "20261003");
        assert_eq!(rows.iter().map(|r| r.UseMargin).sum::<f64>(), 0.0);
    }

    #[test]
    fn disabled_algorithm_keeps_sum() {
        let catalog = fixture(false);
        let mut ledger = Ledger::new(10000.0);
        ledger.ensure_account("TEST", "alice");
        ledger.on_fill(&fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3), &catalog, 10.0, "20261003");
        ledger.on_fill(&fill("TESTA02", Direction::Sell, OffsetFlag::Open, 1), &catalog, 10.0, "20261003");
        assert_eq!(ledger.account("TEST", "alice").unwrap().used_margin, 450.0);
    }
}
