//! Scenario DSL spec (DESIGN.md §7.4): the JSON form the control plane sends.
//!
//! YAML is the authoring format; the Python control plane parses and validates
//! it and normalizes to this shape — times already resolved to virtual ms
//! since midnight, one assertion per `expect` entry. The core stays a pure
//! consumer: no YAML parser, no locale, no wall clock in the pipeline.

use ctpbuddy_market::transform::Transform;

fn valid_date(s: &str) -> bool {
    if s.len() != 8 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    let y: i32 = s[0..4].parse().unwrap_or(0);
    let m: u32 = s[4..6].parse().unwrap_or(0);
    let d: u32 = s[6..8].parse().unwrap_or(0);
    if y < 1900 || !(1..=12).contains(&m) || d == 0 {
        return false;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let max = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    d <= max
}

use crate::json::Value;

/// Comparison operators for scenario assertions (`expect: { metric: ">=1" }`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmp {
    Ge,
    Le,
    Gt,
    Lt,
    Eq,
    Ne,
}

impl Cmp {
    pub fn parse(s: &str) -> Option<Cmp> {
        match s.trim() {
            ">=" => Some(Cmp::Ge),
            "<=" => Some(Cmp::Le),
            ">" => Some(Cmp::Gt),
            "<" => Some(Cmp::Lt),
            "==" | "=" => Some(Cmp::Eq),
            "!=" => Some(Cmp::Ne),
            _ => None,
        }
    }

    pub fn apply(self, actual: f64, value: f64) -> bool {
        match self {
            Cmp::Ge => actual >= value,
            Cmp::Le => actual <= value,
            Cmp::Gt => actual > value,
            Cmp::Lt => actual < value,
            Cmp::Eq => (actual - value).abs() < 1e-9,
            Cmp::Ne => (actual - value).abs() >= 1e-9,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Cmp::Ge => ">=",
            Cmp::Le => "<=",
            Cmp::Gt => ">",
            Cmp::Lt => "<",
            Cmp::Eq => "==",
            Cmp::Ne => "!=",
        }
    }
}

/// One one-shot assertion, evaluated by the world loop when the virtual clock
/// passes `t0 + after_ms` (t0 = the scenario's first delivered tick).
#[derive(Clone, Debug)]
pub struct Assertion {
    pub after_ms: f64,
    pub investor: String,
    pub metric: String,
    pub op: Cmp,
    pub value: f64,
    pub evaluated: bool,
    pub pass: bool,
    pub actual: f64,
}

/// A normalized scenario spec (the JSON form of scenario.yaml).
#[derive(Clone, Debug)]
pub struct BootstrapPosition {
    pub instrument: String,
    pub exchange: String,
    pub direction: String,
    pub open_date: String,
    pub trade_id: String,
    pub open_price: f64,
    pub volume: i32,
    pub pre_settlement: f64,
    pub margin: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct AccountSpec {
    pub investor: String,
    pub balance: Option<f64>,
    pub positions: Vec<BootstrapPosition>,
}

#[derive(Clone, Debug, Default)]
pub struct Spec {
    pub name: String,
    /// `source.path` relative to the scenario dir (default: ticks.csv).
    pub source_path: Option<String>,
    pub transforms: Vec<Transform>,
    /// `clock.time_scale` (playback speed multiplier; 0 = as fast as possible).
    pub time_scale: Option<f64>,
    /// `clock.start` — ticks before this virtual ms are dropped at load.
    pub start_ms: Option<f64>,
    /// Scenario-authored accounts and optional bootstrap positions.
    pub accounts: Vec<AccountSpec>,
    pub assertions: Vec<Assertion>,
}

fn get_num(v: &Value, key: &str) -> Option<f64> {
    v.get_num(key)
}

/// Parse the normalized spec JSON. Errors carry the offending field so the
/// control plane can surface them verbatim.
pub fn parse_spec(v: &Value) -> Result<Spec, String> {
    let mut spec = Spec::default();
    spec.name = v.get_str("name").unwrap_or_default();
    if let Some(src) = v.get("source") {
        if let Some(p) = src.get_str("path") {
            if !p.trim().is_empty() {
                spec.source_path = Some(p.trim().to_string());
            }
        }
        let kind = src.get_str("kind").unwrap_or_else(|| "csv".to_string());
        if kind != "csv" {
            return Err(format!("source kind '{kind}' 暂不支持（M2 仅 csv）"));
        }
    }
    if let Some(Value::Arr(items)) = v.get("transforms") {
        for (i, item) in items.iter().enumerate() {
            let kind = item
                .get_str("kind")
                .ok_or_else(|| format!("transforms[{i}]: 缺少 kind"))?;
            let t = match kind.as_str() {
                "freeze" => Transform::Freeze {
                    at_ms: get_num(item, "at_ms")
                        .ok_or_else(|| format!("transforms[{i}](freeze): 缺少 at_ms"))?,
                    duration_ms: get_num(item, "duration_ms")
                        .ok_or_else(|| format!("transforms[{i}](freeze): 缺少 duration_ms"))?,
                },
                "gap" => Transform::Gap {
                    at_ms: get_num(item, "at_ms")
                        .ok_or_else(|| format!("transforms[{i}](gap): 缺少 at_ms"))?,
                    shift: get_num(item, "shift")
                        .ok_or_else(|| format!("transforms[{i}](gap): 缺少 shift"))?,
                },
                "liquidity" => Transform::Liquidity {
                    from_ms: get_num(item, "from_ms")
                        .ok_or_else(|| format!("transforms[{i}](liquidity): 缺少 from_ms"))?,
                    scale: get_num(item, "scale")
                        .ok_or_else(|| format!("transforms[{i}](liquidity): 缺少 scale"))?,
                },
                other => return Err(format!("transforms[{i}]: 未知 kind '{other}'")),
            };
            spec.transforms.push(t);
        }
    }
    if let Some(clock) = v.get("clock") {
        spec.time_scale = get_num(clock, "time_scale");
        spec.start_ms = get_num(clock, "start_ms");
        if let Some(ts) = spec.time_scale {
            if ts < 0.0 {
                return Err("clock.time_scale 不能为负".to_string());
            }
        }
    }
    if let Some(raw) = v.get("accounts") {
        let items = match raw {
            Value::Arr(items) => items,
            _ => return Err("accounts 必须是列表".into()),
        };
        for (i, item) in items.iter().enumerate() {
            let investor = item
                .get_str("investor")
                .ok_or_else(|| format!("accounts[{i}]: 缺少 investor"))?;
            if investor.trim().is_empty() || investor.len() > 12 || investor.contains('\0') {
                return Err(format!("accounts[{i}]: investor 不能为空"));
            }
            let balance = get_num(item, "balance");
            if let Some(b) = balance {
                if !b.is_finite() || b <= 0.0 {
                    return Err(format!("accounts[{i}]: balance 必须为有限正数"));
                }
            }
            let mut positions = Vec::new();
            let mut keys = std::collections::HashSet::new();
            if let Some(raw_positions) = item.get("positions") {
                let items = match raw_positions {
                    Value::Arr(items) => items,
                    _ => return Err(format!("accounts[{i}].positions 必须是列表")),
                };
                for (j, p) in items.iter().enumerate() {
                    let field = |k: &str| {
                        p.get_str(k)
                            .ok_or_else(|| format!("accounts[{i}].positions[{j}]: 缺少 {k}"))
                    };
                    let instrument = field("instrument")?;
                    let exchange = field("exchange")?;
                    let direction = field("direction")?;
                    let open_date = field("open_date")?;
                    let trade_id = field("trade_id")?;
                    let open_price = p
                        .get("open_price")
                        .and_then(|v| v.as_num())
                        .ok_or_else(|| format!("accounts[{i}].positions[{j}].open_price 非数字"))?;
                    let volume = p
                        .get("volume")
                        .and_then(|v| v.as_num())
                        .ok_or_else(|| format!("accounts[{i}].positions[{j}].volume 非数字"))?;
                    let pre_settlement = p
                        .get("pre_settlement")
                        .and_then(|v| v.as_num())
                        .ok_or_else(|| {
                            format!("accounts[{i}].positions[{j}].pre_settlement 非数字")
                        })?;
                    if !matches!(direction.as_str(), "long" | "short")
                        || !valid_date(&open_date)
                        || instrument.is_empty()
                        || exchange.is_empty()
                        || trade_id.is_empty()
                        || instrument.len() > 80
                        || exchange.len() > 8
                        || trade_id.len() > 20
                        || [&instrument, &exchange, &trade_id]
                            .iter()
                            .any(|s| s.contains('\0'))
                        || open_price <= 0.0
                        || pre_settlement <= 0.0
                        || !open_price.is_finite()
                        || !pre_settlement.is_finite()
                        || !volume.is_finite()
                        || volume <= 0.0
                        || volume > i32::MAX as f64
                        || volume.fract() != 0.0
                    {
                        return Err(format!("accounts[{i}].positions[{j}] 字段非法"));
                    }
                    let margin = match p.get("margin") {
                        None => None,
                        Some(Value::Num(m)) if m.is_finite() && *m >= 0.0 => Some(*m),
                        _ => {
                            return Err(format!(
                                "accounts[{i}].positions[{j}].margin 必须是有限非负数字"
                            ))
                        }
                    };
                    let key = format!("{instrument}|{exchange}|{direction}|{open_date}|{trade_id}");
                    if !keys.insert(key) {
                        return Err(format!("accounts[{i}].positions[{j}] 重复逐笔 key"));
                    }
                    positions.push(BootstrapPosition {
                        instrument,
                        exchange,
                        direction,
                        open_date,
                        trade_id,
                        open_price,
                        volume: volume as i32,
                        pre_settlement,
                        margin,
                    });
                }
            }
            let total: i64 = positions.iter().map(|p| p.volume as i64).sum();
            if total > i32::MAX as i64 {
                return Err(format!("accounts[{i}] positions 聚合 volume 超过 i32"));
            }
            spec.accounts.push(AccountSpec {
                investor,
                balance,
                positions,
            });
        }
    }
    if let Some(Value::Arr(items)) = v.get("assertions") {
        for (i, item) in items.iter().enumerate() {
            let after_ms = get_num(item, "after_ms")
                .ok_or_else(|| format!("assertions[{i}]: 缺少 after_ms"))?;
            let investor = item
                .get_str("investor")
                .ok_or_else(|| format!("assertions[{i}]: 缺少 investor"))?;
            let metric = item
                .get_str("metric")
                .ok_or_else(|| format!("assertions[{i}]: 缺少 metric"))?;
            let op_s = item
                .get_str("op")
                .ok_or_else(|| format!("assertions[{i}]: 缺少 op"))?;
            let op =
                Cmp::parse(&op_s).ok_or_else(|| format!("assertions[{i}]: 未知 op '{op_s}'"))?;
            let value =
                get_num(item, "value").ok_or_else(|| format!("assertions[{i}]: 缺少 value"))?;
            if after_ms < 0.0 {
                return Err(format!("assertions[{i}]: after_ms 不能为负"));
            }
            spec.assertions.push(Assertion {
                after_ms,
                investor,
                metric,
                op,
                value,
                evaluated: false,
                pass: false,
                actual: f64::NAN,
            });
        }
    }
    Ok(spec)
}

/// Parse "HH:MM:SS[.mmm]" (a leading "YYYY-MM-DD " date part is accepted and
/// ignored — single trading day). Returns the trading-day timeline value
/// (`ctpbuddy_market::session_ms`): night-session clock times are negative.
pub fn parse_hms_ms(s: &str) -> Option<f64> {
    let s = s.trim();
    let time = match s.rsplit_once(' ') {
        Some((_, t)) => t,
        None => s,
    };
    let mut it = time.split(':');
    let h: f64 = it.next()?.parse().ok()?;
    let m: f64 = it.next()?.parse().ok()?;
    let sec: f64 = it.next()?.parse().ok()?;
    if it.next().is_some()
        || !(0.0..24.0).contains(&h)
        || !(0.0..60.0).contains(&m)
        || !(0.0..60.0).contains(&sec)
    {
        return None;
    }
    Some(ctpbuddy_market::session_ms(
        (h * 3600.0 + m * 60.0 + sec) * 1000.0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json;

    #[test]
    fn parses_full_spec() {
        let text = r#"{
            "name": "demo",
            "source": {"kind": "csv", "path": "ticks.csv"},
            "transforms": [
                {"kind": "freeze", "at_ms": 34200000, "duration_ms": 30000},
                {"kind": "gap", "at_ms": 34380000, "shift": -2},
                {"kind": "liquidity", "from_ms": 34260000, "scale": 0.5}
            ],
            "clock": {"time_scale": 0, "start_ms": 34200000},
            "accounts": [{"investor": "smoke001", "balance": 500000}],
            "assertions": [{"after_ms": 0, "investor": "smoke001", "metric": "balance", "op": "==", "value": 500000}]
        }"#;
        let v = json::parse(text).unwrap();
        let spec = parse_spec(&v).unwrap();
        assert_eq!(spec.name, "demo");
        assert_eq!(spec.transforms.len(), 3);
        assert_eq!(spec.time_scale, Some(0.0));
        assert_eq!(spec.start_ms, Some(34200000.0));
        assert_eq!(spec.accounts.len(), 1);
        assert_eq!(spec.accounts[0].investor, "smoke001");
        assert_eq!(spec.accounts[0].balance, Some(500000.0));
        assert!(spec.accounts[0].positions.is_empty());
        assert_eq!(spec.assertions.len(), 1);
        assert_eq!(spec.assertions[0].op, Cmp::Eq);
    }

    #[test]
    fn rejects_unknown_transform_kind() {
        let v = json::parse(r#"{"transforms": [{"kind": "splice"}]}"#).unwrap();
        let e = parse_spec(&v).unwrap_err();
        assert!(e.contains("splice"), "{e}");
    }

    #[test]
    fn rejects_bad_op() {
        let v = json::parse(
            r#"{"assertions": [{"after_ms": 0, "investor": "a", "metric": "balance", "op": "~=", "value": 1}]}"#,
        )
        .unwrap();
        assert!(parse_spec(&v).unwrap_err().contains("op"));
    }

    #[test]
    fn rejects_invalid_bootstrap_types_dates_and_margin() {
        for text in [
            r#"{"accounts":[{"investor":"x","positions":{}}]}"#,
            r#"{"accounts":[{"investor":"x","positions":[{"instrument":"rb2601","exchange":"SHFE","direction":"long","open_date":"20260230","trade_id":"t","open_price":1,"volume":1,"pre_settlement":1}]}]}"#,
            r#"{"accounts":[{"investor":"x","positions":[{"instrument":"rb2601","exchange":"SHFE","direction":"long","open_date":"20260202","trade_id":"t","open_price":1,"volume":1,"pre_settlement":1,"margin":"bad"}]}]}"#,
            r#"{"accounts":[{"investor":"x","positions":[{"instrument":"rb2601","exchange":"SHFE","direction":"long","open_date":"20260202","trade_id":"t","open_price":1,"volume":2147483648,"pre_settlement":1}]}]}"#,
        ] {
            let v = json::parse(text).unwrap();
            assert!(parse_spec(&v).is_err(), "accepted {text}");
        }
    }

    #[test]
    fn hms_parsing() {
        assert_eq!(parse_hms_ms("09:30:00"), Some(34200000.0));
        assert_eq!(parse_hms_ms("09:30:00.500"), Some(34200500.0));
        assert_eq!(parse_hms_ms("2026-10-02 09:30:00"), Some(34200000.0));
        assert_eq!(parse_hms_ms("nonsense"), None);
        assert_eq!(parse_hms_ms("25:00:00"), None);
        // night session sits before the day session on the same timeline
        assert_eq!(parse_hms_ms("21:00:00"), Some(-3.0 * 3_600_000.0));
        assert!(parse_hms_ms("23:59:00").unwrap() < parse_hms_ms("00:30:00").unwrap());
        assert!(parse_hms_ms("02:30:00").unwrap() < parse_hms_ms("09:00:00").unwrap());
    }

    #[test]
    fn cmp_apply() {
        assert!(Cmp::Ge.apply(1.0, 1.0));
        assert!(!Cmp::Gt.apply(1.0, 1.0));
        assert!(Cmp::Ne.apply(1.0, 2.0));
        assert!(Cmp::Lt.apply(-1.0, 0.0));
    }
}
