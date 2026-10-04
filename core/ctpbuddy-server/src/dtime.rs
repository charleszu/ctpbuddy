//! Wall-clock utilities without external crates: civil date math (UTC) and
//! virtual-time helpers. All wall timestamps in the journal are UTC, labeled
//! as such; the *trading day* is the virtual clock's domain (scenario ticks),
//! falling back to the China Standard Time date with the night-session rule
//! (>= 20:00 CST belongs to the next trading day).

use std::time::{SystemTime, UNIX_EPOCH};

/// `(YYYYMMDD, HH:MM:SS, ms_since_midnight, ISO8601)` — all UTC.
pub fn now_wall() -> (String, String, f64, String) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    let ms = now.subsec_millis() as f64;
    let (day, time) = civil_from_unix(secs);
    let iso = format!("{day}T{time}.{:03}+00:00", now.subsec_millis());
    let (d, t, m) = civil_parts(secs, ms);
    (d, t, m, iso)
}

fn civil_parts(secs: i64, ms: f64) -> (String, String, f64) {
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let (hh, mm, ss) = (sod / 3600, (sod % 3600) / 60, sod % 60);
    (
        format!("{y:04}{m:02}{d:02}"),
        format!("{hh:02}:{mm:02}:{ss:02}"),
        sod as f64 * 1000.0 + ms,
    )
}

fn civil_from_unix(secs: i64) -> (String, String) {
    let (d, t, _) = civil_parts(secs, 0.0);
    (d, t)
}

/// Days since 1970-01-01 -> (year, month, day). Howard Hinnant's algorithm.
pub fn civil_from_days(z0: i64) -> (i64, u32, u32) {
    let z = z0 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Trading day for a scenario-less run, from the China Standard Time (UTC+8)
/// clock: on or after 20:00 CST the night session already belongs to the
/// next trading day. Weekends and holidays are not modelled here — a run that
/// cares about them loads a scenario or an offline TradingCalendar.
pub fn today_trading_day() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let cst = now.as_secs() as i64 + 8 * 3600;
    let mut days = cst.div_euclid(86_400);
    if cst.rem_euclid(86_400) >= 20 * 3600 {
        days += 1;
    }
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}{m:02}{d:02}")
}

/// (year, month, day) -> days since 1970-01-01. Inverse of `civil_from_days`.
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = if m > 2 { m - 3 } else { m + 9 } as i64; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d as i64 - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}
