//! Instrument catalog: contract directory + margin / commission rules.
//!
//! The catalog is a broker-level entity (DESIGN.md §6.4/§10). M1 keeps one
//! global table shared by the single default broker; per-broker override is a
//! TODO once multi-broker lands. Rates here are **illustrative defaults** —
//! a scenario's `instruments.csv` is the authoritative override.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use ctpbuddy_wire::generated::{set_cstr, CThostFtdcInstrumentField};

use crate::{Direction, OffsetFlag};

#[derive(Clone, Debug)]
pub struct InstrumentInfo {
    pub instrument_id: String,
    pub exchange_id: String,
    pub product_id: String,
    pub name: String,
    /// `THOST_FTDC_PC_*`: '1' future, '5' option, '3' spot...
    pub product_class: u8,
    pub is_trading: bool,
    pub volume_multiple: i32,
    pub price_tick: f64,
    pub long_margin_ratio: f64,
    pub short_margin_ratio: f64,
    /// Commission rate by turnover (`rate * price * volume * multiple`).
    pub commission_rate: f64,
    /// Per-lot fee floor in CNY (`max(turnover * rate, lots * floor)`).
    pub commission_per_lot: f64,
    pub min_limit_order_volume: i32,
    pub max_limit_order_volume: i32,
    pub min_market_order_volume: i32,
    pub max_market_order_volume: i32,
}

impl InstrumentInfo {
    pub fn margin_ratio(&self, direction: Direction) -> f64 {
        match direction {
            Direction::Buy => self.long_margin_ratio,
            Direction::Sell => self.short_margin_ratio,
        }
    }

    /// ByAmount margin for `volume` lots at `price` (DESIGN.md §8.6).
    pub fn margin(&self, direction: Direction, price: f64, volume: i32) -> f64 {
        price * volume as f64 * self.volume_multiple as f64 * self.margin_ratio(direction)
    }

    /// Fee for one fill at `price` / `volume` (M1 flat rate, no close-today
    /// differentiation — rule table TODO per DESIGN.md §8.3).
    pub fn commission(&self, price: f64, volume: i32) -> f64 {
        let turnover = price * volume as f64 * self.volume_multiple as f64;
        (turnover * self.commission_rate).max(self.commission_per_lot * volume as f64)
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

    /// `CThostFtdcInstrumentField` mirror for query responses. Delivery year /
    /// month are parsed from the contract code suffix when possible
    /// (`rb2610` -> 2026-10); dates are approximations for a synthetic desk.
    pub fn to_field(&self) -> CThostFtdcInstrumentField {
        let mut f = CThostFtdcInstrumentField::zeroed();
        set_cstr(&mut f.InstrumentID, &self.instrument_id);
        set_cstr(&mut f.ExchangeID, &self.exchange_id);
        set_cstr(&mut f.ExchangeInstID, &self.instrument_id);
        set_cstr(&mut f.ProductID, &self.product_id);
        set_cstr(&mut f.InstrumentName, &self.name);
        f.ProductClass = self.product_class;
        f.VolumeMultiple = self.volume_multiple;
        f.PriceTick = self.price_tick;
        f.MaxMarketOrderVolume = self.max_market_order_volume;
        f.MinMarketOrderVolume = self.min_market_order_volume;
        f.MaxLimitOrderVolume = self.max_limit_order_volume;
        f.MinLimitOrderVolume = self.min_limit_order_volume;
        f.InstLifePhase = b'1';
        f.IsTrading = if self.is_trading { 1 } else { 0 };
        f.PositionType = b'1';
        f.PositionDateType = b'2';
        f.LongMarginRatio = self.long_margin_ratio;
        f.ShortMarginRatio = self.short_margin_ratio;
        f.MaxMarginSideAlgorithm = b'0';
        f.OptionsType = b'0';
        f.UnderlyingMultiple = 1.0;
        f.CombinationType = b'0';
        if let Some((y, m)) = parse_delivery_ym(&self.instrument_id) {
            f.DeliveryYear = y;
            f.DeliveryMonth = m;
            set_cstr(&mut f.OpenDate, &format!("{:04}{:02}01", y, m));
            set_cstr(&mut f.CreateDate, &format!("{:04}{:02}01", y, m));
            set_cstr(&mut f.ExpireDate, &format!("{:04}{:02}15", y, m));
            set_cstr(&mut f.StartDelivDate, &format!("{:04}{:02}16", y, m));
            set_cstr(&mut f.EndDelivDate, &format!("{:04}{:02}15", y, m));
        }
        f
    }
}

/// Contract directory keyed by `InstrumentID`.
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    instruments: HashMap<String, InstrumentInfo>,
}

impl Catalog {
    pub fn new() -> Self {
        Catalog::default()
    }

    pub fn get(&self, instrument_id: &str) -> Option<&InstrumentInfo> {
        self.instruments.get(instrument_id)
    }

    pub fn insert(&mut self, info: InstrumentInfo) {
        self.instruments.insert(info.instrument_id.clone(), info);
    }

    pub fn len(&self) -> usize {
        self.instruments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instruments.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &InstrumentInfo> {
        self.instruments.values()
    }

    /// Load `instruments.csv` from a scenario directory (returns empty when the
    /// file is absent). Schema:
    /// `instrument,exchange,product,name,volume_multiple,price_tick,margin_ratio,
    ///  commission_rate,commission_per_lot,min_vol,max_vol`
    pub fn load_csv(path: &str) -> std::io::Result<Self> {
        let mut catalog = Catalog::new();
        let f = File::open(path)?;
        for (i, line) in BufReader::new(f).lines().enumerate() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if i == 0 && line.starts_with("instrument,") {
                continue;
            }
            let c: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if c.len() < 7 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("instruments.csv line {}: expected >=7 columns", i + 1),
                ));
            }
            let f_at = |idx: usize, default: f64| -> f64 {
                c.get(idx).filter(|s| !s.is_empty()).and_then(|s| s.parse().ok()).unwrap_or(default)
            };
            let i_at = |idx: usize, default: i32| -> i32 {
                c.get(idx).filter(|s| !s.is_empty()).and_then(|s| s.parse().ok()).unwrap_or(default)
            };
            let margin = f_at(6, 0.10);
            catalog.insert(InstrumentInfo {
                instrument_id: c[0].to_string(),
                exchange_id: c.get(1).copied().unwrap_or("").to_string(),
                product_id: c.get(2).copied().unwrap_or(c[0]).to_string(),
                name: c.get(3).copied().unwrap_or(c[0]).to_string(),
                product_class: b'1',
                is_trading: true,
                volume_multiple: i_at(4, 1),
                price_tick: f_at(5, 0.01),
                long_margin_ratio: margin,
                short_margin_ratio: margin,
                commission_rate: f_at(7, 0.0001),
                commission_per_lot: f_at(8, 1.0),
                min_limit_order_volume: i_at(9, 1),
                max_limit_order_volume: i_at(10, 500),
                min_market_order_volume: i_at(9, 1),
                max_market_order_volume: i_at(10, 500),
            });
        }
        Ok(catalog)
    }

    /// Builtin defaults (illustrative rates) so a bare `ctpbuddy up` without a
    /// scenario still has tradable contracts.
    pub fn builtin() -> Self {
        let mut c = Catalog::new();
        c.insert(builtin_contract("rb2610", "SHFE", "rb", "螺纹钢主力", 10, 1.0, 0.10, 0.000023, 1.0));
        c.insert(builtin_contract("au2612", "SHFE", "au", "沪金主力", 1000, 0.02, 0.09, 0.00001, 1.0));
        c.insert(builtin_contract("cu2610", "SHFE", "cu", "沪铜主力", 5, 10.0, 0.10, 0.00005, 1.0));
        c.insert(builtin_contract("m2609", "DCE", "m", "豆粕主力", 10, 1.0, 0.10, 0.00005, 1.0));
        c.insert(builtin_contract("IF2606", "CFFEX", "IF", "沪深300股指", 300, 0.2, 0.12, 0.0000234, 1.0));
        c
    }
}

/// Parse the trailing `YYMM` of a contract code (`rb2610` -> (2026, 10)).
fn parse_delivery_ym(instrument_id: &str) -> Option<(i32, i32)> {
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

fn builtin_contract(    instrument_id: &str,
    exchange_id: &str,
    product_id: &str,
    name: &str,
    multiple: i32,
    tick: f64,
    margin: f64,
    comm_rate: f64,
    comm_per_lot: f64,
) -> InstrumentInfo {
    InstrumentInfo {
        instrument_id: instrument_id.to_string(),
        exchange_id: exchange_id.to_string(),
        product_id: product_id.to_string(),
        name: name.to_string(),
        product_class: b'1',
        is_trading: true,
        volume_multiple: multiple,
        price_tick: tick,
        long_margin_ratio: margin,
        short_margin_ratio: margin,
        commission_rate: comm_rate,
        commission_per_lot: comm_per_lot,
        min_limit_order_volume: 1,
        max_limit_order_volume: 500,
        min_market_order_volume: 1,
        max_market_order_volume: 500,
    }
}

/// Error ids chosen to be recognizable against CTP's table; the authoritative
/// mapping is broker-configurable (DESIGN.md §8.3, M2 rule table).
/// Official prompts verbatim (error.xml): 30 OVER_CLOSE_POSITION,
/// 50 OVER_CLOSETODAY_POSITION, 51 OVER_CLOSEYESTERDAY_POSITION.
pub const ERR_CLOSE_TODAY_SHORT: i32 = 50; // OVER_CLOSETODAY_POSITION     CTP:平今仓位不足
pub const ERR_CLOSE_YD_SHORT: i32 = 51; //   OVER_CLOSEYESTERDAYPOSITION CTP:平昨仓位不足
pub const ERR_POSITION_CHECK: i32 = 30; //  OVER_CLOSE_POSITION         CTP:平仓量超过持仓量

/// Checked close-volume availability against a (today, yd) split.
///
/// `Close` prefers today positions then falls back to yd (rule table TODO);
/// `CloseToday` / `CloseYesterday` are strict.
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
