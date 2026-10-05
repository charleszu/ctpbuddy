//! Positions and per-lot details (DESIGN.md §8.6, notes/04).

use crate::Money;
use ctpbuddy_matching::Direction;
use ctpbuddy_wire::generated::{
    set_cstr, CThostFtdcInvestorPositionDetailField, CThostFtdcInvestorPositionField,
};

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
    pub margin: Money,
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
    pub close_profit: Money,
    pub close_profit_trade: Money,
    pub commission: Money,
    /// Lots of this detail already closed.
    pub close_volume: i32,
    /// Turnover of the closes attributed to this detail.
    pub close_amount: Money,
}

impl PositionDetail {
    pub fn new(
        open_date: &str,
        trade_id: &str,
        open_price: f64,
        volume: i32,
        margin: Money,
        margin_price: f64,
    ) -> Self {
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
            close_profit: Money::ZERO,
            close_profit_trade: Money::ZERO,
            commission: Money::ZERO,
            close_volume: 0,
            close_amount: Money::ZERO,
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
        f.CloseAmount = self.close_amount.to_f64();
        f.Margin = self
            .margin
            .ratio(self.volume as i64, self.open_volume.max(1) as i64)
            .to_f64();
        f.LastSettlementPrice = self.last_settlement_price;
        f.SettlementPrice = settlement_price;
        let basis = if self.open_date == trading_day {
            self.open_price
        } else {
            self.last_settlement_price
        };
        f.PositionProfitByDate =
            Position::detail_pnl(side, basis, last_price, self.volume, multiple).to_f64();
        f.PositionProfitByTrade =
            Position::detail_pnl(side, self.open_price, last_price, self.volume, multiple).to_f64();
        f.CloseProfitByDate = self.close_profit.to_f64();
        f.CloseProfitByTrade = self.close_profit_trade.to_f64();
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
    pub open_amount: Money,
    pub open_volume: i32,
    pub position_cost: Money,
    pub open_cost: Money,
    pub margin: Money,
    /// Volume reserved by outstanding close orders (today / yd split).
    pub frozen_today: i32,
    pub frozen_yd: i32,
    pub commission: Money,
    /// 逐日盯市 realized PnL (昨仓 against 昨结算, 今仓 against 开仓价).
    pub close_profit: Money,
    /// 逐笔对冲 realized PnL (every lot against its own 开仓价).
    pub close_profit_trade: Money,
    pub position_profit: Money,
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
            open_amount: Money::ZERO,
            open_volume: 0,
            position_cost: Money::ZERO,
            open_cost: Money::ZERO,
            margin: Money::ZERO,
            frozen_today: 0,
            frozen_yd: 0,
            commission: Money::ZERO,
            close_profit: Money::ZERO,
            close_profit_trade: Money::ZERO,
            position_profit: Money::ZERO,
            pre_settlement_price: 0.0,
            settlement_price: 0.0,
            details: Vec::new(),
        }
    }

    /// Record an opening fill as its own detail (notes/04 B2: a detail is
    /// created per opening fill, keyed by `(OpenDate, TradeID)`).
    pub fn add_detail(
        &mut self,
        open_date: &str,
        trade_id: &str,
        price: f64,
        volume: i32,
        margin: Money,
        margin_price: f64,
    ) {
        self.details.push(PositionDetail::new(
            open_date,
            trade_id,
            price,
            volume,
            margin,
            margin_price,
        ));
    }

    pub fn add_bootstrap_detail(
        &mut self,
        open_date: &str,
        trade_id: &str,
        price: f64,
        volume: i32,
        margin: Money,
        pre_settlement: f64,
        multiple: i32,
    ) {
        self.yd_position += volume;
        self.yd_initial += volume;
        self.open_cost += Money::from_f64(price * volume as f64 * multiple as f64);
        self.position_cost += Money::from_f64(pre_settlement * volume as f64 * multiple as f64);
        self.margin += margin;
        self.settlement_price = pre_settlement;
        let mut detail =
            PositionDetail::new(open_date, trade_id, price, volume, margin, pre_settlement);
        detail.bootstrap = true;
        detail.last_settlement_price = pre_settlement;
        self.details.push(detail);
        self.details.sort_by(|a, b| {
            a.open_date
                .cmp(&b.open_date)
                .then(a.trade_id.cmp(&b.trade_id))
        });
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
    pub fn detail_pnl(side: PositionSide, basis: f64, price: f64, n: i32, multiple: i32) -> Money {
        let sign = match side {
            PositionSide::Long => 1.0,
            PositionSide::Short => -1.0,
        };
        Money::from_f64((price - basis) * sign * n as f64 * multiple as f64)
    }

    pub fn volume(&self) -> i32 {
        self.today_position + self.yd_position
    }

    pub fn avg_cost(&self, volume_multiple: i32) -> f64 {
        let denom = self.open_volume as f64 * volume_multiple as f64;
        if denom > 0.0 {
            self.open_amount.to_f64() / denom
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
        if !ctpbuddy_matching::rules_for(exchange_id).splits_today_yesterday {
            return vec![base];
        }
        let yd = self.detail_bucket_field(base, trading_day, mult, false);
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
        let frozen = if today {
            self.frozen_today
        } else {
            self.frozen_yd
        };
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
            field.CloseProfit += detail.close_profit.to_f64();
            field.CloseProfitByDate += detail.close_profit.to_f64();
            field.CloseProfitByTrade += detail.close_profit_trade.to_f64();
            field.Commission += detail.commission.to_f64();
            field.CloseVolume += detail.close_volume;
            field.CloseAmount += detail.close_amount.to_f64();
            if today && !detail.bootstrap {
                field.OpenVolume += detail.open_volume;
                field.OpenAmount += detail.open_price * detail.open_volume as f64 * mult as f64;
            }
            if volume > 0 {
                field.Position += volume;
                field.OpenCost += detail.open_price * volume as f64 * mult as f64;
                field.PositionCost += basis * volume as f64 * mult as f64;
                field.UseMargin += detail
                    .margin
                    .ratio(volume as i64, detail.open_volume.max(1) as i64)
                    .to_f64();
                field.PositionProfit +=
                    Position::detail_pnl(self.side, basis, self.settlement_price, volume, mult)
                        .to_f64();
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
        f.PositionDate = if !ctpbuddy_matching::rules_for(exchange_id).splits_today_yesterday
            || (self.today_position > 0 && self.yd_position == 0)
        {
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
        f.OpenAmount = self.open_amount.to_f64();
        f.OpenCost = self.open_cost.to_f64();
        f.PositionCost = self.position_cost.to_f64();
        f.UseMargin = self.margin.to_f64();
        f.Commission = self.commission.to_f64();
        f.CloseProfit = self.close_profit.to_f64();
        f.PositionProfit = self.position_profit.to_f64();
        f.PreSettlementPrice = self.pre_settlement_price;
        f.SettlementPrice = self.settlement_price;
        f.CloseProfitByDate = self.close_profit.to_f64();
        f.CloseProfitByTrade = self.close_profit_trade.to_f64();
        let cost = self.avg_cost(mult);
        f.PositionCostOffset = cost;
        f.SettlementID = 1;
        f
    }
}
