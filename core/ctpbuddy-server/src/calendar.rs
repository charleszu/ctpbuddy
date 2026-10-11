//! Rust-side reader for the validated offline TradingCalendar snapshot.
//!
//! Python remains the authoring and SHA256 authority. The core still reads the
//! snapshot so ADMIN settlement can derive the next TradingDay without
//! falling back to weekend arithmetic.

use std::collections::BTreeMap;
use std::ops::Bound;

use serde_json::Value;

use crate::CalendarMetadata;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarSnapshot {
    pub version: String,
    trading_days: BTreeMap<String, bool>,
}

impl CalendarSnapshot {
    pub fn from_file(path: &str, expected: Option<&CalendarMetadata>) -> Result<Self, String> {
        let text =
            std::fs::read_to_string(path).map_err(|e| format!("交易日历读取失败 {path}: {e}"))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| format!("交易日历 JSON 无法解析 {path}: {e}"))?;
        let snapshot = Self::from_value(&value)?;
        if let Some(expected) = expected {
            if snapshot.version != expected.version {
                return Err("交易日历 version 与显式 provenance 不匹配".into());
            }
        }
        Ok(snapshot)
    }

    pub fn from_value(value: &Value) -> Result<Self, String> {
        let object = value.as_object().ok_or("交易日历必须是对象")?;
        if object.get("schema").and_then(Value::as_str) != Some("ctpbuddy.trading-calendar/v1") {
            return Err("不支持的交易日历 schema".into());
        }
        let version = object
            .get("version")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or("交易日历必须提供固定非空 version")?
            .to_string();
        let rows = object
            .get("days")
            .and_then(Value::as_array)
            .filter(|rows| !rows.is_empty())
            .ok_or("交易日历 days 必须是非空数组")?;
        let mut trading_days = BTreeMap::new();
        for row in rows {
            let row = row.as_object().ok_or("交易日历 day 必须是对象")?;
            let date = row
                .get("date")
                .and_then(Value::as_str)
                .ok_or("交易日历 day.date 必须是字符串")?;
            let compact = compact_date(date)?;
            let open = row
                .get("is_trading_day")
                .and_then(Value::as_bool)
                .ok_or("交易日历 day.is_trading_day 必须是布尔值")?;
            let day = row.get("trading_day");
            if open {
                let trading = day
                    .and_then(Value::as_str)
                    .ok_or("交易日历开放日缺少 trading_day")?;
                if trading != compact {
                    return Err("交易日历日盘 trading_day 必须与自然日一致".into());
                }
            } else if day.is_some_and(|v| !v.is_null()) {
                return Err("交易日历休市日不能设置 trading_day".into());
            }
            if trading_days.insert(compact, open).is_some() {
                return Err("交易日历含重复自然日".into());
            }
        }
        Ok(Self {
            version,
            trading_days,
        })
    }

    pub fn next_trading_day(&self, current: &str) -> Result<String, String> {
        let current = compact_date(current)?;
        for (day, open) in self
            .trading_days
            .range((Bound::Excluded(current.clone()), Bound::Unbounded))
        {
            if *open {
                return Ok(day.clone());
            }
        }
        Err(format!("交易日历没有 {} 之后的下一交易日", current))
    }
}

fn compact_date(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    if bytes.len() == 8 && bytes.iter().all(u8::is_ascii_digit) {
        let year: u32 = value[..4].parse().unwrap();
        let month: u32 = value[4..6].parse().unwrap();
        let day: u32 = value[6..].parse().unwrap();
        if valid_ymd(year, month, day) {
            return Ok(value.to_string());
        }
    }
    if bytes.len() == 10 && &value[4..5] == "-" && &value[7..8] == "-" {
        let compact = format!("{}{}{}", &value[..4], &value[5..7], &value[8..]);
        return compact_date(&compact);
    }
    Err(format!("非法日期: {value}"))
}

fn valid_ymd(year: u32, month: u32, day: u32) -> bool {
    if year < 1900 || !(1..=12).contains(&month) {
        return false;
    }
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let max = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=max).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_days_and_derives_next_trading_day() {
        let value = json!({
            "schema": "ctpbuddy.trading-calendar/v1",
            "version": "fixture-r1",
            "days": [
                {"date": "2026-10-02", "is_trading_day": true, "trading_day": "20261002"},
                {"date": "2026-10-03", "is_trading_day": false},
                {"date": "2026-10-05", "is_trading_day": true, "trading_day": "20261005"}
            ]
        });
        let calendar = CalendarSnapshot::from_value(&value).unwrap();
        assert_eq!(calendar.next_trading_day("20261002").unwrap(), "20261005");
    }

    #[test]
    fn rejects_invalid_or_duplicate_days() {
        let value = json!({
            "schema": "ctpbuddy.trading-calendar/v1",
            "version": "fixture-r1",
            "days": [
                {"date": "2026-02-30", "is_trading_day": false},
                {"date": "2026-02-30", "is_trading_day": false}
            ]
        });
        assert!(CalendarSnapshot::from_value(&value).is_err());
    }
}
