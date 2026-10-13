//! Rust-side reader for the validated offline TradingCalendar snapshot.
//!
//! Python remains the authoring and SHA256 authority. The core still reads the
//! snapshot so ADMIN settlement can derive the next TradingDay without
//! falling back to weekend arithmetic. Night sessions use the same explicit
//! rows and exchange mapping policy as the Python authoring side.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound;

use serde_json::Value;

use crate::CalendarMetadata;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarSnapshot {
    pub version: String,
    trading_days: BTreeMap<String, bool>,
    night_sessions: BTreeMap<(String, String), NightSession>,
    night_policy: Option<NightPolicy>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NightSession {
    pub status: String,
    pub action_day: Option<String>,
    pub trading_day: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NightPolicy {
    exchanges: BTreeSet<String>,
    closed_dates: BTreeSet<String>,
    mappings: BTreeMap<String, NightMapping>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NightMapping {
    status: String,
    action_day: Option<String>,
    trading_day: Option<String>,
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
        let mut night_sessions = BTreeMap::new();
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
            let date = row
                .get("date")
                .and_then(Value::as_str)
                .ok_or("交易日历 day.date 必须是字符串")?;
            let date = compact_date(date)?;
            if let Some(exchanges) = row.get("exchanges") {
                let exchanges = exchanges
                    .as_object()
                    .ok_or("交易日历 day.exchanges 必须是对象")?;
                for (exchange, night) in exchanges {
                    let night = night.as_object().ok_or("交易日历 night 必须是对象")?;
                    let status = night
                        .get("status")
                        .and_then(Value::as_str)
                        .unwrap_or("open");
                    if !matches!(status, "open" | "closed") {
                        return Err("交易日历 night.status 必须是 open/closed".into());
                    }
                    let action = night.get("night_action_day");
                    let trading = night.get("night_trading_day");
                    if status == "closed" {
                        if action.is_some_and(|value| !value.is_null())
                            || trading.is_some_and(|value| !value.is_null())
                        {
                            return Err("关闭夜盘不能设置 ActionDay/TradingDay".into());
                        }
                        night_sessions.insert(
                            (date.clone(), exchange.clone()),
                            NightSession {
                                status: status.to_string(),
                                action_day: None,
                                trading_day: None,
                            },
                        );
                    } else {
                        let action = action
                            .and_then(Value::as_str)
                            .ok_or("开放夜盘缺少 night_action_day")?;
                        let trading = trading
                            .and_then(Value::as_str)
                            .ok_or("开放夜盘缺少 night_trading_day")?;
                        let action = compact_date(action)?;
                        let trading = compact_date(trading)?;
                        night_sessions.insert(
                            (date.clone(), exchange.clone()),
                            NightSession {
                                status: status.to_string(),
                                action_day: Some(action),
                                trading_day: Some(trading),
                            },
                        );
                    }
                }
            }
        }
        let night_policy = parse_night_policy(object.get("night_policy"))?;
        Ok(Self {
            version,
            trading_days,
            night_sessions,
            night_policy,
        })
    }

    pub fn trading_day_for_date(&self, date: &str) -> Result<String, String> {
        let date = compact_date(date)?;
        match self.trading_days.get(&date) {
            Some(true) => Ok(date),
            Some(false) => Err(format!("交易日历明确标记 {date} 为休市")),
            None => Err(format!("交易日历未覆盖 {date}")),
        }
    }

    pub fn night_session(&self, date: &str, exchange: &str) -> Result<(String, String), String> {
        let date = compact_date(date)?;
        if let Some(session) = self
            .night_sessions
            .get(&(date.clone(), exchange.to_string()))
        {
            if session.status == "closed" {
                return Err(format!("交易日历明确标记 {date} 的 {exchange} 夜盘休市"));
            }
            return Ok((
                session.action_day.clone().expect("open night action day"),
                session.trading_day.clone().expect("open night trading day"),
            ));
        }
        let policy = self
            .night_policy
            .as_ref()
            .ok_or_else(|| format!("交易日历未覆盖 {date} 的 {exchange} 夜盘"))?;
        let mapping = policy
            .mappings
            .get(exchange)
            .ok_or_else(|| format!("交易日历未覆盖 {date} 的 {exchange} 夜盘映射"))?;
        if mapping.status == "not_applicable" {
            return Err(format!("{exchange} 没有夜盘"));
        }
        if !policy.exchanges.contains(exchange) {
            return Err(format!("交易日历未覆盖 {date} 的 {exchange} 夜盘"));
        }
        if !self.trading_days.get(&date).copied().unwrap_or(false) {
            return Err(format!("交易日历明确标记 {date} 为休市"));
        }
        if policy.closed_dates.contains(&date) {
            return Err(format!("交易日历明确标记 {date} 的 {exchange} 夜盘休市"));
        }
        let action = resolve_night_day(self, &date, mapping.action_day.as_deref())?;
        let trading = resolve_night_day(self, &date, mapping.trading_day.as_deref())?;
        Ok((action, trading))
    }

    pub fn night_status(&self, date: &str, exchange: &str) -> Result<&'static str, String> {
        let date = compact_date(date)?;
        if let Some(session) = self
            .night_sessions
            .get(&(date.clone(), exchange.to_string()))
        {
            return Ok(if session.status == "closed" {
                "closed"
            } else {
                "open"
            });
        }
        let policy = self
            .night_policy
            .as_ref()
            .ok_or_else(|| format!("交易日历未覆盖 {date} 的 {exchange} 夜盘"))?;
        let mapping = policy
            .mappings
            .get(exchange)
            .ok_or_else(|| format!("交易日历未覆盖夜盘映射: {exchange}"))?;
        if mapping.status == "not_applicable" {
            return Ok("not_applicable");
        }
        if !policy.exchanges.contains(exchange)
            || !self.trading_days.get(&date).copied().unwrap_or(false)
        {
            return Err(format!("交易日历未覆盖 {date} 的 {exchange} 夜盘"));
        }
        Ok(if policy.closed_dates.contains(&date) {
            "closed"
        } else {
            "open"
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

fn parse_night_policy(value: Option<&Value>) -> Result<Option<NightPolicy>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let object = value
        .as_object()
        .ok_or("交易日历 night_policy 必须是对象")?;
    if object.get("kind").and_then(Value::as_str) != Some("official-preholiday-closure") {
        return Err("交易日历 night_policy.kind 不受支持".into());
    }
    let scope = object.get("scope").and_then(Value::as_str).unwrap_or("");
    if !matches!(scope, "status-only" | "status-and-mapping") {
        return Err("交易日历 night_policy.scope 不受支持".into());
    }
    let exchanges = object
        .get("exchanges")
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty())
        .ok_or("交易日历 night_policy.exchanges 必须是非空数组")?;
    let exchanges = exchanges
        .iter()
        .map(|item| {
            item.as_str()
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
                .ok_or_else(|| "交易日历 night_policy.exchanges 必须是非空字符串数组".to_string())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let closed_dates = object
        .get("closed_dates")
        .and_then(Value::as_array)
        .ok_or("交易日历 night_policy.closed_dates 必须是数组")?
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| "交易日历 night_policy.closed_dates 必须是字符串数组".to_string())
                .and_then(compact_date)
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mappings_value = object.get("mappings");
    if scope == "status-and-mapping" && mappings_value.is_none() {
        return Err("交易日历 status-and-mapping 缺少 mappings".into());
    }
    if scope == "status-only" && mappings_value.is_some() {
        return Err("交易日历 status-only 不能提供 mappings".into());
    }
    let mut mappings = BTreeMap::new();
    if let Some(value) = mappings_value {
        let value = value
            .as_object()
            .ok_or("交易日历 night_policy.mappings 必须是对象")?;
        for (exchange, mapping) in value {
            let mapping = mapping
                .as_object()
                .ok_or("交易日历 night_policy.mapping 必须是对象")?;
            let status = mapping
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("open");
            if !matches!(status, "open" | "not_applicable") {
                return Err("交易日历 night_policy.mapping.status 不受支持".into());
            }
            let action_day = mapping
                .get("action_day")
                .and_then(Value::as_str)
                .map(str::to_string);
            let trading_day = mapping
                .get("trading_day")
                .and_then(Value::as_str)
                .map(str::to_string);
            if status == "not_applicable" {
                if action_day.is_some() || trading_day.is_some() {
                    return Err("交易日历 not_applicable 映射不能设置日期模式".into());
                }
            } else {
                if !matches!(
                    action_day.as_deref(),
                    Some("natural_date") | Some("next_trading_day")
                ) {
                    return Err("交易日历 mapping.action_day 不受支持".into());
                }
                if !matches!(
                    trading_day.as_deref(),
                    Some("same_trading_day") | Some("next_trading_day")
                ) {
                    return Err("交易日历 mapping.trading_day 不受支持".into());
                }
            }
            mappings.insert(
                exchange.clone(),
                NightMapping {
                    status: status.to_string(),
                    action_day,
                    trading_day,
                },
            );
        }
    }
    if scope == "status-and-mapping"
        && exchanges
            .iter()
            .any(|exchange| !mappings.contains_key(exchange))
    {
        return Err("交易日历 night_policy.mappings 缺少状态覆盖交易所".into());
    }
    Ok(Some(NightPolicy {
        exchanges,
        closed_dates,
        mappings,
    }))
}

fn resolve_night_day(
    calendar: &CalendarSnapshot,
    date: &str,
    mode: Option<&str>,
) -> Result<String, String> {
    match mode {
        Some("natural_date") | Some("same_trading_day") => Ok(date.to_string()),
        Some("next_trading_day") => calendar.next_trading_day(date),
        _ => Err("交易日历缺少有效的夜盘日期模式".into()),
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
        assert!(calendar.night_session("20261002", "SHFE").is_err());
    }

    #[test]
    fn validates_and_exposes_explicit_night_session_mapping() {
        let value = json!({
            "schema": "ctpbuddy.trading-calendar/v1",
            "version": "fixture-r1",
            "days": [{
                "date": "2026-10-02",
                "is_trading_day": true,
                "trading_day": "20261002",
                "exchanges": {
                    "SHFE": {
                        "status": "open",
                        "night_action_day": "2026-10-02",
                        "night_trading_day": "20261002"
                    }
                }
            }]
        });
        let calendar = CalendarSnapshot::from_value(&value).unwrap();
        assert_eq!(
            calendar.night_session("20261002", "SHFE").unwrap(),
            ("20261002".into(), "20261002".into())
        );
        let closed = json!({
            "schema": "ctpbuddy.trading-calendar/v1",
            "version": "fixture-r1",
            "days": [{
                "date": "2026-10-02",
                "is_trading_day": true,
                "trading_day": "20261002",
                "exchanges": {"SHFE": {"status": "closed"}}
            }]
        });
        assert!(CalendarSnapshot::from_value(&closed)
            .unwrap()
            .night_session("20261002", "SHFE")
            .is_err());
    }

    #[test]
    fn derives_exchange_specific_night_mapping_from_policy() {
        let value = json!({
            "schema": "ctpbuddy.trading-calendar/v1",
            "version": "fixture-r2",
            "night_policy": {
                "kind": "official-preholiday-closure",
                "source_revision": "fixture",
                "scope": "status-and-mapping",
                "exchanges": ["SHFE", "DCE", "CZCE"],
                "closed_dates": ["2026-10-04"],
                "mappings": {
                    "SHFE": {"action_day": "natural_date", "trading_day": "next_trading_day"},
                    "DCE": {"action_day": "next_trading_day", "trading_day": "next_trading_day"},
                    "CZCE": {"action_day": "natural_date", "trading_day": "same_trading_day"},
                    "CFFEX": {"status": "not_applicable"}
                }
            },
            "days": [
                {"date": "2026-10-02", "is_trading_day": true, "trading_day": "20261002"},
                {"date": "2026-10-03", "is_trading_day": false},
                {"date": "2026-10-04", "is_trading_day": false},
                {"date": "2026-10-05", "is_trading_day": true, "trading_day": "20261005"}
            ]
        });
        let calendar = CalendarSnapshot::from_value(&value).unwrap();
        assert_eq!(
            calendar.night_session("20261002", "SHFE").unwrap(),
            ("20261002".into(), "20261005".into())
        );
        assert_eq!(
            calendar.night_session("20261002", "DCE").unwrap(),
            ("20261005".into(), "20261005".into())
        );
        assert_eq!(
            calendar.night_session("20261002", "CZCE").unwrap(),
            ("20261002".into(), "20261002".into())
        );
        assert_eq!(
            calendar.night_status("20261002", "CFFEX").unwrap(),
            "not_applicable"
        );
        assert!(calendar.night_session("20261002", "CFFEX").is_err());
        assert!(calendar.night_session("20261004", "SHFE").is_err());
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
