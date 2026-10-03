//! Canonical tick representation, CSV source, and the virtual-clock playback.
//!
//! Tick sources (CSV / Parquet / user plugins) are decoupled from the counter:
//! everything downstream of a source consumes this canonical representation
//! (DESIGN.md §7). The virtual clock is the only time authority of the world.

pub mod transform;

use ctpbuddy_wire::generated::{cstr, set_cstr, CThostFtdcDepthMarketDataField};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

pub const DEPTH: usize = 5;

#[derive(Clone, Debug)]
pub struct Tick {
    pub instrument_id: String,
    pub exchange_id: String,
    pub trading_day: String,
    pub update_time: String,
    pub update_millisec: i32,
    pub last_price: f64,
    pub volume: i32,
    /// 0 when the source does not record turnover (derived fields follow).
    pub turnover: f64,
    pub open_interest: f64,
    pub pre_open_interest: f64,
    pub pre_settlement_price: f64,
    pub pre_close_price: f64,
    pub settlement_price: f64,
    pub open_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub close_price: f64,
    pub average_price: f64,
    pub upper_limit_price: f64,
    pub lower_limit_price: f64,
    pub bid_prices: [f64; DEPTH],
    pub ask_prices: [f64; DEPTH],
    pub bid_volumes: [i32; DEPTH],
    pub ask_volumes: [i32; DEPTH],
}

impl Default for Tick {
    fn default() -> Self {
        Tick {
            instrument_id: String::new(),
            exchange_id: String::new(),
            trading_day: String::new(),
            update_time: String::new(),
            update_millisec: 0,
            last_price: 0.0,
            volume: 0,
            turnover: 0.0,
            open_interest: 0.0,
            pre_open_interest: 0.0,
            pre_settlement_price: 0.0,
            pre_close_price: 0.0,
            settlement_price: 0.0,
            open_price: 0.0,
            high_price: 0.0,
            low_price: 0.0,
            close_price: 0.0,
            average_price: 0.0,
            upper_limit_price: f64::MAX,
            lower_limit_price: 0.0,
            bid_prices: [0.0; DEPTH],
            ask_prices: [0.0; DEPTH],
            bid_volumes: [0; DEPTH],
            ask_volumes: [0; DEPTH],
        }
    }
}

impl Tick {
    /// Virtual timestamp in milliseconds since midnight (the clock authority).
    pub fn virtual_ms(&self) -> f64 {
        parse_time_ms(&self.update_time) + self.update_millisec as f64
    }

    /// Best bid/ask, falling back to the last price when depth is absent.
    pub fn best_bid(&self) -> f64 {
        if self.bid_prices[0] > 0.0 {
            self.bid_prices[0]
        } else {
            self.last_price
        }
    }

    pub fn best_ask(&self) -> f64 {
        if self.ask_prices[0] > 0.0 {
            self.ask_prices[0]
        } else {
            self.last_price
        }
    }

    /// Convert into the wire form of `CThostFtdcDepthMarketDataField`
    /// (RTN_DEPTH_MD payload). Pure function over the tick.
    pub fn to_depth_md(&self) -> CThostFtdcDepthMarketDataField {
        let mut f = CThostFtdcDepthMarketDataField::zeroed();
        set_cstr(&mut f.TradingDay, &self.trading_day);
        set_cstr(&mut f.ExchangeID, &self.exchange_id);
        set_cstr(&mut f.InstrumentID, &self.instrument_id);
        f.LastPrice = self.last_price;
        f.PreSettlementPrice = self.pre_settlement_price;
        f.PreClosePrice = self.pre_close_price;
        f.PreOpenInterest = self.pre_open_interest;
        f.OpenPrice = self.open_price;
        f.HighestPrice = self.high_price;
        f.LowestPrice = self.low_price;
        f.Volume = self.volume;
        f.Turnover = self.turnover;
        f.OpenInterest = self.open_interest;
        f.ClosePrice = self.close_price;
        f.SettlementPrice = self.settlement_price;
        f.UpperLimitPrice = self.upper_limit_price;
        f.LowerLimitPrice = self.lower_limit_price;
        set_cstr(&mut f.UpdateTime, &self.update_time);
        f.UpdateMillisec = self.update_millisec;
        f.BidPrice1 = self.bid_prices[0];
        f.BidPrice2 = self.bid_prices[1];
        f.BidPrice3 = self.bid_prices[2];
        f.BidPrice4 = self.bid_prices[3];
        f.BidPrice5 = self.bid_prices[4];
        f.AskPrice1 = self.ask_prices[0];
        f.AskPrice2 = self.ask_prices[1];
        f.AskPrice3 = self.ask_prices[2];
        f.AskPrice4 = self.ask_prices[3];
        f.AskPrice5 = self.ask_prices[4];
        f.BidVolume1 = self.bid_volumes[0];
        f.BidVolume2 = self.bid_volumes[1];
        f.BidVolume3 = self.bid_volumes[2];
        f.BidVolume4 = self.bid_volumes[3];
        f.BidVolume5 = self.bid_volumes[4];
        f.AskVolume1 = self.ask_volumes[0];
        f.AskVolume2 = self.ask_volumes[1];
        f.AskVolume3 = self.ask_volumes[2];
        f.AskVolume4 = self.ask_volumes[3];
        f.AskVolume5 = self.ask_volumes[4];
        f.AveragePrice = self.average_price;
        set_cstr(&mut f.ActionDay, &self.trading_day);
        f
    }

    pub fn instrument_id_c(&self) -> String {
        cstr(&self.to_depth_md().InstrumentID)
    }
}

/// Canonical CSV schema (header row required). See scenarios/sample_ticks.csv.
/// The first 20 columns mirror `CThostFtdcDepthMarketDataField` so a plain
/// CTP md recording (exported by the Python tooling) can be replayed as-is.
pub const CSV_COLUMNS: &[&str] = &[
    "instrument", "exchange", "trading_day", "update_time", "update_millisec", "last_price",
    "volume", "turnover", "open_interest", "pre_settlement", "settlement", "pre_close", "open",
    "high", "low", "close", "upper", "lower", "pre_open_interest", "average", "bid1", "bid2",
    "bid3", "bid4", "bid5", "ask1", "ask2", "ask3", "ask4", "ask5", "bidvol1", "bidvol2", "bidvol3",
    "bidvol4", "bidvol5", "askvol1", "askvol2", "askvol3", "askvol4", "askvol5",
];

pub struct CsvSource;

impl CsvSource {
    /// Eager load; M1 sizes (a few million ticks) fit comfortably in memory.
    ///
    /// Lines are decoded as UTF-8 with lossy fallback (numeric columns are
    /// ASCII; GB18030 Chinese content would garble but never breaks parsing).
    pub fn load(path: &str) -> std::io::Result<Vec<Tick>> {
        let f = File::open(path)?;
        let mut out = Vec::new();
        for (i, line) in BufReader::new(f).lines().enumerate() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if i == 0 && line.starts_with("instrument,") {
                continue;
            }
            if line.starts_with('#') {
                continue;
            }
            let c: Vec<&str> = line.split(',').collect();
            if c.len() < CSV_COLUMNS.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "csv line {}: expected {} columns, got {}",
                        i + 1,
                        CSV_COLUMNS.len(),
                        c.len()
                    ),
                ));
            }
            let num = |idx: usize| -> f64 {
                let s = c[idx].trim();
                if s.is_empty() {
                    0.0
                } else {
                    s.parse().unwrap_or(0.0)
                }
            };
            let int = |idx: usize| -> i32 { num(idx) as i32 };
            let mut t = Tick::default();
            t.instrument_id = c[0].trim().into();
            t.exchange_id = c[1].trim().into();
            t.trading_day = c[2].trim().into();
            t.update_time = c[3].trim().into();
            t.update_millisec = int(4);
            t.last_price = num(5);
            t.volume = int(6);
            t.turnover = num(7);
            t.open_interest = num(8);
            t.pre_settlement_price = num(9);
            t.settlement_price = num(10);
            t.pre_close_price = num(11);
            t.open_price = num(12);
            t.high_price = num(13);
            t.low_price = num(14);
            t.close_price = num(15);
            t.upper_limit_price = {
                let v = num(16);
                if v > 0.0 {
                    v
                } else {
                    f64::MAX
                }
            };
            t.lower_limit_price = num(17);
            t.pre_open_interest = num(18);
            t.average_price = num(19);
            for k in 0..DEPTH {
                t.bid_prices[k] = num(20 + k);
                t.ask_prices[k] = num(25 + k);
                t.bid_volumes[k] = int(30 + k);
                t.ask_volumes[k] = int(35 + k);
            }
            out.push(t);
        }
        Ok(out)
    }
}

fn parse_time_ms(s: &str) -> f64 {
    let p: Vec<&str> = s.split(':').collect();
    if p.len() != 3 {
        return 0.0;
    }
    let h: f64 = p[0].parse().unwrap_or(0.0);
    let m: f64 = p[1].parse().unwrap_or(0.0);
    let sec: f64 = p[2].parse().unwrap_or(0.0);
    (h * 3600.0 + m * 60.0 + sec) * 1000.0
}

/// Format virtual ms since midnight as "HH:MM:SS" (CTP time field format).
pub fn format_hhmmss(ms: f64) -> String {
    let total = ms.max(0.0) as i64;
    let h = total / 3_600_000;
    let m = (total % 3_600_000) / 60_000;
    let s = (total % 60_000) / 1000;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// Virtual-clock playback state. Owned exclusively by the world loop (DESIGN.md §7.5):
/// pause / step / speed are mutated only from admin events, never from IO threads.
#[derive(Clone)]
pub struct Playback {
    ticks: Vec<Tick>,
    idx: usize,
    paused: bool,
    step_once: bool,
    /// 0 = as fast as possible; otherwise wall-clock ms per virtual ms (1/speed).
    speed: f64,
    /// Loop the whole stream when it runs out (engine/ledger state is NOT
    /// reset — use reset_account / a scenario reload for that).
    looping: bool,
    started: Option<Instant>,
    virtual_time: f64,
}

impl Playback {
    pub fn new(ticks: Vec<Tick>, speed: f64) -> Self {
        Playback {
            ticks,
            idx: 0,
            paused: false,
            step_once: false,
            speed,
            looping: false,
            started: None,
            virtual_time: 0.0,
        }
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed;
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// 日结后丢弃旧日未播放行情，保持暂停，避免旧行情污染新日账本。
    pub fn advance_trading_day(&mut self, next_day: &str) {
        self.ticks.retain(|t| t.trading_day.as_str() >= next_day);
        self.idx = 0;
        self.paused = true;
        self.step_once = false;
        self.looping = false;
        self.started = None;
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn step(&mut self) {
        self.step_once = true;
    }

    /// Reposition to the first tick at/after `target_ms` (virtual ms since
    /// midnight). Skipped ticks are never delivered; the wall-clock baseline
    /// is re-anchored so pacing continues smoothly from the new position.
    pub fn seek(&mut self, target_ms: f64) {
        let mut i = 0;
        while i < self.ticks.len() && self.ticks[i].virtual_ms() < target_ms {
            i += 1;
        }
        self.idx = i;
        self.virtual_time = if i < self.ticks.len() {
            self.ticks[i].virtual_ms()
        } else {
            target_ms
        };
        self.step_once = false;
    }

    pub fn set_loop(&mut self, on: bool) {
        self.looping = on;
    }

    pub fn looping(&self) -> bool {
        self.looping
    }

    pub fn finished(&self) -> bool {
        self.idx >= self.ticks.len()
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.idx, self.ticks.len())
    }

    pub fn virtual_time(&self) -> f64 {
        self.virtual_time
    }

    /// Release every tick that is due at `now`. Deterministic release order:
    /// ticks are returned in file order, never reordered.
    pub fn poll(&mut self, now: Instant) -> Vec<Tick> {
        if self.finished() {
            if !self.looping || self.ticks.is_empty() {
                return Vec::new();
            }
            // loop restart: re-anchor both the stream position and the
            // wall-clock baseline so the next pass paces normally
            self.idx = 0;
            self.virtual_time = self.ticks[0].virtual_ms();
            self.started = Some(now);
        }
        let start = *self.started.get_or_insert(now);
        let mut due = Vec::new();
        if self.step_once {
            self.step_once = false;
            if self.idx < self.ticks.len() {
                due.push(self.ticks[self.idx].clone());
                self.idx += 1;
                self.advance_virtual_time();
            }
            return due;
        }
        if self.paused {
            return due;
        }
        let elapsed_ms = now.duration_since(start).as_secs_f64() * 1000.0;
        let horizon = if self.speed <= 0.0 {
            f64::MAX
        } else {
            elapsed_ms * self.speed
        };
        // first tick sets the virtual origin
        if self.idx == 0 {
            self.virtual_time = self.ticks[0].virtual_ms();
        }
        while self.idx < self.ticks.len() && self.ticks[self.idx].virtual_ms() <= self.virtual_time + horizon {
            due.push(self.ticks[self.idx].clone());
            self.idx += 1;
            self.advance_virtual_time();
        }
        due
    }

    fn advance_virtual_time(&mut self) {
        if let Some(t) = self.ticks.get(self.idx.saturating_sub(1)) {
            self.virtual_time = t.virtual_ms();
        }
    }
}
