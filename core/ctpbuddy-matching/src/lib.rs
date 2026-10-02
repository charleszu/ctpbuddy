//! Matching engine: domain types, instrument catalog, immediate-fill engine.
//!
//! The engine is a pure event-sourced function over (tick, order) streams:
//! it never touches IO and holds no clocks (the virtual clock is passed in via
//! [`ClockCtx`]). All state mutation happens inside the world loop
//! (DESIGN.md §8.1); the engine is single-threaded per process.

pub mod catalog;
pub mod engine;

pub use catalog::{Catalog, InstrumentInfo};
pub use engine::{
    CancelQuery, ClockCtx, EngineEvent, MatchingEngine, OrderRecord, SubmitOutcome, ERR_DIRECTION,
    ERR_DUPLICATE_ORDER, ERR_INSTRUMENT_NOT_FOUND, ERR_NO_CLOSE_TODAY, ERR_NO_COUNTERPARTY,
    ERR_NO_POSITION, ERR_ORDER_NOT_FOUND, ERR_ORDER_STATUS, ERR_PRICE_LIMIT, ERR_PRICE_TICK,
    ERR_VOLUME_RANGE,
};

use ctpbuddy_wire::generated::set_cstr;

/// Copy `s` into a fixed CTP char buffer (NUL-padded, truncated).
pub fn to_fixed<const N: usize>(s: &str) -> [u8; N] {
    let mut buf = [0u8; N];
    set_cstr(&mut buf, s);
    buf
}

/// CTP `THOST_FTDC_D_Buy` / `THOST_FTDC_D_Sell`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Buy,
    Sell,
}

impl Direction {
    pub fn as_ctp(self) -> u8 {
        match self {
            Direction::Buy => b'0',
            Direction::Sell => b'1',
        }
    }

    pub fn from_ctp(b: u8) -> Option<Self> {
        match b {
            b'0' | b'2' => Some(Direction::Buy), // '2' = ETF purchase; unused in M1
            b'1' | b'3' => Some(Direction::Sell),
            _ => None,
        }
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::Buy => Direction::Sell,
            Direction::Sell => Direction::Buy,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Direction::Buy => "买",
            Direction::Sell => "卖",
        }
    }
}

/// CTP `THOST_FTDC_OFEN_*` open/close flags.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OffsetFlag {
    Open,
    Close,
    CloseToday,
    CloseYesterday,
}

impl OffsetFlag {
    pub fn as_ctp(self) -> u8 {
        match self {
            OffsetFlag::Open => b'0',
            OffsetFlag::Close => b'1',
            OffsetFlag::CloseToday => b'3',
            OffsetFlag::CloseYesterday => b'4',
        }
    }

    pub fn from_ctp(b: u8) -> Option<Self> {
        match b {
            b'0' => Some(OffsetFlag::Open),
            b'1' => Some(OffsetFlag::Close),
            b'3' => Some(OffsetFlag::CloseToday),
            b'4' => Some(OffsetFlag::CloseYesterday),
            _ => None,
        }
    }

    pub fn is_close(self) -> bool {
        !matches!(self, OffsetFlag::Open)
    }

    pub fn label(self) -> &'static str {
        match self {
            OffsetFlag::Open => "开",
            OffsetFlag::Close => "平",
            OffsetFlag::CloseToday => "平今",
            OffsetFlag::CloseYesterday => "平昨",
        }
    }
}

/// Normalized order request extracted from `CThostFtdcInputOrderField`.
/// Built by the server (shim layer); the engine only sees this.
#[derive(Clone, Debug)]
pub struct OrderIntent {
    pub broker_id: [u8; 11],
    pub investor_id: [u8; 13],
    pub user_id: [u8; 16],
    /// Front-assigned order reference (`OrderRef`), unique per (front, session)
    /// while the order is active.
    pub order_ref: [u8; 13],
    /// Front-assigned local id (`OrderLocalID`), unique per front.
    pub order_local_id: [u8; 13],
    pub instrument_id: String,
    pub exchange_id: String,
    pub direction: Direction,
    pub offset: OffsetFlag,
    pub hedge_flag: u8,
    /// `OrderPriceType`: '1' any price, '2' limit (G/Q only M1 TODO).
    pub price_type: u8,
    pub limit_price: f64,
    pub volume: i32,
    /// `TimeCondition` (official CTP values, `ThostFtdcUserApiDataType.h`):
    /// '1' IOC, '3' GFD; GFS '2' is normalized to GFD, GTC '5' / GTD '4' are
    /// rejected at the server.
    pub time_condition: u8,
    /// `VolumeCondition` (official CTP values): '1' any volume (FAK / IOC),
    /// '2' minimum volume (FAK with `MinVolume`), '3' all volume (FOK).
    pub volume_condition: u8,
    pub min_volume: i32,
    pub contingent_condition: u8,
    pub stop_price: f64,
    pub force_close_reason: u8,
    pub request_id: i32,
    pub front_id: i32,
    pub session_id: i32,
}

impl OrderIntent {
    pub fn broker_id_s(&self) -> String {
        String::from_utf8_lossy(&self.broker_id)
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn investor_id_s(&self) -> String {
        String::from_utf8_lossy(&self.investor_id)
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn order_ref_s(&self) -> String {
        String::from_utf8_lossy(&self.order_ref)
            .trim_end_matches('\0')
            .to_string()
    }
}

/// A realized fill. Carries everything the ledger needs to settle volumes,
/// margin and commission — and the raw `TradeField` the client receives.
#[derive(Clone, Debug)]
pub struct Fill {
    pub broker_id: [u8; 11],
    pub investor_id: [u8; 13],
    pub user_id: [u8; 16],
    pub instrument_id: String,
    pub exchange_id: String,
    pub direction: Direction,
    pub offset: OffsetFlag,
    pub hedge_flag: u8,
    pub price: f64,
    pub volume: i32,
    pub volume_total_original: i32,
    pub order_sys_id: [u8; 21],
    pub order_ref: [u8; 13],
    pub trade_id: [u8; 21],
    /// Ledger freeze key: `"{front_id}/{session_id}/{order_ref}"` (assigned by
    /// the server at insert time; the engine mirrors it so cancel/fill paths
    /// agree without extra bookkeeping).
    pub order_key: String,
}
