//! 柜台参数：严格 schema、整批校验、落盘后生效。
use crate::{
    json::{self, Value},
    Config, World,
};
use std::{collections::HashSet, io::Write, path::Path};

pub const KEYS: [&str; 5] = [
    "qry_freq",
    "order_freq",
    "max_user_sessions",
    "settlement_required",
    "initial_funds",
];

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
        if !KEYS.contains(&key.as_str()) {
            return Err(format!("未知参数 {key}"));
        }
        if key == "settlement_required" {
            let Value::Bool(b) = value else {
                return Err("settlement_required 必须是布尔值".into());
            };
            next.settlement_required = *b;
            continue;
        }
        let Value::Num(n) = value else {
            return Err(format!("{key} 必须是数字"));
        };
        let (min, max) = match key.as_str() {
            "initial_funds" => (0.0, 1_000_000_000_000.0),
            "max_user_sessions" => (0.0, 10_000.0),
            _ => (1.0, 100_000.0),
        };
        if !n.is_finite() || *n < min || *n > max || (key != "initial_funds" && n.fract() != 0.0) {
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
    Value::Arr(KEYS.iter().map(|k| {
        let (label, description, lifecycle, min, max, kind) = match *k {
            "qry_freq" => ("每会话查询额度", "每秒 ReqQry* 额度，超出返回 90；修改不清空窗口。", "runtime", 1.0, 100000.0, "integer"),
            "order_freq" => ("报单 / 撤单额度", "每用户每秒额度，报单与撤单各自独立计数，超出返回 116。", "runtime", 1.0, 100000.0, "integer"),
            "max_user_sessions" => ("用户在线会话上限", "按 BrokerID + UserID 计已登录连接，0 关闭限制（兼容旧行为）；降低不踢已有会话。", "runtime", 0.0, 10000.0, "integer"),
            "settlement_required" => ("结算确认门禁", "默认开启；未确认当前交易日的报单返回官方 42；不实施日结。", "runtime", 0.0, 0.0, "boolean"),
            _ => ("初次开户默认资金", "仅影响首次登录自动开户；已有账户与场景显式账户资金不变。", "new-account", 0.0, 1e12, "number"),
        };
        json::obj_sorted(vec![("key".into(), json::s(k)), ("label".into(), json::s(label)), ("description".into(), json::s(description)), ("lifecycle".into(), json::s(lifecycle)), ("type".into(), json::s(kind)), ("min".into(), json::n(min)), ("max".into(), json::n(max))])
    }).collect())
}

impl World {
    pub(crate) fn settings_reply(&self) -> Value {
        json::obj_sorted(vec![("ok".into(), json::b(true)), ("settings".into(), values(&self.cfg)), ("schema".into(), schema()), ("precedence".into(), json::s("启动：CLI > env > settings.json > 默认；运行时 Web/ADMIN 更新生效，重启仍受显式启动覆盖影响")), ("startup_overrides".into(), Value::Arr(self.cfg.settings_overrides.iter().map(|k| json::s(k)).collect())), ("persistence_enabled".into(), json::b(!self.cfg.data_dir.is_empty() && self.journal.is_some()))])
    }
    pub(crate) fn update_settings(&mut self, v: &Value) -> Result<Value, String> {
        let Value::Obj(pairs) = v else {
            return Err("请求必须是对象".into());
        };
        let mut seen = HashSet::new();
        for (k, _) in pairs {
            if !seen.insert(k) || !["cmd", "patch"].contains(&k.as_str()) {
                return Err(format!("未知或重复请求键 {k}"));
            }
        }
        let next = patch(&self.cfg, v.get("patch").ok_or("缺少 patch")?)?;
        if self.journal.is_none() {
            return Err("journal 不可用，拒绝无审计更新".into());
        }
        persist(&next)?;
        let before = values(&self.cfg);
        let after = values(&next);
        self.ledger.set_initial_funds(next.initial_funds);
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
            "{\"initial_funds\":1e999}",
            "{\"qry_freq\":3,\"unknown\":4}",
            "{\"qry_freq\":2,\"qry_freq\":3}",
        ] {
            assert!(patch(&cfg, &json::parse(text).unwrap()).is_err(), "{text}");
        }
        assert_eq!(cfg.qry_freq, 2);
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
