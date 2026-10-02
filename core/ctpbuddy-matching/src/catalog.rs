//! Contract directory + reference data, and the fee/margin formulas.
//!
//! # Where the numbers come from
//!
//! **From the user, not from this file.** A trading desk already has
//! authoritative contract and rate data — the exchange's daily settlement
//! parameter file, a broker export, last week's query dump. CTPBuddy defines
//! the *shape* and reads whatever the user supplies (see [`crate::refdata`]
//! and the Python provider protocol `py/ctpbuddy/refdata`).
//!
//! [`Catalog::demo`] exists so a bare `ctpbuddy up` with no data at all still
//! has something tradable. Its rates are **made up** and it says so in every
//! doc comment that mentions it — do not read a number out of it and call it a
//! market fact. A scenario that cares supplies `instruments.jsonl` +
//! rate tables, and those take precedence.
//!
//! # Which rate applies
//!
//! notes/04 C3 is explicit: the counter charges the **公司费率** returned by
//! `ReqQryInstrumentMarginRate`, while `ReqQryInstrument`'s `LongMarginRatio`
//! is the *exchange* rate and is never used in a calculation. Both live in
//! [`RefData`], and [`crate::refdata::RefData::margin`] prefers the company
//! rate, falling back to the exchange one only so an unpriced desk still
//! behaves sanely.

use std::collections::HashMap;

pub use crate::refdata::{
    parse_delivery_ym, CommissionKind, CommissionRate, Instrument, MarginPrice, MarginRate,
    OrderCommRate, RefData, TradingParams, HEDGE_FLAG_SPECULATION, MPT_AVERAGE, MPT_OPEN_PRICE,
    MPT_PRE_SETTLEMENT, MPT_SETTLEMENT,
};

use crate::{Direction, OffsetFlag};

/// Contract directory + rate tables.
///
/// A thin name over [`RefData`] kept because "catalog" is the word the rest of
/// the core (and `DESIGN.md` §6.4) already uses for the contract directory.
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    inner: RefData,
}

impl Catalog {
    pub fn new() -> Self {
        Catalog {
            inner: RefData::new(),
        }
    }

    /// Build from a provider-supplied dataset.
    pub fn from_refdata(rd: RefData) -> Self {
        Catalog { inner: rd }
    }

    /// Load a canonical JSONL directory written by a ref-data provider
    /// (`ctpbuddy refdata export`, or any provider writing the same five
    /// files). See [`RefData::load_jsonl_dir`].
    pub fn load_refdata_dir(dir: &str) -> Result<Self, String> {
        RefData::load_jsonl_dir(dir).map(Catalog::from_refdata)
    }

    pub fn refdata(&self) -> &RefData {
        &self.inner
    }

    pub fn refdata_mut(&mut self) -> &mut RefData {
        &mut self.inner
    }

    // ---- accessors used across the core ----

    pub fn get(&self, instrument_id: &str) -> Option<&Instrument> {
        self.inner.instrument(instrument_id)
    }

    pub fn insert(&mut self, info: Instrument) {
        self.inner.insert_instrument(info);
    }

    pub fn len(&self) -> usize {
        self.inner.instrument_count()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.instrument_count() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &Instrument> {
        self.inner.instruments()
    }

    /// 公司保证金率 — the row `ReqQryInstrumentMarginRate` returns.
    pub fn margin_rate(&self, instrument_id: &str) -> Option<&MarginRate> {
        self.inner.margin_rate(instrument_id)
    }

    /// 手续费率 — the row `ReqQryInstrumentCommissionRate` returns.
    pub fn commission_rate(&self, instrument_id: &str) -> Option<&CommissionRate> {
        self.inner.commission_rate(instrument_id)
    }

    /// 报单/撤单费 — `ReqQryInstrumentOrderCommRate` (申报费).
    pub fn order_comm_rate(&self, instrument_id: &str) -> Option<&OrderCommRate> {
        self.inner.order_comm_rate(instrument_id)
    }

    // ---- rate-table writers (fixtures, tests, admin-driven refits) ----

    pub fn insert_margin_rate(&mut self, r: MarginRate) {
        self.inner.insert_margin_rate(r);
    }

    pub fn insert_commission_rate(&mut self, r: CommissionRate) {
        self.inner.insert_commission_rate(r);
    }

    pub fn insert_order_comm_rate(&mut self, r: OrderCommRate) {
        self.inner.insert_order_comm_rate(r);
    }

    pub fn set_trading_params(&mut self, p: TradingParams) {
        self.inner.set_params(p);
    }

    // ---- formulas (delegate; the logic lives in `refdata`) ----

    /// 保证金 for `volume` lots, honouring 公司费率 then 交易所费率.
    pub fn margin(
        &self,
        instrument_id: &str,
        direction: Direction,
        price: MarginPrice,
        volume: i32,
    ) -> f64 {
        self.inner.margin(instrument_id, direction, price, volume)
    }

    /// 手续费 for one fill leg (开仓 / 平昨 / 平今 priced separately).
    pub fn commission(
        &self,
        instrument_id: &str,
        kind: CommissionKind,
        price: f64,
        volume: i32,
    ) -> f64 {
        self.inner.commission(instrument_id, kind, price, volume)
    }

    /// Which price a fill's margin is computed against, per
    /// `MarginPriceType` (昨仓恒用昨结算价 — notes/04 C2).
    pub fn margin_price_for(
        &self,
        is_today: bool,
        pre_settlement: f64,
        fill_price: f64,
    ) -> MarginPrice {
        self.inner.margin_price_for(is_today, pre_settlement, fill_price)
    }

    /// Legacy single-rate estimate used when freezing funds at insert time,
    /// before the fill tells us which kind of leg it will be. Prices the whole
    /// order as if it were the more expensive of opening / closing today, so
    /// the freeze is never short and the release path stays symmetric.
    pub fn estimated_commission(&self, instrument_id: &str, price: f64, volume: i32) -> f64 {
        let open = self.inner.commission(instrument_id, CommissionKind::Open, price, volume);
        let close_today =
            self.inner.commission(instrument_id, CommissionKind::CloseToday, price, volume);
        open.max(close_today)
    }

    pub fn trading_params(&self) -> &TradingParams {
        self.inner.params()
    }

    // ---- fixtures ----

    /// Contracts + margin rates shipped with CTPBuddy, used when the operator
    /// supplies no `--refdata`.
    ///
    /// The contract rows are **real**: 789 futures across all six CTP groups
    /// (SHFE / INE / DCE / CZCE / CFFEX / GFEX), with genuine volume
    /// multiples, price ticks, delivery dates and 交易所保证金率. They are a
    /// snapshot, not a live feed — margin rates in particular move with
    /// exchange risk adjustments — so a desk testing anything rate-sensitive
    /// should pass its own `--refdata`.
    ///
    /// Commission is the exception: **no commission table ships**, because a
    /// plausible-looking invented fee is worse than none. Fee-free is a
    /// legitimate counter configuration and keeps assertions honest; a desk
    /// that needs fees supplies `commission_rates.jsonl`.
    ///
    /// Seven contracts used to be hardcoded here for the FAK-layout e2e; they
    /// are now real codes in this file (`m2_ioc.py` lists them), which is why
    /// the three exchange groups remain provable.
    pub fn bundled() -> Self {
        Catalog::load_refdata_dir(bundled_refdata_dir()).unwrap_or_else(|e| {
            eprintln!("[ctpbuddy] bundled ref data unusable ({e}); starting with no contracts");
            Catalog::new()
        })
    }
}

/// Filesystem path of the ref data shipped beside the executable.
///
/// The server passes `--refdata` explicitly in every supported path; this
/// exists so a bare `ctpbuddy up` — and the crate's own unit tests, whose
/// working directory is the crate root — still find the bundled snapshot.
/// Looked up next to the exe first, then by walking up from the current
/// directory (a checkout run, a test run, and `cargo run` all differ).
pub fn bundled_refdata_dir() -> &'static str {
    if let Ok(exe) = std::env::current_exe() {
        // <prefix>/bin/ctpbuddy-server -> <prefix>/refdata
        if let Some(prefix) = exe.parent().and_then(|p| p.parent()) {
            let cand = prefix.join("refdata");
            if cand.join("instruments.jsonl").exists() {
                return leak_path(cand);
            }
        }
        // target/debug/ctpbuddy-server -> core/target/debug -> repo root
        let mut up = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..5 {
            match up {
                Some(ref d) => {
                    let cand = d.join("refdata");
                    if cand.join("instruments.jsonl").exists() {
                        return leak_path(cand);
                    }
                    up = d.parent().map(|p| p.to_path_buf());
                }
                None => break,
            }
        }
    }
    // Walk up from the cwd: covers `cargo test` (crate root) and running the
    // binary from the repository root.
    let mut cur = std::env::current_dir().ok();
    for _ in 0..5 {
        match cur {
            Some(ref d) => {
                let cand = d.join("refdata");
                if cand.join("instruments.jsonl").exists() {
                    return leak_path(cand);
                }
                cur = d.parent().map(|p| p.to_path_buf());
            }
            None => break,
        }
    }
    "refdata"
}

fn leak_path(p: std::path::PathBuf) -> &'static str {
    Box::leak(p.to_string_lossy().into_owned().into_boxed_str())
}

/// Official CTP error.xml codes for close-volume rejections: ids and prompts
/// verbatim, because downstream clients branch on these numbers.
pub const ERR_CLOSE_TODAY_SHORT: i32 = 50; // OVER_CLOSETODAY_POSITION     CTP:平今仓位不足
pub const ERR_CLOSE_YD_SHORT: i32 = 51; //   OVER_CLOSEYESTERDAYPOSITION CTP:平昨仓位不足
pub const ERR_POSITION_CHECK: i32 = 30; //  OVER_CLOSE_POSITION         CTP:平仓量超过持仓量

/// Checked close-volume availability against a (today, yd) split.
///
/// `Close` prefers today positions then falls back to yd; `CloseToday` /
/// `CloseYesterday` are strict. Which leg a `Close` fill ends up pricing is
/// the same rule applied again at fill time in the ledger — keeping the two
/// in step is what makes 平今/平昨 commission observable.
pub fn close_volume_available(
    offset: OffsetFlag,
    today: i32,
    yd: i32,
    volume: i32,
) -> Result<(), (i32, String)> {
    match offset {
        OffsetFlag::Open => Ok(()),
        OffsetFlag::CloseToday => {
            if today < volume {
                Err((ERR_CLOSE_TODAY_SHORT, "CTP:平今仓位不足".into()))
            } else {
                Ok(())
            }
        }
        OffsetFlag::CloseYesterday => {
            if yd < volume {
                Err((ERR_CLOSE_YD_SHORT, "CTP:平昨仓位不足".into()))
            } else {
                Ok(())
            }
        }
        OffsetFlag::Close => {
            let total = today + yd;
            if total < volume {
                Err((ERR_POSITION_CHECK, "CTP:平仓量超过持仓量".into()))
            } else {
                Ok(())
            }
        }
    }
}

/// Per-(instrument, exchange) last-known prices, keyed for mark-to-market.
pub type PriceMap = HashMap<String, f64>;
