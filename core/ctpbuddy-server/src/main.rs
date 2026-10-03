//! ctpbuddy-server binary: CLI + environment overrides -> world loop.
//!
//! Precedence: CLI flags > CTPBUDDY_* environment variables > built-in defaults.

use std::process::ExitCode;

use ctpbuddy_server::{run, Config};

const USAGE: &str = "\
ctpbuddy-server - local CTP-compatible simulated front-end

usage: ctpbuddy-server [options]

options:
  -a, --admin <addr>     admin control endpoint      (default 127.0.0.1:5561)
  -b, --broker-id <id>   the only BrokerID served     (default 8888)
      --data-dir <dir>   journal / runtime data dir   (default ./data)
  -h, --help             show this help
  -f, --initial-funds <n>
                          new-account initial funds    (default 2000000)
      --qry-freq <n>     ReqQry* budget per second     (default 2)
      --order-freq <n>   order insert+cancel budget per
                          second, per investor          (default 20)
      --refdata <dir>   contracts + margin / commission
                          rates (JSONL from a ref-data
                          provider; default = bundled
                          snapshot)
      --scenario <dir>   scenario dir with refdata/ + ticks.csv
      --speed <n>        playback speed multiplier    (0 = as fast as possible)
      --td <addr>        CTP td front endpoint        (default 127.0.0.1:5560)
  -v, --version          show version
";

/// Environment variables override built-in defaults (CLI wins over both).
fn apply_env(cfg: &mut Config) {
    let env_or = |key: &str, slot: &mut String| {
        if let Ok(v) = std::env::var(key) {
            if !v.is_empty() {
                *slot = v;
            }
        }
    };
    env_or("CTPBUDDY_TD_ADDR", &mut cfg.td_endpoint);
    env_or("CTPBUDDY_ADMIN_ADDR", &mut cfg.admin_endpoint);
    env_or("CTPBUDDY_BROKER_ID", &mut cfg.broker_id);
    env_or("CTPBUDDY_DATA_DIR", &mut cfg.data_dir);
    if let Ok(v) = std::env::var("CTPBUDDY_INITIAL_FUNDS") {
        if let Ok(n) = v.parse::<f64>() {
            if n.is_finite() && n >= 0.0 {
                cfg.initial_funds = n;
                cfg.settings_overrides.push("initial_funds".into());
            }
        }
    }
    if let Ok(v) = std::env::var("CTPBUDDY_SPEED") {
        if let Ok(n) = v.parse::<f64>() {
            cfg.playback_speed = Some(n);
        }
    }
    if let Ok(v) = std::env::var("CTPBUDDY_QRY_FREQ") {
        if let Ok(n) = v.parse::<u32>() {
            if n >= 1 {
                cfg.qry_freq = n;
                cfg.settings_overrides.push("qry_freq".into());
            }
        }
    }
    if let Ok(v) = std::env::var("CTPBUDDY_ORDER_FREQ") {
        if let Ok(n) = v.parse::<u32>() {
            if n >= 1 {
                cfg.order_freq = n;
                cfg.settings_overrides.push("order_freq".into());
            }
        }
    }
    for (env, key) in [("CTPBUDDY_MAX_USER_SESSIONS", "max_user_sessions"), ("CTPBUDDY_SETTLEMENT_REQUIRED", "settlement_required")] {
        if let Ok(v) = std::env::var(env) {
            let value = if key == "settlement_required" {
                match v.as_str() { "true" => ctpbuddy_server::json::b(true), "false" => ctpbuddy_server::json::b(false), _ => { eprintln!("invalid {env}: expected true/false"); std::process::exit(1); } }
            } else {
                match v.parse::<u32>() { Ok(n) => ctpbuddy_server::json::n(n as f64), Err(_) => { eprintln!("invalid {env}"); std::process::exit(1); } }
            };
            match ctpbuddy_server::settings::patch(cfg, &ctpbuddy_server::json::Value::Obj(vec![(key.into(), value)])) {
                Ok(next) => *cfg = next,
                Err(e) => { eprintln!("invalid {env}: {e}"); std::process::exit(1); }
            }
            cfg.settings_overrides.push(key.into());
        }
    }
    if let Ok(v) = std::env::var("CTPBUDDY_SCENARIO") {
        if !v.is_empty() {
            cfg.scenario_dir = Some(v);
        }
    }
    if let Ok(v) = std::env::var("CTPBUDDY_REFDATA") {
        if !v.is_empty() {
            cfg.refdata_dir = Some(v);
        }
    }
}

/// Parse CLI flags on top of `base` (which already carries env/defaults).
fn parse_cli(mut cfg: Config) -> Result<Config, String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        // Every value-taking option accepts a following value.
        let mut take = |what: &str| -> Result<String, String> {
            args.next()
                .ok_or_else(|| format!("missing value for {what}"))
        };
        match arg.as_str() {
            "-a" | "--admin" => cfg.admin_endpoint = take("--admin")?,
            "-b" | "--broker-id" => cfg.broker_id = take("--broker-id")?,
            "-f" | "--initial-funds" => {
                let v = take("--initial-funds")?;
                cfg.initial_funds = v
                    .parse::<f64>()
                    .map_err(|_| format!("invalid --initial-funds: {v}"))?;
                if !cfg.initial_funds.is_finite() || cfg.initial_funds < 0.0 { return Err("--initial-funds must be finite and >= 0".into()); }
                cfg.settings_overrides.push("initial_funds".into());
            }
            "--scenario" => cfg.scenario_dir = Some(take("--scenario")?),
            "--refdata" => cfg.refdata_dir = Some(take("--refdata")?),
            "--qry-freq" => {
                let v = take("--qry-freq")?;
                cfg.qry_freq = v
                    .parse::<u32>()
                    .map_err(|_| format!("invalid --qry-freq: {v}"))?;
                if cfg.qry_freq < 1 {
                    return Err("--qry-freq must be >= 1".to_string());
                }
                cfg.settings_overrides.push("qry_freq".into());
            }
            "--order-freq" => {
                let v = take("--order-freq")?;
                cfg.order_freq = v
                    .parse::<u32>()
                    .map_err(|_| format!("invalid --order-freq: {v}"))?;
                if cfg.order_freq < 1 {
                    return Err("--order-freq must be >= 1".to_string());
                }
                cfg.settings_overrides.push("order_freq".into());
            }
            "--max-user-sessions" | "--settlement-required" => {
                let key = if arg == "--max-user-sessions" { "max_user_sessions" } else { "settlement_required" };
                let v = take(&arg)?;
                let value = if key == "settlement_required" {
                    match v.as_str() { "true" => ctpbuddy_server::json::b(true), "false" => ctpbuddy_server::json::b(false), _ => return Err("--settlement-required expects true/false".into()) }
                } else {
                    ctpbuddy_server::json::n(v.parse::<u32>().map_err(|_| "invalid session limit")? as f64)
                };
                cfg = ctpbuddy_server::settings::patch(&cfg, &ctpbuddy_server::json::Value::Obj(vec![(key.into(), value)]))?;
                cfg.settings_overrides.push(key.into());
            }
            "--speed" => {
                let v = take("--speed")?;
                let n = v
                    .parse::<f64>()
                    .map_err(|_| format!("invalid --speed: {v}"))?;
                cfg.playback_speed = Some(n);
            }
            "--td" => cfg.td_endpoint = take("--td")?,
            "--data-dir" => cfg.data_dir = take("--data-dir")?,
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "-v" | "--version" => {
                println!("ctpbuddy-server {}", ctpbuddy_server::SERVER_VERSION);
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}\n\n{USAGE}")),
        }
    }
    Ok(cfg)
}

fn main() -> ExitCode {
    let mut cfg = Config::default();
    apply_env(&mut cfg);
    let mut cfg = match parse_cli(cfg) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    if cfg.data_dir.is_empty() {
        cfg.data_dir = "data".to_string();
    }

    match run(cfg) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("[ctpbuddy] fatal: {e}");
            ExitCode::FAILURE
        }
    }
}
