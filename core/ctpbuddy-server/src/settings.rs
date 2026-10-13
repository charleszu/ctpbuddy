//! 柜台参数：严格 schema、整批校验、落盘后生效。
use crate::{
    json::{self, Value},
    Config, World,
};
use std::{collections::HashSet, io::Write, path::Path};

/// One row per runtime setting; the single source for patch validation and
/// the schema the Web settings page renders.
pub struct SettingDef {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub lifecycle: &'static str,
    /// "boolean" | "integer" | "number"
    pub kind: &'static str,
    pub min: f64,
    pub max: f64,
}

const SETTING_DEFS: &[SettingDef] = &[
    SettingDef {
        key: "qry_freq",
        label: "每会话查询额度",
        description: "每秒 ReqQry* 额度，超出返回 90；修改不清空窗口。",
        lifecycle: "runtime",
        kind: "integer",
        min: 1.0,
        max: 100_000.0,
    },
    SettingDef {
        key: "order_freq",
        label: "报单 / 撤单额度",
        description: "每用户每秒额度，报单与撤单各自独立计数，超出返回 116。",
        lifecycle: "runtime",
        kind: "integer",
        min: 1.0,
        max: 100_000.0,
    },
    SettingDef {
        key: "max_user_sessions",
        label: "用户在线会话上限",
        description: "按 BrokerID + UserID 计已登录连接，0 关闭限制（兼容旧行为）；降低不踢已有会话。",
        lifecycle: "runtime",
        kind: "integer",
        min: 0.0,
        max: 10_000.0,
    },
    SettingDef {
        key: "settlement_required",
        label: "结算确认门禁",
        description: "默认开启；未确认当前交易日的报单返回官方 42；不实施日结。",
        lifecycle: "runtime",
        kind: "boolean",
        min: 0.0,
        max: 0.0,
    },
    SettingDef {
        key: "self_trade_prevention",
        label: "自成交预防",
        description: "默认关闭（真实交易所不阻止同一投资者与自己的挂单成交）；开启后撮合跳过同账户挂单，会打破价格时间优先。",
        lifecycle: "runtime",
        kind: "boolean",
        min: 0.0,
        max: 0.0,
    },
    SettingDef {
        key: "initial_funds",
        label: "初次开户默认资金",
        description: "仅影响首次登录自动开户；已有账户与场景显式账户资金不变。",
        lifecycle: "new-account",
        kind: "number",
        min: 0.0,
        max: 1e12,
    },
];

fn setting_def(key: &str) -> Option<&'static SettingDef> {
    SETTING_DEFS.iter().find(|d| d.key == key)
}

/// Boolean settings (everything else is numeric).
pub fn is_boolean(key: &str) -> bool {
    setting_def(key).is_some_and(|d| d.kind == "boolean")
}

pub fn values(cfg: &Config) -> Value {
    json::obj_sorted(vec![
        ("qry_freq".into(), json::n(cfg.qry_freq as f64)),
        ("order_freq".into(), json::n(cfg.order_freq as f64)),
        (
            "max_user_sessions".into(),
            json::n(cfg.max_user_sessions as f64),
        ),
        (
            "settlement_required".into(),
            json::b(cfg.settlement_required),
        ),
        (
            "self_trade_prevention".into(),
            json::b(cfg.self_trade_prevention),
        ),
        ("initial_funds".into(), json::n(cfg.initial_funds)),
    ])
}

pub fn patch(cfg: &Config, v: &Value) -> Result<Config, String> {
    let Value::Obj(pairs) = v else {
        return Err("patch 必须是 JSON 对象".into());
    };
    let mut next = cfg.clone();
    let mut seen = HashSet::new();
    for (key, value) in pairs {
        if !seen.insert(key) {
            return Err(format!("重复参数 {key}"));
        }
        let Some(def) = setting_def(key) else {
            return Err(format!("未知参数 {key}"));
        };
        if def.kind == "boolean" {
            let Value::Bool(b) = value else {
                return Err(format!("{key} 必须是布尔值"));
            };
            match key.as_str() {
                "settlement_required" => next.settlement_required = *b,
                "self_trade_prevention" => next.self_trade_prevention = *b,
                _ => unreachable!(),
            }
            continue;
        }
        let Value::Num(n) = value else {
            return Err(format!("{key} 必须是数字"));
        };
        let (min, max) = (def.min, def.max);
        if !n.is_finite() || *n < min || *n > max || (def.kind != "number" && n.fract() != 0.0) {
            return Err(format!(
                "{key} 超出范围 [{min}, {max}] 或不是合法整数/有限数字"
            ));
        }
        match key.as_str() {
            "qry_freq" => next.qry_freq = *n as u32,
            "order_freq" => next.order_freq = *n as u32,
            "max_user_sessions" => next.max_user_sessions = *n as u32,
            "initial_funds" => next.initial_funds = *n,
            _ => unreachable!(),
        }
    }
    Ok(next)
}

pub fn load(cfg: &mut Config) -> Result<(), String> {
    patch(cfg, &values(cfg))?;
    if cfg.data_dir.is_empty() {
        return Ok(());
    }
    let path = Path::new(&cfg.data_dir).join("settings.json");
    let text = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(format!("读取 {} 失败: {e}", path.display())),
    };
    let persisted = patch(cfg, &json::parse(&text)?)?;
    let current = values(cfg);
    let mut merged = persisted;
    for key in &cfg.settings_overrides {
        merged = patch(
            &merged,
            &Value::Obj(vec![(key.clone(), current.get(key).unwrap().clone())]),
        )?;
    }
    eprintln!(
        "[ctpbuddy] settings: CLI/env > 持久化 > 默认；启动覆盖字段 {:?}",
        cfg.settings_overrides
    );
    *cfg = merged;
    Ok(())
}

fn persist(cfg: &Config) -> Result<(), String> {
    if cfg.data_dir.is_empty() {
        return Err("data_dir 未启用，拒绝不可持久化的更新".into());
    }
    let dir = Path::new(&cfg.data_dir);
    let dest = dir.join("settings.json");
    let tmp = dir.join("settings.json.tmp");
    let result = (|| -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)?;
        file.write_all(values(cfg).to_json().as_bytes())?;
        file.sync_all()?;
        drop(file);
        replace(&tmp, &dest)
    })();
    result.map_err(|e| format!("设置未生效：原子持久化失败: {e}"))
}

#[cfg(not(windows))]
fn replace(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::rename(src, dst)
}
#[cfg(windows)]
fn replace(src: &Path, dst: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    extern "system" {
        fn MoveFileExW(from: *const u16, to: *const u16, flags: u32) -> i32;
    }
    let from: Vec<u16> = src.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = dst.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 1 | 8) } == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn schema() -> Value {
    Value::Arr(
        SETTING_DEFS
            .iter()
            .map(|d| {
                json::obj_sorted(vec![
                    ("key".into(), json::s(d.key)),
                    ("label".into(), json::s(d.label)),
                    ("description".into(), json::s(d.description)),
                    ("lifecycle".into(), json::s(d.lifecycle)),
                    ("type".into(), json::s(d.kind)),
                    ("min".into(), json::n(d.min)),
                    ("max".into(), json::n(d.max)),
                ])
            })
            .collect(),
    )
}

impl World {
    pub(crate) fn settings_reply(&self) -> Value {
        json::obj_sorted(vec![("ok".into(), json::b(true)), ("settings".into(), values(&self.cfg)), ("schema".into(), schema()), ("precedence".into(), json::s("启动：CLI > env > settings.json > 默认；运行时 Web/ADMIN 更新生效，重启仍受显式启动覆盖影响")), ("startup_overrides".into(), Value::Arr(self.cfg.settings_overrides.iter().map(|k| json::s(k)).collect())), ("persistence_enabled".into(), json::b(!self.cfg.data_dir.is_empty() && self.journal.is_some()))])
    }
    pub(crate) fn update_settings(&mut self, v: &Value) -> Result<Value, String> {
        let Value::Obj(pairs) = v else {
            return Err("请求必须是对象".into());
        };
        if let Some(k) = json::unknown_or_duplicate(pairs, &["cmd", "patch"]) {
            return Err(format!("未知或重复请求键 {k}"));
        }
        let next = patch(&self.cfg, v.get("patch").ok_or("缺少 patch")?)?;
        if self.journal.is_none() {
            return Err("journal 不可用，拒绝无审计更新".into());
        }
        persist(&next)?;
        let before = values(&self.cfg);
        let after = values(&next);
        self.ledger.set_initial_funds(next.initial_funds);
        self.engine
            .set_self_trade_prevention(next.self_trade_prevention);
        self.cfg = next;
        self.journal_record_json(
            "settings_updated",
            "",
            "",
            json::obj_sorted(vec![
                ("before".into(), before.clone()),
                ("after".into(), after.clone()),
            ]),
        );
        let seq = self.journal.as_ref().unwrap().seq();
        let flush_error = self
            .journal
            .as_mut()
            .unwrap()
            .flush()
            .err()
            .map(|e| e.to_string());
        let mut reply = self.settings_reply();
        if let Value::Obj(ref mut p) = reply {
            p.push((
                "audit".into(),
                json::obj_sorted(vec![
                    ("seq".into(), json::n(seq as f64)),
                    ("before".into(), before),
                    ("after".into(), after),
                    (
                        "flush_error".into(),
                        flush_error.map(|e| json::s(&e)).unwrap_or(Value::Null),
                    ),
                ]),
            ));
        }
        Ok(reply)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_atomic_patch() {
        let cfg = Config::default();
        for text in [
            "{\"qry_freq\":0}",
            "{\"qry_freq\":1.5}",
            "{\"qry_freq\":true}",
            "{\"qry_freq\":\"2\"}",
            "{\"max_user_sessions\":10001}",
            "{\"settlement_required\":1}",
            "{\"self_trade_prevention\":\"true\"}",
            "{\"qry_freq\":3,\"unknown\":4}",
            "{\"qry_freq\":2,\"qry_freq\":3}",
        ] {
            assert!(patch(&cfg, &json::parse(text).unwrap()).is_err(), "{text}");
        }
        // out-of-range numbers are rejected by the JSON parser itself
        assert!(json::parse("{\"initial_funds\":1e999}").is_err());
        assert_eq!(cfg.qry_freq, 2);
        assert!(!cfg.self_trade_prevention);
        assert!(
            patch(
                &cfg,
                &json::parse("{\"self_trade_prevention\":true}").unwrap()
            )
            .unwrap()
            .self_trade_prevention
        );
        assert_eq!(
            patch(
                &cfg,
                &json::parse("{\"max_user_sessions\":0,\"initial_funds\":0}").unwrap()
            )
            .unwrap()
            .initial_funds,
            0.0
        );
    }
}
