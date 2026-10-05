//! Reference data: contracts + margin / commission / order-commission rates.
//!
//! The shapes here are **not** CTPBuddy's invention — each table mirrors the
//! CTP query response a real client receives for the same information
//! (notes/04 G):
//!
//! | table | CTP query it mirrors |
//! |---|---|
//! | `Instrument` | `ReqQryInstrument` |
//! | `MarginRate` | `ReqQryInstrumentMarginRate` |
//! | `CommissionRate` | `ReqQryInstrumentCommissionRate` |
//! | `OrderCommRate` | `ReqQryInstrumentOrderCommRate` (申报费) |
//! | `TradingParams` | `ReqQryBrokerTradingParams` (`MarginPriceType`) |
//!
//! This matters for more than tidiness. notes/04 C3: the rate the counter
//! *charges* you is the one `ReqQryInstrumentMarginRate` returns (公司费率),
//! while `ReqQryInstrument`'s `LongMarginRatio` is the *exchange* rate and is
//! **never used in the calculation**. A design where the query surface and the
//! calculation surface are separate data structures cannot honour that rule —
//! they drift, and the drift is invisible until a client compares a query
//! response against its own margin total.
//!
//! So: one [`RefData`] holds all five tables, the ledger computes from them,
//! and the query handlers project the very same rows into the CTP structs.
//! A client that cross-checks `ReqQryInstrumentMarginRate` against
//! `ReqQryTradingAccount.CurrMargin` sees consistent numbers, exactly as on a
//! real desk.
//!
//! # Provenance
//!
//! Values come from the user, never from a hardcoded table. The Python
//! provider protocol (`py/ctpbuddy/refdata`) materializes a directory of
//! JSONL files that this module reads — the same split as the market-data
//! plugin protocol (DESIGN §7.3): plugins run on the Python side, the core
//! consumes a normalized file with no interpreter in the loop.
//!
//! A data set with only *some* of the tables is perfectly normal and loads
//! fine: a desk charging no 申报费 ships no `order_comm_rates.jsonl`, and one
//! running at zero commission ships none either. Absent tables mean "this desk
//! has no such rule", not "error".
//!
//! [`Catalog::bundled`] is the one built-in data set — real contracts, no
//! invented fees. See its doc comment.

use std::collections::HashMap;

mod jsonl;
use jsonl::read_jsonl;

use ctpbuddy_wire::generated::{
    set_cstr, CThostFtdcBrokerTradingParamsField, CThostFtdcInstrumentCommissionRateField,
    CThostFtdcInstrumentField, CThostFtdcInstrumentMarginRateField,
    CThostFtdcInstrumentOrderCommRateField,
};

use crate::Direction;

/// `MarginPriceType` — which price today's margin is computed against
/// (notes/04 C2). Yesterday's positions **always** use the previous
/// settlement price regardless of this setting; only 今仓 follows it.
pub const MPT_PRE_SETTLEMENT: u8 = b'1';
pub const MPT_SETTLEMENT: u8 = b'2';
pub const MPT_AVERAGE: u8 = b'3';
pub const MPT_OPEN_PRICE: u8 = b'4';

/// `THOST_FTDC_HF_Speculation` — the default hedge flag for rate lookups.
pub const HEDGE_FLAG_SPECULATION: u8 = b'1';

/// The reason a margin number is what it is. Kept as an explicit input rather
/// than than a `f64` so the price-type rule (notes/04 C2) is a type-level
/// statement and the ledger cannot quietly substitute the latest price.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarginPrice {
    /// 昨结算价 — the only price yesterday's positions ever use.
    PreSettlement(f64),
    /// 最新价 — `MarginPriceType == '2'`.
    Last(f64),
    /// 成交均价 — `MarginPriceType == '3'`.
    Average(f64),
    /// 开仓价 — `MarginPriceType == '4'`; the fill price of an opening fill.
    Open(f64),
}

impl MarginPrice {
    pub fn value(&self) -> f64 {
        match self {
            MarginPrice::PreSettlement(p)
            | MarginPrice::Last(p)
            | MarginPrice::Average(p)
            | MarginPrice::Open(p) => *p,
        }
    }
}

/// One contract's static attributes — `ReqQryInstrument` rows.
#[derive(Clone, Debug)]
pub struct Instrument {
    pub instrument_id: String,
    pub exchange_id: String,
    pub exchange_inst_id: String,
    pub instrument_name: String,
    pub product_id: String,
    /// `THOST_FTDC_PC_*`: '1' future, '5' option.
    pub product_class: u8,
    pub delivery_year: i32,
    pub delivery_month: i32,
    pub create_date: String,
    pub open_date: String,
    pub expire_date: String,
    pub start_deliv_date: String,
    pub end_deliv_date: String,
    /// `InstLifePhase`: '1' live. CTP filters nothing on this for futures.
    pub inst_life_phase: u8,
    pub is_trading: bool,
    pub position_type: u8,
    pub position_date_type: u8,
    pub volume_multiple: i32,
    pub price_tick: f64,
    pub min_limit_order_volume: i32,
    pub max_limit_order_volume: i32,
    pub min_market_order_volume: i32,
    pub max_market_order_volume: i32,
    /// **交易所**保证金率（仅展示，计算不用）— notes/04 C3.
    pub long_margin_ratio: f64,
    pub short_margin_ratio: f64,
    /// `MaxMarginSideAlgorithm` — 品种内大单边标志（notes/04 C4）。
    pub max_margin_side_algorithm: u8,
    pub options_type: u8,
    pub underlying_multiple: f64,
    pub combination_type: u8,
}

impl Instrument {
    pub fn new(instrument_id: &str, exchange_id: &str) -> Self {
        Instrument {
            instrument_id: instrument_id.to_string(),
            exchange_id: exchange_id.to_string(),
            exchange_inst_id: instrument_id.to_string(),
            instrument_name: instrument_id.to_string(),
            product_id: product_of(instrument_id),
            product_class: b'1',
            delivery_year: 0,
            delivery_month: 0,
            create_date: String::new(),
            open_date: String::new(),
            expire_date: String::new(),
            start_deliv_date: String::new(),
            end_deliv_date: String::new(),
            inst_life_phase: b'1',
            is_trading: true,
            // `THOST_FTDC_PT_*`: '1' Net 净持仓; '2' Gross 综合持仓。国内期货合约
            // 一律综合持仓（随包 refdata 里全是 '2'），合成合约跟它保持一致。
            position_type: b'2',
            // `THOST_FTDC_PDT_*` (ThostFtdcUserApiDataType.h): '1' UseHistory =
            // 使用历史持仓（分今昨仓，上期所/能源中心）; '2' NoUseHistory = 不使用
            // 历史持仓（不分今昨，其他四所）。合成合约默认分今昨；refdata 有值时
            // 以文件为准（`load_jsonl_dir`）。
            position_date_type: b'1',
            volume_multiple: 1,
            price_tick: 0.01,
            min_limit_order_volume: 1,
            max_limit_order_volume: 500,
            min_market_order_volume: 1,
            max_market_order_volume: 500,
            long_margin_ratio: 0.0,
            short_margin_ratio: 0.0,
            max_margin_side_algorithm: b'0',
            options_type: b'0',
            underlying_multiple: 1.0,
            combination_type: b'0',
        }
    }

    /// Delivery dates are a pure function of the contract code when the
    /// provider does not supply them (`rb2610` -> 2026-10; CZCE's 3-digit
    /// `TA609` -> 2026-9). A synthetic desk has no other calendar to draw on.
    ///
    /// Strictly 「缺省才补」: a value the provider already supplied (delivery
    /// year/month, any of the five dates) is never overwritten — the real
    /// `ExpireDate` of `rb2610` is the 15th only by coincidence.
    pub fn fill_dates_from_code(&mut self) {
        if let Some((y, m)) = parse_delivery_ym(&self.instrument_id) {
            if self.delivery_year == 0 {
                self.delivery_year = y;
            }
            if self.delivery_month == 0 {
                self.delivery_month = m;
            }
            // derived dates follow the provider's year/month when present
            let (y, m) = (self.delivery_year, self.delivery_month);
            if self.create_date.is_empty() {
                self.create_date = format!("{y:04}{m:02}01");
            }
            if self.open_date.is_empty() {
                self.open_date = format!("{y:04}{m:02}01");
            }
            if self.expire_date.is_empty() {
                self.expire_date = format!("{y:04}{m:02}15");
            }
            if self.start_deliv_date.is_empty() {
                self.start_deliv_date = format!("{y:04}{m:02}16");
            }
            if self.end_deliv_date.is_empty() {
                self.end_deliv_date = format!("{y:04}{m:02}15");
            }
        }
    }

    pub fn min_volume(&self, price_type: u8) -> i32 {
        if price_type == b'2' {
            self.min_limit_order_volume
        } else {
            self.min_market_order_volume
        }
    }

    pub fn max_volume(&self, price_type: u8) -> i32 {
        if price_type == b'2' {
            self.max_limit_order_volume
        } else {
            self.max_market_order_volume
        }
    }

    /// `ReqQryInstrument` response row.
    pub fn to_field(&self) -> CThostFtdcInstrumentField {
        let mut f = CThostFtdcInstrumentField::zeroed();
        set_cstr(&mut f.InstrumentID, &self.instrument_id);
        set_cstr(&mut f.ExchangeID, &self.exchange_id);
        set_cstr(&mut f.ExchangeInstID, &self.exchange_inst_id);
        set_cstr(&mut f.InstrumentName, &self.instrument_name);
        set_cstr(&mut f.ProductID, &self.product_id);
        f.ProductClass = self.product_class;
        f.DeliveryYear = self.delivery_year;
        f.DeliveryMonth = self.delivery_month;
        f.VolumeMultiple = self.volume_multiple;
        f.PriceTick = self.price_tick;
        f.CreateDate = date8(&self.create_date);
        f.OpenDate = date8(&self.open_date);
        f.ExpireDate = date8(&self.expire_date);
        f.StartDelivDate = date8(&self.start_deliv_date);
        f.EndDelivDate = date8(&self.end_deliv_date);
        f.InstLifePhase = self.inst_life_phase;
        f.IsTrading = i32::from(self.is_trading);
        f.PositionType = self.position_type;
        f.PositionDateType = self.position_date_type;
        f.LongMarginRatio = self.long_margin_ratio;
        f.ShortMarginRatio = self.short_margin_ratio;
        f.MaxMarginSideAlgorithm = self.max_margin_side_algorithm;
        f.OptionsType = self.options_type;
        f.UnderlyingMultiple = self.underlying_multiple;
        f.CombinationType = self.combination_type;
        f.MaxMarketOrderVolume = self.max_market_order_volume;
        f.MinMarketOrderVolume = self.min_market_order_volume;
        f.MaxLimitOrderVolume = self.max_limit_order_volume;
        f.MinLimitOrderVolume = self.min_limit_order_volume;
        f
    }
}

fn date8(s: &str) -> [u8; 9] {
    let mut out = [0u8; 9];
    if s.len() >= 8 {
        set_cstr(&mut out, &s[..8]);
    }
    out
}

/// 公司保证金率 — `ReqQryInstrumentMarginRate` rows. This is the rate the
/// counter actually charges (notes/04 C3).
#[derive(Clone, Debug, Default)]
pub struct MarginRate {
    pub broker_id: String,
    pub investor_id: String,
    pub hedge_flag: u8,
    pub exchange_id: String,
    pub instrument_id: String,
    pub invest_unit_id: String,
    pub investor_range: u8,
    pub is_relative: i32,
    pub long_margin_ratio_by_money: f64,
    pub long_margin_ratio_by_volume: f64,
    pub short_margin_ratio_by_money: f64,
    pub short_margin_ratio_by_volume: f64,
}

impl MarginRate {
    pub fn by_money(&self, direction: Direction) -> f64 {
        match direction {
            Direction::Buy => self.long_margin_ratio_by_money,
            Direction::Sell => self.short_margin_ratio_by_money,
        }
    }

    pub fn by_volume(&self, direction: Direction) -> f64 {
        match direction {
            Direction::Buy => self.long_margin_ratio_by_volume,
            Direction::Sell => self.short_margin_ratio_by_volume,
        }
    }

    /// 期货保证金 = `(ByVolume + ByMoney × Price × VolumeMultiple) × Volume`
    /// (notes/04 C1). The 按手数 term is zero for every current futures
    /// contract but the field is real and the formula carries it.
    pub fn margin(&self, direction: Direction, price: f64, multiple: i32, volume: i32) -> f64 {
        let per_lot =
            self.by_volume(direction) + self.by_money(direction) * price * multiple as f64;
        per_lot * volume as f64
    }

    pub fn to_field(&self) -> CThostFtdcInstrumentMarginRateField {
        let mut f = CThostFtdcInstrumentMarginRateField::zeroed();
        set_cstr(&mut f.BrokerID, &self.broker_id);
        set_cstr(&mut f.InvestorID, &self.investor_id);
        f.HedgeFlag = self.hedge_flag;
        set_cstr(&mut f.ExchangeID, &self.exchange_id);
        set_cstr(&mut f.InstrumentID, &self.instrument_id);
        set_cstr(&mut f.InvestUnitID, &self.invest_unit_id);
        f.InvestorRange = self.investor_range;
        f.IsRelative = self.is_relative;
        f.LongMarginRatioByMoney = self.long_margin_ratio_by_money;
        f.LongMarginRatioByVolume = self.long_margin_ratio_by_volume;
        f.ShortMarginRatioByMoney = self.short_margin_ratio_by_money;
        f.ShortMarginRatioByVolume = self.short_margin_ratio_by_volume;
        f
    }
}

/// 手续费率 — `ReqQryInstrumentCommissionRate` rows. Six independent rates:
/// 开仓 / 平仓 / 平今, each 按金额率 + 按手数费 (notes/04 D1).
#[derive(Clone, Debug, Default)]
pub struct CommissionRate {
    pub broker_id: String,
    pub investor_id: String,
    pub exchange_id: String,
    pub instrument_id: String,
    pub invest_unit_id: String,
    pub investor_range: u8,
    pub biz_type: u8,
    pub open_ratio_by_money: f64,
    pub open_ratio_by_volume: f64,
    pub close_ratio_by_money: f64,
    pub close_ratio_by_volume: f64,
    pub close_today_ratio_by_money: f64,
    pub close_today_ratio_by_volume: f64,
}

/// Which of the three rate pairs applies to a fill.
///
/// The split is finer than the 今/昨仓 split: notes/04 B3 warns that even
/// 大商所, which does not distinguish 今仓/昨仓 positions, still prices 平今
/// and 平昨 with different rates — unifying them produces a visible
/// divergence. A plain `Close` fill is priced by *which positions it
/// actually consumed* (today first), not by its own offset flag, because a
/// `Close` order routinely eats both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommissionKind {
    Open,
    /// 平昨 — closing yesterday's positions.
    CloseYesterday,
    /// 平今 — closing today's positions.
    CloseToday,
}

impl CommissionRate {
    /// 手续费 = `数量 × (成交价 × 乘数 × RatioByMoney + RatioByVolume)` — the
    /// two methods are **summed**, not maxed. `RatioByVolume` is the per-lot
    /// fee, not a floor (notes/04 D1).
    pub fn commission(&self, kind: CommissionKind, price: f64, multiple: i32, volume: i32) -> f64 {
        let (by_money, by_volume) = match kind {
            CommissionKind::Open => (self.open_ratio_by_money, self.open_ratio_by_volume),
            CommissionKind::CloseYesterday => {
                (self.close_ratio_by_money, self.close_ratio_by_volume)
            }
            CommissionKind::CloseToday => (
                self.close_today_ratio_by_money,
                self.close_today_ratio_by_volume,
            ),
        };
        volume as f64 * (price * multiple as f64 * by_money + by_volume)
    }

    pub fn to_field(&self) -> CThostFtdcInstrumentCommissionRateField {
        let mut f = CThostFtdcInstrumentCommissionRateField::zeroed();
        set_cstr(&mut f.BrokerID, &self.broker_id);
        set_cstr(&mut f.InvestorID, &self.investor_id);
        set_cstr(&mut f.ExchangeID, &self.exchange_id);
        set_cstr(&mut f.InstrumentID, &self.instrument_id);
        set_cstr(&mut f.InvestUnitID, &self.invest_unit_id);
        f.InvestorRange = self.investor_range;
        f.BizType = self.biz_type;
        f.OpenRatioByMoney = self.open_ratio_by_money;
        f.OpenRatioByVolume = self.open_ratio_by_volume;
        f.CloseRatioByMoney = self.close_ratio_by_money;
        f.CloseRatioByVolume = self.close_ratio_by_volume;
        f.CloseTodayRatioByMoney = self.close_today_ratio_by_money;
        f.CloseTodayRatioByVolume = self.close_today_ratio_by_volume;
        f
    }
}

/// 申报费 — `ReqQryInstrumentOrderCommRate` rows. 报单与撤单各收一笔
/// (notes/04 D2 notes it as 中金所特有 in practice; nothing here whitelists an
/// exchange — a row exists exactly when the supplied data has one).
/// 报单 and 撤单 each cost money, and a FAK/FOK's automatic exchange-side
/// cancel counts as a second one.
#[derive(Clone, Debug, Default)]
pub struct OrderCommRate {
    pub broker_id: String,
    pub investor_id: String,
    pub hedge_flag: u8,
    pub exchange_id: String,
    pub instrument_id: String,
    pub invest_unit_id: String,
    pub investor_range: u8,
    pub order_comm_by_volume: f64,
    pub order_action_comm_by_volume: f64,
    pub order_comm_by_trade: f64,
    pub order_action_comm_by_trade: f64,
}

impl OrderCommRate {
    /// 报单 + 撤单 一次性收齐：CTP 在成交/撤单时按次收取，盘中可用资金
    /// 不含申报费，只体现在结算单（notes/04 D2）。`trades` counts FAK/FOK
    /// 的自动撤单 as one extra 撤单, so a completed FAK pays twice.
    pub fn insert_fee(&self, volume: i32) -> f64 {
        self.order_comm_by_volume * volume as f64 + self.order_comm_by_trade
    }

    pub fn action_fee(&self, volume: i32) -> f64 {
        self.order_action_comm_by_volume * volume as f64 + self.order_action_comm_by_trade
    }

    pub fn to_field(&self) -> CThostFtdcInstrumentOrderCommRateField {
        let mut f = CThostFtdcInstrumentOrderCommRateField::zeroed();
        set_cstr(&mut f.BrokerID, &self.broker_id);
        set_cstr(&mut f.InvestorID, &self.investor_id);
        f.HedgeFlag = self.hedge_flag;
        set_cstr(&mut f.ExchangeID, &self.exchange_id);
        set_cstr(&mut f.InstrumentID, &self.instrument_id);
        set_cstr(&mut f.InvestUnitID, &self.invest_unit_id);
        f.InvestorRange = self.investor_range;
        f.OrderCommByVolume = self.order_comm_by_volume;
        f.OrderActionCommByVolume = self.order_action_comm_by_volume;
        f.OrderCommByTrade = self.order_comm_by_trade;
        f.OrderActionCommByTrade = self.order_action_comm_by_trade;
        f
    }
}

/// 交易参数 — `ReqQryBrokerTradingParams`. One row per broker, not per
/// instrument; `MarginPriceType` is the field with teeth (notes/04 C2).
#[derive(Clone, Debug)]
pub struct TradingParams {
    pub broker_id: String,
    pub investor_id: String,
    pub account_id: String,
    pub currency_id: String,
    pub margin_price_type: u8,
    pub algorithm: u8,
    /// Whether `Available` includes today's close profit. CTP desks vary; the
    /// default (`1`) matches the conservative reading of notes/04 A1, where
    /// `Balance` is 动态权益 and therefore already contains 平仓盈亏.
    pub avail_include_close_profit: u8,
    pub option_royalty_price_type: u8,
}

impl Default for TradingParams {
    fn default() -> Self {
        TradingParams {
            broker_id: String::new(),
            investor_id: String::new(),
            account_id: String::new(),
            currency_id: "CNY".to_string(),
            // 昨结算价: today's margin then does not move with the market,
            // which is the least surprising default for a test counter.
            margin_price_type: MPT_PRE_SETTLEMENT,
            algorithm: b'1',
            avail_include_close_profit: b'1',
            option_royalty_price_type: b'1',
        }
    }
}

impl TradingParams {
    pub fn to_field(&self) -> CThostFtdcBrokerTradingParamsField {
        let mut f = CThostFtdcBrokerTradingParamsField::zeroed();
        set_cstr(&mut f.BrokerID, &self.broker_id);
        set_cstr(&mut f.InvestorID, &self.investor_id);
        set_cstr(&mut f.AccountID, &self.account_id);
        set_cstr(&mut f.CurrencyID, &self.currency_id);
        f.MarginPriceType = self.margin_price_type;
        f.Algorithm = self.algorithm;
        f.AvailIncludeCloseProfit = self.avail_include_close_profit;
        f.OptionRoyaltyPriceType = self.option_royalty_price_type;
        f
    }
}

/// All reference data, indexed for the two access patterns: the engine's
/// per-fill lookups (by instrument) and the query handlers' filtered scans
/// (by broker / investor / instrument).
#[derive(Clone, Debug, Default)]
pub struct RefData {
    instruments: HashMap<String, Instrument>,
    margin_rates: HashMap<String, MarginRate>,
    commission_rates: HashMap<String, CommissionRate>,
    order_comm_rates: HashMap<String, OrderCommRate>,
    params: TradingParams,
}

impl RefData {
    pub fn new() -> Self {
        RefData::default()
    }

    pub fn insert_instrument(&mut self, mut i: Instrument) {
        i.fill_dates_from_code();
        self.instruments.insert(i.instrument_id.clone(), i);
    }

    pub fn insert_margin_rate(&mut self, r: MarginRate) {
        self.margin_rates.insert(r.instrument_id.clone(), r);
    }

    pub fn insert_commission_rate(&mut self, r: CommissionRate) {
        self.commission_rates.insert(r.instrument_id.clone(), r);
    }

    pub fn insert_order_comm_rate(&mut self, r: OrderCommRate) {
        self.order_comm_rates.insert(r.instrument_id.clone(), r);
    }

    pub fn set_params(&mut self, p: TradingParams) {
        self.params = p;
    }

    pub fn params(&self) -> &TradingParams {
        &self.params
    }

    pub fn instrument(&self, id: &str) -> Option<&Instrument> {
        self.instruments.get(id)
    }

    pub fn product_margin_algorithm(&self, id: &str) -> bool {
        self.instrument(id)
            .map(|i| i.max_margin_side_algorithm == b'1')
            .unwrap_or(false)
    }

    pub fn margin_rate(&self, id: &str) -> Option<&MarginRate> {
        self.margin_rates.get(id)
    }

    pub fn commission_rate(&self, id: &str) -> Option<&CommissionRate> {
        self.commission_rates.get(id)
    }

    pub fn order_comm_rate(&self, id: &str) -> Option<&OrderCommRate> {
        self.order_comm_rates.get(id)
    }

    pub fn instruments(&self) -> impl Iterator<Item = &Instrument> {
        self.instruments.values()
    }

    pub fn margin_rate_rows(&self) -> impl Iterator<Item = &MarginRate> {
        self.margin_rates.values()
    }

    pub fn commission_rate_rows(&self) -> impl Iterator<Item = &CommissionRate> {
        self.commission_rates.values()
    }

    pub fn order_comm_rate_rows(&self) -> impl Iterator<Item = &OrderCommRate> {
        self.order_comm_rates.values()
    }

    pub fn instrument_count(&self) -> usize {
        self.instruments.len()
    }

    // ---- ordered scans for the query handlers ----
    //
    // `HashMap::values()` yields in hash order, which varies per process. A
    // query stream whose row order is unstable would break journal replay
    // hashes for no good reason, so every scan used by a `ReqQry*` handler
    // goes through these instead.

    /// Every margin rate, ordered by instrument.
    pub fn margin_rate_rows_sorted(&self) -> Vec<&MarginRate> {
        let mut v: Vec<&MarginRate> = self.margin_rates.values().collect();
        v.sort_by(|a, b| a.instrument_id.cmp(&b.instrument_id));
        v
    }

    /// Every commission rate, ordered by instrument.
    pub fn commission_rate_rows_sorted(&self) -> Vec<&CommissionRate> {
        let mut v: Vec<&CommissionRate> = self.commission_rates.values().collect();
        v.sort_by(|a, b| a.instrument_id.cmp(&b.instrument_id));
        v
    }

    /// Every 报单/撤单费 row, ordered by instrument.
    pub fn order_comm_rate_rows_sorted(&self) -> Vec<&OrderCommRate> {
        let mut v: Vec<&OrderCommRate> = self.order_comm_rates.values().collect();
        v.sort_by(|a, b| a.instrument_id.cmp(&b.instrument_id));
        v
    }

    // ---- calculation helpers (the ledger calls these, not the raw fields) ----

    /// 保证金 for `volume` lots. Falls back to the **exchange** rate from
    /// `ReqQryInstrument` only when no 公司费率 exists — a desk that supplied
    /// nothing still gets a plausible number instead of zero margin.
    pub fn margin(
        &self,
        instrument_id: &str,
        direction: Direction,
        price: MarginPrice,
        volume: i32,
    ) -> f64 {
        let multiple = self
            .instrument(instrument_id)
            .map(|i| i.volume_multiple)
            .unwrap_or(1);
        let p = price.value();
        if let Some(r) = self.margin_rate(instrument_id) {
            return r.margin(direction, p, multiple, volume);
        }
        if let Some(i) = self.instrument(instrument_id) {
            return p
                * volume as f64
                * multiple as f64
                * match direction {
                    Direction::Buy => i.long_margin_ratio,
                    Direction::Sell => i.short_margin_ratio,
                };
        }
        0.0
    }

    /// 手续费 for one fill leg. The per-leg split is what makes 平今/平昨
    /// observable: a `Close` fill that eats 2 lots of yesterday and 1 of
    /// today is charged as two legs, not one blended rate.
    pub fn commission(
        &self,
        instrument_id: &str,
        kind: CommissionKind,
        price: f64,
        volume: i32,
    ) -> f64 {
        let multiple = self
            .instrument(instrument_id)
            .map(|i| i.volume_multiple)
            .unwrap_or(1);
        if let Some(r) = self.commission_rate(instrument_id) {
            return r.commission(kind, price, multiple, volume);
        }
        // No rate table: fee 0 is a legitimate answer for a desk that does not
        // charge commission, and guessing a rate would be worse than honest.
        0.0
    }

    /// [`margin_price`] with the caller's own price arguments filled from
    /// what the ledger has at fill time: `pre_settlement` from the position,
    /// and the fill price standing in for 最新价 / 成交均价 / 开仓价 alike —
    /// at the instant of a single fill those are the same number, and the
    /// three settings only diverge once mark-to-market re-runs against later
    /// prints (which is [`crate::refdata::RefData::margin`]'s job, not this
    /// one's).
    pub fn margin_price_for(
        &self,
        is_today: bool,
        pre_settlement: f64,
        fill_price: f64,
    ) -> MarginPrice {
        self.margin_price(is_today, pre_settlement, fill_price, fill_price, fill_price)
    }

    /// Resolve the margin price for a fill.
    ///
    /// `is_today` is the position leg being margined: yesterday's positions
    /// are pinned to `pre_settlement` whatever `MarginPriceType` says
    /// (notes/04 C2: "昨仓一律用昨结算价计算"). Today's follow the setting.
    pub fn margin_price(
        &self,
        is_today: bool,
        pre_settlement: f64,
        last: f64,
        average: f64,
        open_price: f64,
    ) -> MarginPrice {
        if !is_today {
            return MarginPrice::PreSettlement(pre_settlement);
        }
        match self.params.margin_price_type {
            MPT_SETTLEMENT => MarginPrice::Last(last),
            MPT_AVERAGE => MarginPrice::Average(average),
            MPT_OPEN_PRICE => MarginPrice::Open(open_price),
            // '1' and anything unrecognized: 昨结算价
            _ => MarginPrice::PreSettlement(pre_settlement),
        }
    }

    // ---- loading ----

    /// Load a canonical JSONL directory produced by `ctpbuddy refdata export`
    /// (or any provider that writes the same five files). Missing rate tables
    /// are legal — they mean "this desk has no such rule" — but a missing
    /// instrument table is an error, since nothing can trade without it.
    pub fn load_jsonl_dir(dir: &str) -> Result<Self, String> {
        let mut rd = RefData::new();
        let instruments = read_jsonl(&format!("{dir}/instruments.jsonl"))?;
        if instruments.is_empty() {
            return Err(format!(
                "{dir}/instruments.jsonl 为空（instruments 表是必填的）"
            ));
        }
        for (n, row) in instruments.iter().enumerate() {
            let iid = row
                .str("instrument_id")
                .ok_or_else(|| format!("instruments.jsonl line {}: 缺 instrument_id", n + 1))?;
            let exch = row.str("exchange_id").unwrap_or_default();
            let mut inst = Instrument::new(&iid, &exch);
            if let Some(v) = row.str("instrument_name") {
                inst.instrument_name = v;
            }
            if let Some(v) = row.str("exchange_inst_id") {
                inst.exchange_inst_id = v;
            }
            if let Some(v) = row.str("product_id") {
                inst.product_id = v;
            }
            if let Some(v) = row.num("volume_multiple") {
                inst.volume_multiple = v as i32;
            }
            if let Some(v) = row.num("price_tick") {
                inst.price_tick = v;
            }
            if let Some(v) = row.int("min_limit_order_volume") {
                inst.min_limit_order_volume = v;
            }
            if let Some(v) = row.int("max_limit_order_volume") {
                inst.max_limit_order_volume = v;
            }
            if let Some(v) = row.int("min_market_order_volume") {
                inst.min_market_order_volume = v;
            }
            if let Some(v) = row.int("max_market_order_volume") {
                inst.max_market_order_volume = v;
            }
            if let Some(v) = row.num("long_margin_ratio") {
                inst.long_margin_ratio = v;
            }
            if let Some(v) = row.num("short_margin_ratio") {
                inst.short_margin_ratio = v;
            }
            if let Some(v) = row.int("delivery_year") {
                inst.delivery_year = v;
            }
            if let Some(v) = row.int("delivery_month") {
                inst.delivery_month = v;
            }
            if let Some(v) = row
                .str("max_margin_side_algorithm")
                .and_then(|s| s.as_bytes().first().copied())
            {
                inst.max_margin_side_algorithm = v;
            }
            // 生命周期日期：文件有值就用文件的，`insert_instrument` 只补缺省。
            if let Some(v) = row.str("create_date") {
                inst.create_date = v;
            }
            if let Some(v) = row.str("open_date") {
                inst.open_date = v;
            }
            if let Some(v) = row.str("expire_date") {
                inst.expire_date = v;
            }
            if let Some(v) = row.str("start_deliv_date") {
                inst.start_deliv_date = v;
            }
            if let Some(v) = row.str("end_deliv_date") {
                inst.end_deliv_date = v;
            }
            // 单字符枚举（官方 `THOST_FTDC_*` 编码，文件里是单字符串）
            if let Some(v) = row
                .str("position_type")
                .and_then(|s| s.as_bytes().first().copied())
            {
                inst.position_type = v;
            }
            if let Some(v) = row
                .str("position_date_type")
                .and_then(|s| s.as_bytes().first().copied())
            {
                inst.position_date_type = v;
            }
            if let Some(v) = row
                .str("inst_life_phase")
                .and_then(|s| s.as_bytes().first().copied())
            {
                inst.inst_life_phase = v;
            }
            if let Some(v) = row
                .str("product_class")
                .and_then(|s| s.as_bytes().first().copied())
            {
                inst.product_class = v;
            }
            if let Some(v) = row
                .str("options_type")
                .and_then(|s| s.as_bytes().first().copied())
            {
                inst.options_type = v;
            }
            if let Some(v) = row
                .str("combination_type")
                .and_then(|s| s.as_bytes().first().copied())
            {
                inst.combination_type = v;
            }
            // `is_trading`: 0/1 (or true/false) — 0 让引擎拒单为 17 合约不能交易
            if let Some(v) = row.int("is_trading") {
                inst.is_trading = v != 0;
            }
            if let Some(v) = row.num("underlying_multiple") {
                inst.underlying_multiple = v;
            }
            rd.insert_instrument(inst);
        }
        for (n, row) in read_jsonl(&format!("{dir}/margin_rates.jsonl"))?
            .iter()
            .enumerate()
        {
            let mut r = MarginRate {
                hedge_flag: HEDGE_FLAG_SPECULATION,
                ..Default::default()
            };
            r.broker_id = row.str("broker_id").unwrap_or_default();
            r.investor_id = row.str("investor_id").unwrap_or_default();
            r.exchange_id = row.str("exchange_id").unwrap_or_default();
            r.instrument_id = row
                .str("instrument_id")
                .ok_or_else(|| format!("margin_rates.jsonl line {}: 缺 instrument_id", n + 1))?;
            r.invest_unit_id = row.str("invest_unit_id").unwrap_or_default();
            r.long_margin_ratio_by_money = row.num("long_margin_ratio_by_money").unwrap_or(0.0);
            r.long_margin_ratio_by_volume = row.num("long_margin_ratio_by_volume").unwrap_or(0.0);
            r.short_margin_ratio_by_money = row.num("short_margin_ratio_by_money").unwrap_or(0.0);
            r.short_margin_ratio_by_volume = row.num("short_margin_ratio_by_volume").unwrap_or(0.0);
            rd.insert_margin_rate(r);
        }
        for (n, row) in read_jsonl(&format!("{dir}/commission_rates.jsonl"))?
            .iter()
            .enumerate()
        {
            let mut r = CommissionRate::default();
            r.broker_id = row.str("broker_id").unwrap_or_default();
            r.investor_id = row.str("investor_id").unwrap_or_default();
            r.exchange_id = row.str("exchange_id").unwrap_or_default();
            r.instrument_id = row.str("instrument_id").ok_or_else(|| {
                format!("commission_rates.jsonl line {}: 缺 instrument_id", n + 1)
            })?;
            r.invest_unit_id = row.str("invest_unit_id").unwrap_or_default();
            r.open_ratio_by_money = row.num("open_ratio_by_money").unwrap_or(0.0);
            r.open_ratio_by_volume = row.num("open_ratio_by_volume").unwrap_or(0.0);
            r.close_ratio_by_money = row.num("close_ratio_by_money").unwrap_or(0.0);
            r.close_ratio_by_volume = row.num("close_ratio_by_volume").unwrap_or(0.0);
            r.close_today_ratio_by_money = row.num("close_today_ratio_by_money").unwrap_or(0.0);
            r.close_today_ratio_by_volume = row.num("close_today_ratio_by_volume").unwrap_or(0.0);
            rd.insert_commission_rate(r);
        }
        for (n, row) in read_jsonl(&format!("{dir}/order_comm_rates.jsonl"))?
            .iter()
            .enumerate()
        {
            let mut r = OrderCommRate {
                hedge_flag: HEDGE_FLAG_SPECULATION,
                ..Default::default()
            };
            r.broker_id = row.str("broker_id").unwrap_or_default();
            r.investor_id = row.str("investor_id").unwrap_or_default();
            r.exchange_id = row.str("exchange_id").unwrap_or_default();
            r.instrument_id = row.str("instrument_id").ok_or_else(|| {
                format!("order_comm_rates.jsonl line {}: 缺 instrument_id", n + 1)
            })?;
            r.invest_unit_id = row.str("invest_unit_id").unwrap_or_default();
            r.order_comm_by_volume = row.num("order_comm_by_volume").unwrap_or(0.0);
            r.order_action_comm_by_volume = row.num("order_action_comm_by_volume").unwrap_or(0.0);
            r.order_comm_by_trade = row.num("order_comm_by_trade").unwrap_or(0.0);
            r.order_action_comm_by_trade = row.num("order_action_comm_by_trade").unwrap_or(0.0);
            rd.insert_order_comm_rate(r);
        }
        if let Some(row) = read_jsonl(&format!("{dir}/trading_params.jsonl"))?
            .into_iter()
            .next()
        {
            let mut p = TradingParams {
                broker_id: row.str("broker_id").unwrap_or_default(),
                investor_id: row.str("investor_id").unwrap_or_default(),
                account_id: row.str("account_id").unwrap_or_default(),
                ..Default::default()
            };
            if let Some(c) = row.str("currency_id") {
                p.currency_id = c;
            }
            if let Some(t) = row
                .str("margin_price_type")
                .and_then(|s| s.as_bytes().first().copied())
            {
                p.margin_price_type = t;
            }
            rd.set_params(p);
        }
        Ok(rd)
    }
}

// ---- shared contract-code parsing ------------------------------------------

fn product_of(instrument_id: &str) -> String {
    instrument_id
        .chars()
        .take_while(|c| !c.is_ascii_digit())
        .collect()
}

/// Parse the delivery month off a contract code.
///
/// * `YYMM` (SHFE / DCE / CFFEX / INE / GFEX) — `rb2610` -> (2026, 10).
/// * `YMM` (CZCE) — `TA609` -> (2026, 9): the leading digit is the decade, so
///   the year is `2020 + decade`.
pub fn parse_delivery_ym(instrument_id: &str) -> Option<(i32, i32)> {
    let body: Vec<char> = instrument_id.chars().collect();
    if body.len() >= 4
        && body[body.len() - 1].is_ascii_digit()
        && body[body.len() - 2].is_ascii_digit()
        && body[body.len() - 3].is_ascii_digit()
        && !body[body.len() - 4].is_ascii_digit()
    {
        let decade = body[body.len() - 3].to_digit(10)? as i32;
        let mm: i32 = format!("{}{}", body[body.len() - 2], body[body.len() - 1])
            .parse()
            .ok()?;
        if (1..=12).contains(&mm) {
            return Some((2020 + decade, mm));
        }
        return None;
    }
    if instrument_id.len() < 4 {
        return None;
    }
    let digits: String = instrument_id
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let yy: i32 = digits[0..2].parse().ok()?;
    let mm: i32 = digits[2..4].parse().ok()?;
    if !(1..=12).contains(&mm) {
        return None;
    }
    Some((2000 + yy, mm))
}

#[cfg(test)]
mod tests;
