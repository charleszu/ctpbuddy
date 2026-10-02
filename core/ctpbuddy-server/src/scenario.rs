//! Scenario DSL spec (DESIGN.md §7.4): the JSON form the control plane sends.
//!
//! YAML is the authoring format; the Python control plane parses and validates
//! it and normalizes to this shape — times already resolved to virtual ms
//! since midnight, one assertion per `expect` entry. The core stays a pure
//! consumer: no YAML parser, no locale, no wall clock in the pipeline.

use ctpbuddy_market::transform::Transform;

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
    /// `accounts`: (investor, optional initial balance).
    pub accounts: Vec<(String, Option<f64>)>,
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
    if let Some(Value::Arr(items)) = v.get("accounts") {
        for (i, item) in items.iter().enumerate() {
            let investor = item
                .get_str("investor")
                .ok_or_else(|| format!("accounts[{i}]: 缺少 investor"))?;
            if investor.trim().is_empty() {
                return Err(format!("accounts[{i}]: investor 不能为空"));
            }
            let balance = get_num(item, "balance");
            if let Some(b) = balance {
                if b <= 0.0 {
                    return Err(format!("accounts[{i}]: balance 必须为正"));
                }
            }
            spec.accounts.push((investor, balance));
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
            let op = Cmp::parse(&op_s)
                .ok_or_else(|| format!("assertions[{i}]: 未知 op '{op_s}'"))?;
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
/// ignored — single-day replay). Returns virtual ms since midnight.
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
    if it.next().is_some() || !(0.0..24.0).contains(&h) || !(0.0..60.0).contains(&m) || !(0.0..60.0).contains(&sec) {
        return None;
    }
    Some((h * 3600.0 + m * 60.0 + sec) * 1000.0)
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
        assert_eq!(spec.accounts, vec![("smoke001".to_string(), Some(500000.0))]);
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
    fn hms_parsing() {
        assert_eq!(parse_hms_ms("09:30:00"), Some(34200000.0));
        assert_eq!(parse_hms_ms("09:30:00.500"), Some(34200500.0));
        assert_eq!(parse_hms_ms("2026-10-02 09:30:00"), Some(34200000.0));
        assert_eq!(parse_hms_ms("nonsense"), None);
        assert_eq!(parse_hms_ms("25:00:00"), None);
    }

    #[test]
    fn cmp_apply() {
        assert!(Cmp::Ge.apply(1.0, 1.0));
        assert!(!Cmp::Gt.apply(1.0, 1.0));
        assert!(Cmp::Ne.apply(1.0, 2.0));
        assert!(Cmp::Lt.apply(-1.0, 0.0));
    }
}
