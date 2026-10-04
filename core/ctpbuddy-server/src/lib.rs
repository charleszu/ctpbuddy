//! ctpbuddy-server: the single-threaded world loop plus its TCP transports.
//!
//! Transport note: DESIGN.md §6 targets ZeroMQ; M1 carries the exact same
//! frames over plain TCP (one connection per client, frames back-to-back).
//! The framing is transport-agnostic by construction, so swapping in ZMTP is
//! a transport adapter, not a protocol change.

pub mod admin;
pub mod dtime;
pub mod handlers;
pub mod journal;
pub mod json;
pub mod scenario;
pub mod settings;
mod settlement;

use std::collections::{HashMap, HashSet};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use ctpbuddy_ledger::Ledger;
use ctpbuddy_market::{format_hhmmss, CsvSource, Playback, Tick};
use ctpbuddy_matching::{Catalog, EngineEvent, MarginPrice, MatchingEngine};
use ctpbuddy_wire::frame::Frame;
use ctpbuddy_wire::generated::{cstr, set_cstr, CThostFtdcOrderField, CThostFtdcTradeField};
use ctpbuddy_wire::msgs;
use ctpbuddy_wire::struct_to_bytes;

use journal::Journal;
use json::Value;

pub const SERVER_NAME: &str = "CTPBuddy";
pub const SERVER_VERSION: &str = "0.1.0";
/// World-loop pulse period: playback tick release + mark-to-market cadence.
pub const PULSE: Duration = Duration::from_millis(10);

/// Scenario assertion `status` values written to the journal (DESIGN §7.4).
const ASSERTION_OK: &str = "ok";
const ASSERTION_ACCOUNT_NOT_FOUND: &str = "account_not_found";
const ASSERTION_UNKNOWN_METRIC: &str = "unknown_metric";
const ASSERTION_NOT_FINITE: &str = "not_finite";

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

/// Id the next accepted connection will receive (never reused).
pub(crate) fn next_conn_id() -> u64 {
    NEXT_CONN_ID.load(Ordering::SeqCst)
}

/// 报撤单流控的两条独立预算流（notes/14 §A.3-02：`ReqOrderInsert` 与
/// `ReqOrderAction`「这两个函数流控是分开计算的」）。混做报撤的策略按流
/// 各自受限，绝不是两者之和受限。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GateStream {
    Insert,
    Cancel,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub td_endpoint: String,
    pub admin_endpoint: String,
    /// The only BrokerID this core instance serves (DESIGN.md §6.4).
    pub broker_id: String,
    pub scenario_dir: Option<String>,
    /// Reference-data directory (canonical JSONL from a ref-data provider).
    /// `None` falls back to the built-in **demo** fixture, whose rates are
    /// invented — a real deployment points this at the desk's own contract
    /// and rate data (see `core/ctpbuddy-matching/src/refdata.rs`).
    pub refdata_dir: Option<String>,
    pub initial_funds: f64,
    /// Playback speed multiplier; `None` = unspecified (a scenario's
    /// `clock.time_scale` or the as-fast-as-possible default applies).
    /// 0 = as fast as possible.
    pub playback_speed: Option<f64>,
    /// Directory for the journal / logs. Empty = disabled.
    pub data_dir: String,
    /// 查询流控 (docs: 报单流控、查询流控和会话数控制): per-session
    /// ReqQry* budget per second. Exceeding it answers OnRspError[90]
    /// "CTP：查询未就绪，请稍后重试", same as a real front (front_se QryFreq).
    pub qry_freq: u32,
    /// 报单流控 (DESIGN §8.3): per-(broker, investor) budget per second for
    /// order inserts — and, counted **separately**, the same budget for
    /// cancels. `ReqOrderInsert` and `ReqOrderAction` are two independent
    /// streams (notes/14 §A.3-02「这两个函数流控是分开计算的」); a mixed
    /// insert/cancel strategy is limited per stream, never by their sum.
    /// Exceeding a stream rejects that request outright with
    /// "CTP:下单频率限制" (modern front-office behavior; the 2009 FAQ era
    /// 6/s-queued-silently default is injectable for testing legacy
    /// downstreams).
    pub order_freq: u32,
    /// 0 disables the per-user online-session limit.
    pub max_user_sessions: u32,
    /// Require settlement confirmation before orders (official error 42).
    pub settlement_required: bool,
    /// Skip resting orders of the same (broker, investor) when matching
    /// (DESIGN §8.3). Off by default: real CTP does not block an investor
    /// from crossing their own order.
    pub self_trade_prevention: bool,
    /// Fields explicitly supplied by CLI/env; they override persisted settings.
    pub settings_overrides: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            td_endpoint: "127.0.0.1:5560".into(),
            admin_endpoint: "127.0.0.1:5561".into(),
            broker_id: "8888".into(),
            scenario_dir: None,
            refdata_dir: None,
            initial_funds: 2_000_000.0,
            playback_speed: None,
            data_dir: String::new(),
            qry_freq: 2,
            order_freq: 20,
            max_user_sessions: 0,
            settlement_required: true,
            self_trade_prevention: false,
            settings_overrides: Vec::new(),
        }
    }
}

/// Bind transports and run the world loop until an admin `shutdown`.
pub fn run(mut cfg: Config) -> std::io::Result<()> {
    settings::load(&mut cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let td = TcpListener::bind(&cfg.td_endpoint)?;
    let admin = TcpListener::bind(&cfg.admin_endpoint)?;
    let mut world = World::new(cfg.clone())?;
    let (tx, rx) = mpsc::channel::<WorldMsg>();

    let tx_td = tx.clone();
    thread::spawn(move || accept_loop(td, tx_td, false));
    let tx_admin = tx.clone();
    thread::spawn(move || accept_loop(admin, tx_admin, true));
    drop(tx); // keep only the acceptor/conn threads' clones alive

    println!(
        "[ctpbuddy] td        {} (broker {})",
        cfg.td_endpoint, cfg.broker_id
    );
    println!("[ctpbuddy] admin     {}", cfg.admin_endpoint);
    if let Some(dir) = &cfg.scenario_dir {
        println!("[ctpbuddy] scenario  {dir}");
    }

    world.run_loop(rx);
    println!("[ctpbuddy] stopped");
    Ok(())
}

fn accept_loop(listener: TcpListener, tx: Sender<WorldMsg>, is_admin: bool) {
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let _ = s.set_nodelay(true);
                let id = NEXT_CONN_ID.fetch_add(1, Ordering::SeqCst);
                let (writer_tx, writer_rx) = mpsc::channel::<Frame>();
                let _ = tx.send(WorldMsg::Opened {
                    id,
                    writer: writer_tx,
                    is_admin,
                });
                let reader = match s.try_clone() {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let tx_r = tx.clone();
                thread::spawn(move || reader_loop(id, reader, tx_r));
                thread::spawn(move || writer_loop(s, writer_rx));
            }
            Err(e) => {
                eprintln!("[ctpbuddy] accept error: {e}");
                break;
            }
        }
    }
}

fn reader_loop(id: u64, mut stream: TcpStream, tx: Sender<WorldMsg>) {
    loop {
        match Frame::read_from(&mut stream) {
            Ok(Some(frame)) => {
                if tx.send(WorldMsg::Inbound { id, frame }).is_err() {
                    break;
                }
            }
            Ok(None) => {
                let _ = tx.send(WorldMsg::Closed { id });
                break;
            }
            Err(e) => {
                eprintln!("[ctpbuddy] conn {id} read error: {e}");
                let _ = tx.send(WorldMsg::Closed { id });
                break;
            }
        }
    }
}

fn writer_loop(mut stream: TcpStream, rx: Receiver<Frame>) {
    // Ends when the world drops our sender (conn removed) or the socket dies;
    // the reader loop detects EOF on its own.
    for frame in rx {
        if frame.write_to(&mut stream).is_err() {
            break;
        }
    }
}

pub(crate) enum WorldMsg {
    Opened {
        id: u64,
        writer: Sender<Frame>,
        is_admin: bool,
    },
    Inbound {
        id: u64,
        frame: Frame,
    },
    Closed {
        id: u64,
    },
}

/// One client connection (a "front session" in CTP terms).
struct Conn {
    writer: Sender<Frame>,
    is_admin: bool,
    authenticated: bool,
    /// front id assigned at connect (CTP: per front server address).
    front_id: i32,
    session_id: i32,
    logins: u32,
    broker_id: Option<String>,
    investor_id: Option<String>,
    user_id: [u8; 16],
    /// front-assigned OrderLocalID counter.
    order_local_seq: u32,
    md_subs: HashSet<String>,
    /// 查询流控 window (CTP docs: per-session QryFreq per second).
    qry_window: Option<Instant>,
    qry_count: u32,
}

/// Read + parse `scenario.json` (the compiled form of scenario.yaml) at
/// startup. A missing file means a legacy ticks.csv-only scenario; a broken
/// file is reported and ignored (the core still serves the raw tick stream).
fn load_startup_spec(dir: &str) -> Option<scenario::Spec> {
    let path = format!("{dir}/scenario.json");
    let text = std::fs::read_to_string(&path).ok()?;
    match json::parse(&text) {
        Ok(v) => match scenario::parse_spec(&v) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("[ctpbuddy] scenario.json invalid: {e}");
                None
            }
        },
        Err(e) => {
            eprintln!("[ctpbuddy] scenario.json parse failed: {e}");
            None
        }
    }
}

/// Load the contract directory + rate tables.
///
/// Precedence, most specific first:
/// 1. `refdata_dir` from `--refdata` (canonical JSONL from a provider) — the
///    authoritative path, and what a real deployment uses;
/// 2. `<scenario>/refdata.jsonl` or a `refdata` subdirectory, so a scenario
///    can be self-contained;
/// 3. [`Catalog::bundled`] — the 789-contract snapshot shipped with CTPBuddy,
///    so a bare `ctpbuddy up` has something real to trade.
///
/// An explicit `--refdata` that fails to load is a hard error: silently
/// falling back to made-up rates would make a misconfigured desk believe it
/// is repriced.
fn load_refdata(explicit: Option<&str>, scenario_dir: Option<&str>) -> Result<Catalog, String> {
    if let Some(dir) = explicit {
        return Catalog::load_refdata_dir(dir).map_err(|e| format!("--refdata {dir}: {e}"));
    }
    if let Some(sdir) = scenario_dir {
        let candidates = [format!("{sdir}/refdata"), sdir.to_string()];
        for cand in candidates {
            if std::path::Path::new(&format!("{cand}/instruments.jsonl")).exists() {
                return Catalog::load_refdata_dir(&cand);
            }
        }
    }
    Ok(Catalog::bundled())
}

/// Resolve a scenario directory into (catalog, transformed tick stream).
/// Shared by startup and the admin `start_scenario` command so both paths
/// apply the pipeline identically (DESIGN §7.4: source → transforms → clock).
fn build_scenario(
    dir: &str,
    spec: Option<&scenario::Spec>,
    refdata_dir: Option<&str>,
) -> Result<(Catalog, Vec<Tick>), String> {
    let catalog = load_refdata(refdata_dir, Some(dir))?;
    let ticks_path = match spec.and_then(|s| s.source_path.as_deref()) {
        Some(p) => format!("{dir}/{p}"),
        None => format!("{dir}/ticks.csv"),
    };
    let ticks = CsvSource::load(&ticks_path).map_err(|e| format!("{}: {e}", ticks_path))?;
    let ticks = match spec {
        Some(s) => {
            let mut t = ctpbuddy_market::transform::apply_all(&ticks, &s.transforms);
            if let Some(start) = s.start_ms {
                // clock.start: ticks before the start are dropped, never delivered
                t.retain(|tk| tk.virtual_ms() >= start);
            }
            t
        }
        None => ticks,
    };
    Ok((catalog, ticks))
}

/// The whole world: market, matching, ledger, sessions, transports.
/// State is mutated exclusively inside `run_loop`.
pub struct World {
    cfg: Config,
    engine: MatchingEngine,
    ledger: Ledger,
    playback: Option<Playback>,
    journal: Option<Journal>,
    conns: HashMap<u64, Conn>,
    /// Virtual clock state (DESIGN.md §7): the only time authority.
    vt_trading_day: String,
    vt_now_ms: f64,
    /// Today's full order notifications (QryOrder projection + journal).
    orders_today: Vec<CThostFtdcOrderField>,
    /// Today's fills (QryTrade projection).
    trades_today: Vec<CThostFtdcTradeField>,
    /// Loaded scenario name ("" when none / legacy dir without scenario.json).
    scenario_name: String,
    /// Virtual ms of the scenario's first delivered tick (assertion `after`
    /// is relative to it). None when no scenario is loaded.
    scenario_t0_ms: Option<f64>,
    /// Scenario assertions (DESIGN §7.4), one-shot at their virtual time.
    assertions: Vec<scenario::Assertion>,
    /// 报单流控窗口 (DESIGN §8.3): per-(broker, investor, stream) one-second
    /// window. Insert and cancel are **separate streams with separate
    /// budgets** (notes/14 §A.3-02), so the key carries the stream. Wall
    /// clock, exactly like `qry_gate` — real CTP throttles on real time.
    order_freq_windows: HashMap<(String, String, GateStream), (Instant, u32)>,
    settlement_confirmed: HashMap<(String, String), String>,
    settlement_reports: Vec<settlement::Report>,
    shutdown: bool,
}

impl World {
    pub fn new(cfg: Config) -> std::io::Result<Self> {
        let mut journal = if cfg.data_dir.is_empty() {
            None
        } else {
            Some(Journal::new(&cfg.data_dir)?)
        };

        // Ref data resolution order: explicit --refdata > a `refdata/`
        // directory inside the scenario > the bundled contract snapshot.
        // A directory that *exists but fails to load* is a hard error
        // (DESIGN §8.6.1): only a missing default path may fall back.
        let mut engine_catalog =
            load_refdata(cfg.refdata_dir.as_deref(), cfg.scenario_dir.as_deref())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut playback = None;
        let mut vt_day = dtime::today_trading_day();
        // Before any scenario clock exists there is no virtual time: 0.0 is
        // the deterministic pre-clock value. Journal envelopes must be a pure
        // function of scenario state (DESIGN §11.4 hash determinism), so the
        // startup path may NOT seed vt from the wall clock.
        let mut vt_ms = 0.0;
        let mut scenario_name = String::new();
        let mut scenario_t0_ms = None;
        let mut assertions = Vec::new();
        let mut startup_accounts: Vec<scenario::AccountSpec> = Vec::new();

        if let Some(dir) = &cfg.scenario_dir {
            // scenario.json is the compiled form of scenario.yaml (written by
            // `ctpbuddy scenario compile`); no file = legacy ticks.csv-only.
            let spec = load_startup_spec(dir);
            match build_scenario(dir, spec.as_ref(), cfg.refdata_dir.as_deref()) {
                Ok((catalog, ticks)) => {
                    engine_catalog = catalog;
                    if let Some(first) = ticks.first() {
                        vt_day = first.trading_day.clone();
                        vt_ms = first.virtual_ms();
                        scenario_t0_ms = Some(first.virtual_ms());
                    }
                    if let Some(s) = &spec {
                        scenario_name = s.name.clone();
                        assertions = s.assertions.clone();
                        startup_accounts = s.accounts.clone();
                    }
                    let speed = cfg
                        .playback_speed
                        .or(spec.as_ref().and_then(|s| s.time_scale))
                        .unwrap_or(0.0);
                    playback = Some(Playback::new(ticks, speed));
                    println!(
                        "[ctpbuddy] ticks {} (scenario {})",
                        playback.as_ref().unwrap().progress().1,
                        if scenario_name.is_empty() {
                            "legacy"
                        } else {
                            &scenario_name
                        }
                    );
                }
                Err(e) => eprintln!("[ctpbuddy] scenario load failed: {e}"),
            }
        }

        if let Some(j) = journal.as_mut() {
            j.record(
                &vt_day,
                vt_ms,
                "server_start",
                &cfg.broker_id,
                "",
                json::obj_sorted(vec![
                    ("version".into(), json::s(SERVER_VERSION)),
                    (
                        "scenario".into(),
                        json::s(cfg.scenario_dir.as_deref().unwrap_or("")),
                    ),
                ]),
            );
            j.flush()?;
        }

        let mut engine = MatchingEngine::new(engine_catalog);
        engine.set_self_trade_prevention(cfg.self_trade_prevention);
        let mut world = World {
            engine,
            ledger: Ledger::new(cfg.initial_funds),
            playback,
            journal,
            conns: HashMap::new(),
            vt_trading_day: vt_day,
            vt_now_ms: vt_ms,
            orders_today: Vec::new(),
            trades_today: Vec::new(),
            scenario_name,
            scenario_t0_ms,
            assertions,
            order_freq_windows: HashMap::new(),
            settlement_confirmed: HashMap::new(),
            settlement_reports: settlement::load_reports(&cfg.data_dir, &cfg.broker_id)
                .unwrap_or_else(|e| {
                    eprintln!("[ctpbuddy] settlement reports disabled: {e}");
                    Vec::new()
                }),
            shutdown: false,
            cfg,
        };
        let day = world.vt_trading_day.clone();
        let accounts = startup_accounts;
        let broker = world.cfg.broker_id.clone();
        Self::bootstrap_accounts(
            &mut world.ledger,
            world.engine.catalog(),
            &broker,
            &accounts,
            &day,
        )
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("scenario accounts: {e}"),
            )
        })?;
        world.ledger.refresh(world.engine.catalog());
        world.eval_assertions();
        Ok(world)
    }

    /// Validate every scenario account first, then apply them all; a bad row
    /// leaves `ledger` untouched. Shared by startup and `start_scenario`.
    fn bootstrap_accounts(
        ledger: &mut Ledger,
        catalog: &Catalog,
        broker: &str,
        accounts: &[scenario::AccountSpec],
        day: &str,
    ) -> Result<(), String> {
        for account in accounts {
            if !account.positions.is_empty() && ledger.account(broker, &account.investor).is_some()
            {
                return Err(format!(
                    "账户 {} 已存在，不能重复导入初始持仓",
                    account.investor
                ));
            }
            for p in &account.positions {
                let Some(inst) = catalog.get(&p.instrument) else {
                    return Err(format!("未知合约 {}", p.instrument));
                };
                if inst.exchange_id != p.exchange {
                    return Err(format!(
                        "合约 {} 的交易所错误: {}",
                        p.instrument, p.exchange
                    ));
                }
                if p.open_date.as_str() >= day {
                    return Err(format!(
                        "初仓 {} 的 open_date 必须早于交易日 {}",
                        p.trade_id, day
                    ));
                }
                if !p.open_price.is_finite() || !p.pre_settlement.is_finite() {
                    return Err(format!("初仓 {} 含非有限价格", p.trade_id));
                }
            }
        }
        for account in accounts {
            ledger.ensure_account_with(broker, &account.investor, account.balance.unwrap_or(0.0));
            for p in &account.positions {
                let side = if p.direction == "long" {
                    ctpbuddy_ledger::PositionSide::Long
                } else {
                    ctpbuddy_ledger::PositionSide::Short
                };
                let direction = if side == ctpbuddy_ledger::PositionSide::Long {
                    ctpbuddy_matching::Direction::Buy
                } else {
                    ctpbuddy_matching::Direction::Sell
                };
                let margin = p.margin.unwrap_or_else(|| {
                    catalog.margin(
                        &p.instrument,
                        direction,
                        MarginPrice::PreSettlement(p.pre_settlement),
                        p.volume,
                    )
                });
                let multiple = catalog.get(&p.instrument).unwrap().volume_multiple;
                let pos =
                    ledger.position_mut_or_create(broker, &account.investor, &p.instrument, side);
                pos.pre_settlement_price = p.pre_settlement;
                pos.add_bootstrap_detail(
                    &p.open_date,
                    &p.trade_id,
                    p.open_price,
                    p.volume,
                    margin,
                    p.pre_settlement,
                    multiple,
                );
            }
        }
        Ok(())
    }

    /// Load a scenario into the world (admin `start_scenario`): swap catalog
    /// and tick stream, reset today's order state, create scenario accounts,
    /// install assertions. Returns (tick count, trading day, scenario name).
    pub(crate) fn apply_scenario(
        &mut self,
        dir: &str,
        spec: Option<&scenario::Spec>,
        paused: bool,
        speed: Option<f64>,
    ) -> Result<(usize, String, String), String> {
        // Swapping the engine would silently drop resting orders while their
        // ledger freezes stay booked; refuse instead (cancel them first).
        if self.engine.open_order_count() != 0 {
            return Err("加载场景前必须先撤销全部活动订单".to_string());
        }
        let (catalog, ticks) = build_scenario(dir, spec, self.cfg.refdata_dir.as_deref())?;
        if ticks.is_empty() {
            return Err("场景没有任何 tick（transforms / clock.start 之后为空）".to_string());
        }
        let n = ticks.len();
        let day = ticks[0].trading_day.clone();
        let t0 = ticks[0].virtual_ms();
        let name = spec.map(|s| s.name.clone()).unwrap_or_default();

        let mut staged_ledger = self.ledger.clone();
        Self::bootstrap_accounts(
            &mut staged_ledger,
            &catalog,
            &self.cfg.broker_id,
            spec.map(|s| s.accounts.as_slice()).unwrap_or(&[]),
            &day,
        )?;
        staged_ledger.refresh(&catalog);

        self.engine = MatchingEngine::new(catalog);
        self.engine
            .set_self_trade_prevention(self.cfg.self_trade_prevention);
        self.orders_today.clear();
        self.trades_today.clear();
        // explicit speed > scenario clock.time_scale > server default
        let speed = speed
            .or(spec.and_then(|s| s.time_scale))
            .or(self.cfg.playback_speed)
            .unwrap_or(0.0);
        let mut pb = Playback::new(ticks, speed);
        if paused {
            pb.pause();
        }
        self.playback = Some(pb);
        self.vt_trading_day = day.clone();
        self.vt_now_ms = t0;
        self.scenario_name = name.clone();
        self.scenario_t0_ms = Some(t0);
        self.assertions = spec.map(|s| s.assertions.clone()).unwrap_or_default();

        self.ledger = staged_ledger;
        self.ledger.refresh(self.engine.catalog());
        self.eval_assertions();
        Ok((n, day, name))
    }

    /// Current metric value for a scenario assertion (DESIGN §7.4).
    ///
    /// `Err` carries the journaled `status` reason: the account does not exist
    /// yet (`account_not_found`) or the metric name is unknown
    /// (`unknown_metric`). Either way the assertion fails; the caller never
    /// has a numeric `actual` to serialize.
    fn assertion_actual(
        &self,
        broker: &str,
        investor: &str,
        metric: &str,
    ) -> Result<f64, &'static str> {
        let acct = |f: &dyn Fn(&ctpbuddy_ledger::Account) -> f64| -> Result<f64, &'static str> {
            self.ledger
                .account(broker, investor)
                .map(f)
                .ok_or(ASSERTION_ACCOUNT_NOT_FOUND)
        };
        Ok(match metric {
            "balance" => acct(&|a| a.dynamic_equity())?,
            "available" => acct(&|a| a.available())?,
            "close_profit" => acct(&|a| a.close_profit)?,
            "commission" => acct(&|a| a.commission)?,
            "position_profit" => acct(&|a| a.position_profit)?,
            "used_margin" => acct(&|a| a.used_margin)?,
            "frozen_margin" => acct(&|a| a.frozen_margin)?,
            "orders_filled" => {
                // distinct orders with at least one fill today
                let mut seen = HashSet::new();
                for o in &self.orders_today {
                    if cstr(&o.InvestorID) == investor && o.VolumeTraded > 0 {
                        seen.insert(cstr(&o.OrderSysID));
                    }
                }
                seen.len() as f64
            }
            "open_orders" => self
                .engine
                .active_order_fields("")
                .iter()
                .filter(|o| cstr(&o.InvestorID) == investor)
                .count() as f64,
            "fills" => self
                .trades_today
                .iter()
                .filter(|t| cstr(&t.InvestorID) == investor)
                .count() as f64,
            _ => return Err(ASSERTION_UNKNOWN_METRIC),
        })
    }

    /// (passed, failed) among evaluated assertions (admin status).
    pub(crate) fn assertion_counts(&self) -> (usize, usize) {
        let mut passed = 0;
        let mut failed = 0;
        for a in &self.assertions {
            if a.evaluated {
                if a.pass {
                    passed += 1;
                } else {
                    failed += 1;
                }
            }
        }
        (passed, failed)
    }

    /// Assertion summary for admin status (items carry the one-shot result).
    pub(crate) fn assertions_status(&self) -> Value {
        let (passed, failed) = self.assertion_counts();
        let evaluated = passed + failed;
        let items: Vec<Value> = self
            .assertions
            .iter()
            .map(|a| {
                json::obj_sorted(vec![
                    ("after_ms".into(), json::n(a.after_ms)),
                    ("investor".into(), json::s(&a.investor)),
                    ("metric".into(), json::s(&a.metric)),
                    ("op".into(), json::s(a.op.symbol())),
                    ("value".into(), json::n(a.value)),
                    (
                        "actual".into(),
                        if a.actual.is_finite() {
                            json::n(a.actual)
                        } else {
                            Value::Null
                        },
                    ),
                    ("pass".into(), json::b(a.pass)),
                    ("evaluated".into(), json::b(a.evaluated)),
                ])
            })
            .collect();
        json::obj_sorted(vec![
            ("total".into(), json::n(self.assertions.len() as f64)),
            ("evaluated".into(), json::n(evaluated as f64)),
            ("passed".into(), json::n(passed as f64)),
            ("failed".into(), json::n(failed as f64)),
            ("items".into(), Value::Arr(items)),
        ])
    }

    /// Evaluate due scenario assertions. One-shot per assertion: it fires at
    /// the first pulse where the virtual clock passes `t0 + after_ms` and is
    /// journaled exactly once (determinism for §7.6 / M2-3 hash replay).
    fn eval_assertions(&mut self) {
        if self.assertions.is_empty() {
            return;
        }
        let vt = self.vt_now_ms;
        let t0 = self.scenario_t0_ms.unwrap_or(vt);
        let broker = self.cfg.broker_id.clone();
        let due: Vec<usize> = self
            .assertions
            .iter()
            .enumerate()
            .filter(|(_, a)| !a.evaluated && vt >= t0 + a.after_ms)
            .map(|(i, _)| i)
            .collect();
        for i in due {
            let (investor, metric, after_ms) = {
                let a = &self.assertions[i];
                (a.investor.clone(), a.metric.clone(), a.after_ms)
            };
            // A metric that cannot be evaluated (no such account yet, unknown
            // name) fails with a reason; `actual` is then `null`, never a bare
            // `NaN` token that would make the journal line invalid JSON.
            let (actual, status) = match self.assertion_actual(&broker, &investor, &metric) {
                Ok(v) if v.is_finite() => (v, ASSERTION_OK),
                Ok(_) => (f64::NAN, ASSERTION_NOT_FINITE),
                Err(reason) => (f64::NAN, reason),
            };
            let (op_sym, value, pass) = {
                let a = &mut self.assertions[i];
                a.actual = actual;
                a.pass = status == ASSERTION_OK && a.op.apply(actual, a.value);
                a.evaluated = true;
                (a.op.symbol(), a.value, a.pass)
            };
            self.journal_record_json(
                "assertion",
                &broker,
                &investor,
                Self::assertion_result_json(&metric, op_sym, value, actual, pass, status, after_ms),
            );
            if !pass {
                eprintln!(
                    "[ctpbuddy] assertion FAILED: {investor} {metric} {op_sym} {value} (actual {actual}, status {status})"
                );
            }
        }
    }

    /// Journal payload of one evaluated assertion. `actual` is written as
    /// `null` when non-finite so the line stays valid JSON; `status` says why
    /// (`ok` / `account_not_found` / `unknown_metric` / `not_finite`).
    fn assertion_result_json(
        metric: &str,
        op_sym: &str,
        value: f64,
        actual: f64,
        pass: bool,
        status: &str,
        after_ms: f64,
    ) -> Value {
        json::obj_sorted(vec![
            ("metric".into(), json::s(metric)),
            ("op".into(), json::s(op_sym)),
            ("value".into(), json::n(value)),
            (
                "actual".into(),
                if actual.is_finite() {
                    json::n(actual)
                } else {
                    Value::Null
                },
            ),
            ("pass".into(), json::b(pass)),
            ("ok".into(), json::b(status == ASSERTION_OK)),
            ("status".into(), json::s(status)),
            ("after_ms".into(), json::n(after_ms)),
        ])
    }

    fn run_loop(&mut self, rx: Receiver<WorldMsg>) {
        // The pulse is a true timer, not an idle timeout: it must fire every
        // PULSE even under sustained request load. (A client polling `status`
        // faster than PULSE would otherwise starve the world clock — no tick
        // release, no md_watermark, no journal progress.)
        let mut last_pulse = Instant::now();
        loop {
            let mut wait = PULSE.saturating_sub(last_pulse.elapsed());
            if wait.is_zero() {
                self.pulse();
                last_pulse = Instant::now();
                wait = PULSE;
            }
            match rx.recv_timeout(wait) {
                Ok(msg) => match msg {
                    WorldMsg::Opened {
                        id,
                        writer,
                        is_admin,
                    } => self.on_conn_opened(id, writer, is_admin),
                    WorldMsg::Inbound { id, frame } => self.on_frame(id, frame),
                    WorldMsg::Closed { id } => self.on_conn_closed(id),
                },
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if self.shutdown {
                break;
            }
        }
    }

    /// Timer pulse: release due ticks, settle fills, mark to market, flush journal.
    fn pulse(&mut self) {
        let now = Instant::now();
        let (ticks, vtime) = match self.playback.as_mut() {
            Some(pb) => {
                let t = pb.poll(now);
                (t, pb.virtual_time())
            }
            None => (Vec::new(), 0.0),
        };
        if !ticks.is_empty() {
            self.vt_now_ms = vtime;
        }
        for tick in &ticks {
            self.push_market_data(tick);
            let events = self.engine.on_tick(tick);
            for ev in events {
                self.dispatch_event(ev);
            }
        }
        // scenario assertions fire at deterministic virtual times (§7.4)
        self.eval_assertions();
        if !ticks.is_empty() {
            self.journal_record_json(
                "md_watermark",
                "",
                "",
                json::obj_sorted(vec![("idx".into(), json::n(self.playback_progress()))]),
            );
        }
        // mark to market (unrealized PnL drives account queries)
        let prices = self.engine.last_prices();
        let pre_settlements = self.engine.pre_settlements();
        self.ledger.mark_to_market(
            self.engine.catalog(),
            &prices,
            &pre_settlements,
            &self.vt_trading_day,
        );
        if let Some(j) = self.journal.as_mut() {
            j.flush_if_due(now);
        }
    }

    fn push_market_data(&mut self, tick: &Tick) {
        let field = tick.to_depth_md();
        let frame = Frame::new(msgs::RTN_DEPTH_MD, 0, struct_to_bytes(&field));
        let subs: Vec<u64> = self
            .conns
            .iter()
            .filter(|(_, c)| c.md_subs.contains(&tick.instrument_id))
            .map(|(id, _)| *id)
            .collect();
        for id in subs {
            self.send_frame(id, frame.clone());
        }
    }

    /// Route an engine event: ledger settles first, then the wire push.
    fn dispatch_event(&mut self, ev: EngineEvent) {
        match ev {
            EngineEvent::Order(field) => {
                // A terminal '5' (client cancel or IOC/FOK/FAK auto-cancel)
                // releases whatever is still frozen for the order — §8.3:
                // 撤单按 VolumeTotal（未成交量）解冻. `unfreeze_order` is a
                // no-op when the freeze was already fully released by fills.
                if field.OrderStatus == b'5' {
                    let key = format!(
                        "{}/{}/{}",
                        field.FrontID,
                        field.SessionID,
                        cstr(&field.OrderRef)
                    );
                    self.ledger.unfreeze_order(&key, self.engine.catalog());
                }
                self.orders_today.push(field);
                let frame = Frame::new(msgs::RTN_ORDER, 0, struct_to_bytes(&field));
                let targets = self.investor_conns(&cstr(&field.BrokerID), &cstr(&field.InvestorID));
                for id in targets {
                    self.send_frame(id, frame.clone());
                }
                let day = self.vt_trading_day.clone();
                let now = self.vt_now_ms;
                self.journal_record_order_update(&day, now, &field);
            }
            EngineEvent::Trade { field, fill } => {
                let pre_settle = self
                    .engine
                    .pre_settlement(&fill.instrument_id)
                    .unwrap_or(0.0);
                self.ledger.on_fill(
                    &fill,
                    self.engine.catalog(),
                    pre_settle,
                    &self.vt_trading_day,
                );
                self.trades_today.push(field);
                let frame = Frame::new(msgs::RTN_TRADE, 0, struct_to_bytes(&field));
                let targets = self.investor_conns(&cstr(&field.BrokerID), &cstr(&field.InvestorID));
                for id in targets {
                    self.send_frame(id, frame.clone());
                }
                let day = self.vt_trading_day.clone();
                let now = self.vt_now_ms;
                self.journal_record_fill(&day, now, &fill);
            }
        }
    }

    fn on_conn_opened(&mut self, id: u64, writer: Sender<Frame>, is_admin: bool) {
        self.conns.insert(
            id,
            Conn {
                writer,
                is_admin,
                authenticated: false,
                front_id: id as i32,
                session_id: 0,
                logins: 0,
                broker_id: None,
                investor_id: None,
                user_id: [0u8; 16],
                order_local_seq: 0,
                md_subs: HashSet::new(),
                qry_window: None,
                qry_count: 0,
            },
        );
    }

    fn on_conn_closed(&mut self, id: u64) {
        self.conns.remove(&id);
    }

    // ---- helpers shared by handlers / admin ----

    pub(crate) fn send_frame(&mut self, conn_id: u64, frame: Frame) {
        if let Some(c) = self.conns.get(&conn_id) {
            let _ = c.writer.send(frame);
        }
    }

    pub(crate) fn send_error(&mut self, conn_id: u64, req_id: u32, error_id: i32, msg: &str) {
        let mut f = ctpbuddy_wire::generated::CThostFtdcRspInfoField::zeroed();
        f.ErrorID = error_id;
        set_cstr(&mut f.ErrorMsg, msg);
        self.send_frame(
            conn_id,
            Frame::new(msgs::RSP_ERROR, req_id, struct_to_bytes(&f)),
        );
    }

    /// All connections logged in as (broker, investor) — the `td/{broker}/{investor}`
    /// topic fan-out.
    pub(crate) fn investor_conns(&self, broker_id: &str, investor_id: &str) -> Vec<u64> {
        self.conns
            .iter()
            .filter(|(_, c)| {
                c.broker_id.as_deref() == Some(broker_id)
                    && c.investor_id.as_deref() == Some(investor_id)
            })
            .map(|(id, _)| *id)
            .collect()
    }

    pub(crate) fn vt_day(&self) -> String {
        self.vt_trading_day.clone()
    }

    /// Journal: raw event. Callers build the JSON payload.
    pub(crate) fn journal_record_json(
        &mut self,
        event_type: &str,
        broker_id: &str,
        investor_id: &str,
        data: Value,
    ) {
        let day = self.vt_trading_day.clone();
        let now = self.vt_now_ms;
        if let Some(j) = self.journal.as_mut() {
            j.record(&day, now, event_type, broker_id, investor_id, data);
        }
    }

    fn playback_progress(&self) -> f64 {
        self.playback
            .as_ref()
            .map(|p| p.progress().0 as f64)
            .unwrap_or(0.0)
    }

    fn journal_record_order_update(&mut self, day: &str, ms: f64, field: &CThostFtdcOrderField) {
        let data = json::obj_sorted(vec![
            ("order_ref".into(), json::s(&cstr(&field.OrderRef))),
            ("order_sys_id".into(), json::s(&cstr(&field.OrderSysID))),
            ("instrument".into(), json::s(&cstr(&field.InstrumentID))),
            (
                "status".into(),
                json::s(&(field.OrderStatus as char).to_string()),
            ),
            ("volume_traded".into(), json::n(field.VolumeTraded as f64)),
            ("volume_total".into(), json::n(field.VolumeTotal as f64)),
            ("update_time".into(), json::s(&cstr(&field.UpdateTime))),
        ]);
        if let Some(j) = self.journal.as_mut() {
            j.record(
                day,
                ms,
                "order_update",
                &cstr(&field.BrokerID),
                &cstr(&field.InvestorID),
                data,
            );
        }
    }

    fn journal_record_fill(&mut self, day: &str, ms: f64, fill: &ctpbuddy_matching::Fill) {
        let data = json::obj_sorted(vec![
            ("trade_id".into(), json::s(&cstr(&fill.trade_id))),
            ("order_sys_id".into(), json::s(&cstr(&fill.order_sys_id))),
            ("order_ref".into(), json::s(&cstr(&fill.order_ref))),
            ("instrument".into(), json::s(&fill.instrument_id)),
            ("direction".into(), json::n(fill.direction.as_ctp() as f64)),
            ("offset".into(), json::n(fill.offset.as_ctp() as f64)),
            ("price".into(), json::n(fill.price)),
            ("volume".into(), json::n(fill.volume as f64)),
        ]);
        if let Some(j) = self.journal.as_mut() {
            j.record(
                day,
                ms,
                "fill",
                &cstr(&fill.broker_id),
                &cstr(&fill.investor_id),
                data,
            );
        }
    }

    /// Virtual-clock snapshot as owned values (engine calls take `&str`).
    pub(crate) fn clock_owned(&self) -> (String, f64) {
        (self.vt_trading_day.clone(), self.vt_now_ms)
    }

    pub(crate) fn set_shutdown(&mut self) {
        self.shutdown = true;
    }

    pub(crate) fn now_str(&self) -> String {
        format_hhmmss(self.vt_now_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assertion_journal_line_is_valid_json_when_actual_is_nan() {
        let v = World::assertion_result_json(
            "balance",
            ">=",
            100.0,
            f64::NAN,
            false,
            ASSERTION_ACCOUNT_NOT_FOUND,
            0.0,
        );
        let text = v.to_json();
        assert!(!text.contains("NaN"), "{text}");
        let back = json::parse(&text).expect("journal payload must be valid JSON");
        assert!(matches!(back.get("actual"), Some(Value::Null)));
        assert_eq!(back.get_str("status").as_deref(), Some("account_not_found"));
        assert!(matches!(back.get("ok"), Some(Value::Bool(false))));
        assert!(matches!(back.get("pass"), Some(Value::Bool(false))));

        let v = World::assertion_result_json("fills", "==", 1.0, 1.0, true, ASSERTION_OK, 0.0);
        let back = json::parse(&v.to_json()).unwrap();
        assert_eq!(back.get_num("actual"), Some(1.0));
        assert_eq!(back.get_str("status").as_deref(), Some("ok"));
        assert!(matches!(back.get("ok"), Some(Value::Bool(true))));
    }

    #[test]
    fn explicit_refdata_load_failure_is_a_hard_error() {
        let missing = std::env::temp_dir().join("ctpbuddy-no-such-refdata-dir");
        let err = load_refdata(Some(missing.to_str().unwrap()), None).unwrap_err();
        assert!(err.starts_with("--refdata "), "{err}");
        // no explicit dir and no scenario dir -> bundled snapshot, never an error
        assert!(load_refdata(None, None).is_ok());
    }
}
