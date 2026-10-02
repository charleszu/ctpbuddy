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
      --scenario <dir>   scenario dir with instruments.csv + ticks.csv
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
            if n > 0.0 {
                cfg.initial_funds = n;
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
            }
        }
    }
    if let Ok(v) = std::env::var("CTPBUDDY_SCENARIO") {
        if !v.is_empty() {
            cfg.scenario_dir = Some(v);
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
            }
            "--scenario" => cfg.scenario_dir = Some(take("--scenario")?),
            "--qry-freq" => {
                let v = take("--qry-freq")?;
                cfg.qry_freq = v
                    .parse::<u32>()
                    .map_err(|_| format!("invalid --qry-freq: {v}"))?;
                if cfg.qry_freq < 1 {
                    return Err("--qry-freq must be >= 1".to_string());
                }
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
