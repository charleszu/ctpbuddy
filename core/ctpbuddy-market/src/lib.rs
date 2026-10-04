//! Canonical tick representation, CSV source, and the virtual-clock playback.
//!
//! Tick sources (CSV / Parquet / user plugins) are decoupled from the counter:
//! everything downstream of a source consumes this canonical representation
//! (DESIGN.md §7). The virtual clock is the only time authority of the world.

pub mod transform;

use ctpbuddy_wire::generated::{set_cstr, CThostFtdcDepthMarketDataField};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

pub const DEPTH: usize = 5;

/// Milliseconds in a calendar day.
pub const DAY_MS: f64 = 86_400_000.0;
/// Clock times at or after this belong to the *night session* of the next
/// trading day (国内期货夜盘 21:00 起，归属下一交易日).
pub const NIGHT_START_MS: f64 = 18.0 * 3_600_000.0;

/// Map a wall-clock time of day (ms since midnight) onto the trading-day
/// timeline. The night session (from 18:00) is the *start* of the trading
/// day, so it maps to negative values: 21:00 → -3h, 00:30 → +0.5h,
/// 09:00 → +9h. The timeline is monotonic across midnight, which is what
/// pacing, seek, transforms and assertions all compare on.
pub fn session_ms(clock_ms: f64) -> f64 {
    if clock_ms >= NIGHT_START_MS {
        clock_ms - DAY_MS
    } else {
        clock_ms
    }
}

#[derive(Clone, Debug)]
pub struct Tick {
    pub instrument_id: String,
    pub exchange_id: String,
    pub trading_day: String,
    /// 实际自然日 (`ActionDay`). Empty = same as `trading_day`; a night-session
    /// tick should carry the calendar date it actually printed on.
    pub action_day: String,
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
            action_day: String::new(),
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
    /// Virtual timestamp on the trading-day timeline (see [`session_ms`]):
    /// the clock authority. Night-session ticks are negative.
    pub fn virtual_ms(&self) -> f64 {
        session_ms(parse_time_ms(&self.update_time) + self.update_millisec as f64)
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
        let action_day = if self.action_day.is_empty() {
            &self.trading_day
        } else {
            &self.action_day
        };
        set_cstr(&mut f.ActionDay, action_day);
        f
    }
}

/// Canonical CSV schema (header row required). See scenarios/sample_ticks.csv.
/// The first 20 columns mirror `CThostFtdcDepthMarketDataField` so a plain
/// CTP md recording (exported by the Python tooling) can be replayed as-is.
/// An optional 41st column `action_day` carries the night-session ActionDay.
pub const CSV_COLUMNS: &[&str] = &[
    "instrument",
    "exchange",
    "trading_day",
    "update_time",
    "update_millisec",
    "last_price",
    "volume",
    "turnover",
    "open_interest",
    "pre_settlement",
    "settlement",
    "pre_close",
    "open",
    "high",
    "low",
    "close",
    "upper",
    "lower",
    "pre_open_interest",
    "average",
    "bid1",
    "bid2",
    "bid3",
    "bid4",
    "bid5",
    "ask1",
    "ask2",
    "ask3",
    "ask4",
    "ask5",
    "bidvol1",
    "bidvol2",
    "bidvol3",
    "bidvol4",
    "bidvol5",
    "askvol1",
    "askvol2",
    "askvol3",
    "askvol4",
    "askvol5",
];

pub struct CsvSource;

impl CsvSource {
    /// Eager load; M1 sizes (a few million ticks) fit comfortably in memory.
    ///
    /// Lines are decoded as UTF-8 with lossy fallback (numeric columns are
    /// ASCII; GB18030 Chinese content would garble but never breaks parsing).
    /// An empty numeric cell reads as 0; a non-numeric one is an error naming
    /// the line and column — a garbled price must not become a silent 0.
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
            let num = |idx: usize| -> std::io::Result<f64> {
                let s = c[idx].trim();
                if s.is_empty() {
                    return Ok(0.0);
                }
                s.parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite())
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "csv line {}: column {} is not a finite number: {s:?}",
                                i + 1,
                                CSV_COLUMNS[idx]
                            ),
                        )
                    })
            };
            let int = |idx: usize| -> std::io::Result<i32> { Ok(num(idx)? as i32) };
            let mut t = Tick::default();
            t.instrument_id = c[0].trim().into();
            t.exchange_id = c[1].trim().into();
            t.trading_day = c[2].trim().into();
            t.action_day = c
                .get(CSV_COLUMNS.len())
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            t.update_time = c[3].trim().into();
            t.update_millisec = int(4)?;
            t.last_price = num(5)?;
            t.volume = int(6)?;
            t.turnover = num(7)?;
            t.open_interest = num(8)?;
            t.pre_settlement_price = num(9)?;
            t.settlement_price = num(10)?;
            t.pre_close_price = num(11)?;
            t.open_price = num(12)?;
            t.high_price = num(13)?;
            t.low_price = num(14)?;
            t.close_price = num(15)?;
            t.upper_limit_price = {
                let v = num(16)?;
                if v > 0.0 {
                    v
                } else {
                    f64::MAX
                }
            };
            t.lower_limit_price = num(17)?;
            t.pre_open_interest = num(18)?;
            t.average_price = num(19)?;
            for k in 0..DEPTH {
                t.bid_prices[k] = num(20 + k)?;
                t.ask_prices[k] = num(25 + k)?;
                t.bid_volumes[k] = int(30 + k)?;
                t.ask_volumes[k] = int(35 + k)?;
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

/// Format a trading-day timeline value (see [`session_ms`]) as the clock
/// time "HH:MM:SS" (CTP time field format); night-session values wrap back
/// to the evening clock.
pub fn format_hhmmss(ms: f64) -> String {
    let ms = if ms < 0.0 { ms + DAY_MS } else { ms };
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
    /// Wall-clock instant of the current pacing anchor. `None` whenever the
    /// anchor must be re-taken (fresh stream, pause/resume, seek, speed
    /// change, loop restart) so wall time spent outside the run is never
    /// counted as elapsed playback.
    started: Option<Instant>,
    /// Virtual time at the anchor: a tick is due once
    /// `tick.vt <= anchor_vt + elapsed_wall_ms * speed`.
    anchor_vt: f64,
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
            anchor_vt: 0.0,
            virtual_time: 0.0,
        }
    }

    /// Forget the pacing anchor; the next running poll re-takes it at the
    /// current stream position.
    fn reanchor(&mut self) {
        self.started = None;
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed;
        self.reanchor();
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn pause(&mut self) {
        self.paused = true;
        self.reanchor();
    }

    pub fn resume(&mut self) {
        self.paused = false;
        self.reanchor();
    }

    /// 日结后丢弃旧日未播放行情，保持暂停，避免旧行情污染新日账本。
    pub fn advance_trading_day(&mut self, next_day: &str) {
        self.ticks.retain(|t| t.trading_day.as_str() >= next_day);
        self.idx = 0;
        self.paused = true;
        self.step_once = false;
        self.looping = false;
        self.reanchor();
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
        self.reanchor();
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
            // loop restart: rewind the stream position; the pacing anchor
            // is re-taken below so the next pass paces normally
            self.idx = 0;
            self.virtual_time = self.ticks[0].virtual_ms();
            self.reanchor();
        }
        let mut due = Vec::new();
        if self.step_once {
            self.step_once = false;
            if self.idx < self.ticks.len() {
                due.push(self.ticks[self.idx].clone());
                self.idx += 1;
                self.advance_virtual_time();
            }
            // a manual step moves the position: pacing restarts from here
            self.reanchor();
            return due;
        }
        if self.paused || self.idx >= self.ticks.len() {
            return due;
        }
        if self.idx == 0 {
            // first tick sets the virtual origin
            self.virtual_time = self.ticks[0].virtual_ms();
        }
        let start = match self.started {
            Some(s) => s,
            None => {
                // Anchor at the current position: wall time spent paused,
                // before a seek or at another speed never counts.
                self.started = Some(now);
                self.anchor_vt = self.virtual_time;
                now
            }
        };
        let elapsed_ms = now.duration_since(start).as_secs_f64() * 1000.0;
        let horizon = if self.speed <= 0.0 {
            f64::MAX
        } else {
            self.anchor_vt + elapsed_ms * self.speed
        };
        while self.idx < self.ticks.len() && self.ticks[self.idx].virtual_ms() <= horizon {
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

#[cfg(test)]
mod tests {
    use super::*;
    use ctpbuddy_wire::generated::cstr;
    use std::time::Duration;

    /// Ticks every 500 virtual ms starting 09:30:00.
    fn stream(n: usize) -> Vec<Tick> {
        (0..n)
            .map(|i| {
                let mut t = Tick::default();
                t.instrument_id = "rb2610".into();
                t.update_time = "09:30:00".into();
                t.update_millisec = (i * 500) as i32;
                t
            })
            .collect()
    }

    fn released(pb: &mut Playback, at: Instant) -> usize {
        pb.poll(at).len()
    }

    #[test]
    fn real_time_pacing_does_not_accelerate() {
        let mut pb = Playback::new(stream(5), 1.0);
        let t0 = Instant::now();
        assert_eq!(released(&mut pb, t0), 1, "first tick at the origin");
        assert_eq!(released(&mut pb, t0 + Duration::from_millis(499)), 0);
        assert_eq!(released(&mut pb, t0 + Duration::from_millis(500)), 1);
        // the old bug: horizon was added to the *last released* vt, so this
        // poll 10ms later already released the 1000ms tick
        assert_eq!(released(&mut pb, t0 + Duration::from_millis(510)), 0);
        assert_eq!(released(&mut pb, t0 + Duration::from_millis(999)), 0);
        assert_eq!(released(&mut pb, t0 + Duration::from_millis(1000)), 1);
    }

    #[test]
    fn pause_time_is_not_counted_on_resume() {
        let mut pb = Playback::new(stream(5), 1.0);
        let t0 = Instant::now();
        assert_eq!(released(&mut pb, t0), 1);
        pb.pause();
        assert_eq!(released(&mut pb, t0 + Duration::from_secs(60)), 0);
        pb.resume();
        // a minute paused must not burst out the remaining ticks
        assert_eq!(released(&mut pb, t0 + Duration::from_secs(60)), 0);
        assert_eq!(
            released(
                &mut pb,
                t0 + Duration::from_secs(60) + Duration::from_millis(500)
            ),
            1
        );
    }

    #[test]
    fn seek_reanchors_pacing() {
        let mut pb = Playback::new(stream(10), 2.0);
        let t0 = Instant::now();
        assert_eq!(released(&mut pb, t0), 1);
        let later = t0 + Duration::from_secs(30);
        pb.seek(34_202_000.0); // tick #4 (09:30:02.000)
        assert_eq!(released(&mut pb, later), 1, "only the seek target is due");
        // speed 2: the next 500ms tick is due after 250ms of wall time
        assert_eq!(released(&mut pb, later + Duration::from_millis(249)), 0);
        assert_eq!(released(&mut pb, later + Duration::from_millis(250)), 1);
    }

    #[test]
    fn speed_zero_releases_everything() {
        let mut pb = Playback::new(stream(7), 0.0);
        assert_eq!(released(&mut pb, Instant::now()), 7);
        assert!(pb.finished());
    }

    #[test]
    fn night_session_timeline_is_monotonic_across_midnight() {
        let at = |hms: &str| {
            let mut t = Tick::default();
            t.update_time = hms.into();
            t
        };
        let night = ["21:00:00", "23:59:59", "00:00:00", "02:30:00"];
        let day = ["09:00:00", "11:30:00", "15:00:00"];
        let vts: Vec<f64> = night
            .iter()
            .chain(day.iter())
            .map(|s| at(s).virtual_ms())
            .collect();
        assert!(vts.windows(2).all(|w| w[0] < w[1]), "{vts:?}");
        assert_eq!(format_hhmmss(at("21:00:00").virtual_ms()), "21:00:00");
        assert_eq!(format_hhmmss(at("00:30:00").virtual_ms()), "00:30:00");
        // real-time pacing does not burst at midnight: 23:59:59.500 -> 00:00:00
        let mut a = at("23:59:59");
        a.update_millisec = 500;
        let mut pb = Playback::new(vec![a, at("00:00:00")], 1.0);
        let t0 = Instant::now();
        assert_eq!(released(&mut pb, t0), 1);
        assert_eq!(released(&mut pb, t0 + Duration::from_millis(499)), 0);
        assert_eq!(released(&mut pb, t0 + Duration::from_millis(500)), 1);
    }

    #[test]
    fn action_day_defaults_to_trading_day() {
        let mut t = Tick::default();
        t.trading_day = "20261009".into();
        assert_eq!(cstr(&t.to_depth_md().ActionDay), "20261009");
        t.action_day = "20261008".into();
        assert_eq!(cstr(&t.to_depth_md().ActionDay), "20261008");
    }
}
