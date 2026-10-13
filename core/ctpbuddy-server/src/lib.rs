//! ctpbuddy-server: the single-threaded world loop plus its TCP/ZeroMQ
//! transports.
//!
//! TCP remains the default and ADMIN transport. A `zmq://` TD endpoint uses a
//! ROUTER/DEALER adapter; both transports carry the same CB frames, so the
//! protocol and world semantics are independent of the transport.

pub mod admin;
mod auto_settlement;
pub mod calendar;
pub mod dtime;
mod flow;
mod flow_replay;
pub mod handlers;
pub mod journal;
pub mod journal_contract;
pub mod json;
mod live;
mod private_flow;
pub mod scenario;
mod security;
pub mod settings;
mod settlement;
pub mod state;
mod transport_zmq;
mod transport_zmq_native;

use std::collections::{HashMap, HashSet, VecDeque};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use ctpbuddy_ledger::{Ledger, PositionSide};
use ctpbuddy_market::{format_hhmmss, CsvSource, Playback, Tick};
use ctpbuddy_matching::{
    Catalog, Direction, EngineEvent, Fill, MarginPrice, MatchingEngine, OffsetFlag,
};
use ctpbuddy_wire::frame::Frame;
use ctpbuddy_wire::generated::{cstr, set_cstr, CThostFtdcOrderField, CThostFtdcTradeField};
use ctpbuddy_wire::msgs;
use ctpbuddy_wire::{struct_from_bytes, struct_to_bytes};

use journal::Journal;
use json::Value;
use serde_json::Value as SerdeValue;

pub const SERVER_NAME: &str = "CTPBuddy";
pub const SERVER_VERSION: &str = "0.1.0";
/// How often a changed ledger is written to its recovery snapshot.
const SNAPSHOT_EVERY: Duration = Duration::from_secs(5);
/// World-loop pulse period: playback tick release + mark-to-market cadence.
pub const PULSE: Duration = Duration::from_millis(10);

/// Scenario assertion `status` values written to the journal (DESIGN §7.4).
const ASSERTION_OK: &str = "ok";
const ASSERTION_ACCOUNT_NOT_FOUND: &str = "account_not_found";
const ASSERTION_UNKNOWN_METRIC: &str = "unknown_metric";
const ASSERTION_NOT_FINITE: &str = "not_finite";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarMetadata {
    pub version: String,
    pub sha256: String,
}

/// Build the explicit calendar provenance accepted by the core.
pub fn calendar_metadata(
    version: Option<String>,
    sha256: Option<String>,
) -> Result<Option<CalendarMetadata>, String> {
    match (version, sha256) {
        (None, None) => Ok(None),
        (Some(version), Some(sha256))
            if !version.trim().is_empty()
                && sha256.len() == 64
                && sha256.bytes().all(|b| b.is_ascii_hexdigit()) =>
        {
            Ok(Some(CalendarMetadata {
                version: version.trim().to_string(),
                sha256: sha256.to_ascii_lowercase(),
            }))
        }
        (Some(_), None) | (None, Some(_)) => {
            Err("calendar_version 与 calendar_sha256 必须成对提供".into())
        }
        (Some(_), Some(_)) => Err("calendar version/sha256 无效".into()),
    }
}

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

/// Id the next accepted connection will receive (never reused).
pub(crate) fn next_conn_id() -> u64 {
    NEXT_CONN_ID.load(Ordering::SeqCst)
}

pub(crate) fn allocate_conn_id() -> u64 {
    NEXT_CONN_ID.fetch_add(1, Ordering::SeqCst)
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
    /// BrokerIDs served by this core instance. A comma-separated value enables
    /// a shared world for multiple brokers; the first ID remains the default
    /// journal/settlement namespace for backward compatibility.
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
    /// Restore the ledger from `<data_dir>/state/ledger.json` at startup
    /// (`--recover` / `CTPBUDDY_RECOVER=1`). Working orders are not restored.
    pub recover: bool,
    /// Explicit offline calendar provenance; the core does not infer or load
    /// the Python calendar snapshot itself.
    pub calendar: Option<CalendarMetadata>,
    /// Parsed offline calendar snapshot used for core-side TradingDay derivation.
    pub calendar_snapshot: Option<calendar::CalendarSnapshot>,
    /// Optional wall-clock auto-settlement trigger in `HH:MM` (China time).
    /// Settlement prices remain explicit and are never inferred.
    pub auto_settle_at: Option<String>,
    pub auto_settle_prices: Option<String>,
    /// Front total-message flow budget per connection, per second. Zero disables.
    pub ftd_max_comm_flux: u32,
    /// New connection acceptance budget per second. Zero disables.
    pub connect_freq: u32,
    /// Exchange-level order budget per second. Zero disables.
    pub exchange_freq: u32,
    pub exchange_limits: HashMap<String, u32>,
    /// Explicit offline replay mode; live TD inputs are refused.
    pub flow_replay: bool,
    /// Stop the world loop after a journal write failure: the ledger applies
    /// events before they are journaled, so running on would silently diverge
    /// from the authoritative stream. `CTPBUDDY_JOURNAL_HARD_FAIL=1`.
    pub journal_hard_fail: bool,
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
            recover: false,
            calendar: None,
            calendar_snapshot: None,
            auto_settle_at: None,
            auto_settle_prices: None,
            ftd_max_comm_flux: 0,
            connect_freq: 0,
            exchange_freq: 0,
            exchange_limits: HashMap::new(),
            flow_replay: false,
            journal_hard_fail: false,
        }
    }
}

impl Config {
    pub fn supports_broker(&self, broker: &str) -> bool {
        self.broker_id
            .split(',')
            .map(str::trim)
            .any(|id| !id.is_empty() && id == broker)
    }

    pub fn broker_ids(&self) -> Vec<&str> {
        self.broker_id
            .split(',')
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .collect()
    }

    pub fn primary_broker(&self) -> &str {
        self.broker_ids().into_iter().next().unwrap_or("")
    }
}

/// Bind transports and run the world loop until an admin `shutdown`.
pub fn run(mut cfg: Config) -> std::io::Result<()> {
    settings::load(&mut cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    // Remote ADMIN requires both an explicit bind opt-in and a token.
    // TCP ADMIN remains plaintext and must use a trusted tunnel across hosts.
    ensure_admin_loopback(&cfg.admin_endpoint)?;

    let admin = TcpListener::bind(&cfg.admin_endpoint)?;
    let mut world = World::new(cfg.clone())?;
    let (tx, rx) = mpsc::channel::<WorldMsg>();

    if cfg.td_endpoint.starts_with("zmq://") {
        let pub_tx = transport_zmq::start(
            &cfg.td_endpoint,
            tx.clone(),
            cfg.connect_freq,
            world.security.curve_secret.clone(),
            world.security.curve_clients.clone(),
        )?;
        world.broadcast_tx = pub_tx;
    } else {
        let td = TcpListener::bind(&cfg.td_endpoint)?;
        let tx_td = tx.clone();
        let connect_freq = cfg.connect_freq;
        thread::spawn(move || accept_loop(td, tx_td, false, connect_freq));
    }
    let tx_admin = tx.clone();
    thread::spawn(move || accept_loop(admin, tx_admin, true, 0));
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

/// Env opt-out for [`ensure_admin_loopback`].
pub const ALLOW_REMOTE_ADMIN_ENV: &str = "CTPBUDDY_ALLOW_REMOTE_ADMIN";
/// Optional ZeroMQ PUB endpoint for market/private broadcasts.
pub const ZMQ_PUB_ADDR_ENV: &str = "CTPBUDDY_ZMQ_PUB_ADDR";
/// Bounded queue between the world thread and the ZeroMQ PUB writer.
pub const ZMQ_PUB_QUEUE_CAPACITY: usize = 1024;
pub const ADMIN_TOKEN_ENV: &str = "CTPBUDDY_ADMIN_TOKEN";
pub const ZMQ_CURVE_SERVER_KEY_ENV: &str = "CTPBUDDY_ZMQ_CURVE_SERVER_KEY";
pub const ZMQ_CURVE_PUBLIC_KEY_ENV: &str = "CTPBUDDY_ZMQ_CURVE_PUBLIC_KEY";
pub const ZMQ_CURVE_SECRET_KEY_ENV: &str = "CTPBUDDY_ZMQ_CURVE_SECRET_KEY";
pub const AUTO_SETTLE_AT_ENV: &str = "CTPBUDDY_AUTO_SETTLE_AT";
pub const AUTO_SETTLE_PRICES_ENV: &str = "CTPBUDDY_AUTO_SETTLE_PRICES";
pub const FTD_MAX_COMM_FLUX_ENV: &str = "CTPBUDDY_FTD_MAX_COMM_FLUX";
pub const CONNECT_FREQ_ENV: &str = "CTPBUDDY_CONNECT_FREQ";
pub const EXCHANGE_FREQ_ENV: &str = "CTPBUDDY_EXCHANGE_FREQ";

/// Err unless every address `endpoint` resolves to is loopback (or the
/// operator set `CTPBUDDY_ALLOW_REMOTE_ADMIN=1`).
pub fn ensure_admin_loopback(endpoint: &str) -> std::io::Result<()> {
    use std::net::ToSocketAddrs;
    if std::env::var(ALLOW_REMOTE_ADMIN_ENV).is_ok_and(|v| v == "1") {
        if security::configured_secret(ADMIN_TOKEN_ENV)?.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "remote ADMIN opt-in requires CTPBUDDY_ADMIN_TOKEN",
            ));
        }
        return Ok(());
    }
    let addrs: Vec<_> = endpoint.to_socket_addrs()?.collect();
    if addrs.is_empty() || addrs.iter().any(|a| !a.ip().is_loopback()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "refusing to bind unauthenticated ADMIN to non-loopback {endpoint}; \
                 use 127.0.0.1 or set {ALLOW_REMOTE_ADMIN_ENV}=1"
            ),
        ));
    }
    Ok(())
}

/// Env var: `1` makes resting limit orders fill at their own limit price
/// (exchange maker semantics) rather than the better tick-level price.
pub const MAKER_AT_LIMIT_ENV: &str = "CTPBUDDY_MAKER_AT_LIMIT";

fn maker_at_limit_enabled() -> bool {
    matches!(
        std::env::var(MAKER_AT_LIMIT_ENV)
            .ok()
            .as_deref()
            .map(str::trim),
        Some("1") | Some("true")
    )
}

/// Env var overriding the concurrent-connection cap.
pub const MAX_CONNS_ENV: &str = "CTPBUDDY_MAX_CONNS";
const DEFAULT_MAX_CONNS: usize = 256;
/// A peer that stops reading for this long is dropped (bounds the writer thread).
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// Per-listener connection limits (injectable so tests don't share globals).
#[derive(Clone)]
struct ConnLimits {
    max_conns: usize,
    connect_freq: u32,
    write_timeout: Duration,
    live: Arc<std::sync::atomic::AtomicUsize>,
}

static SHARED_CONN_COUNT: std::sync::OnceLock<Arc<std::sync::atomic::AtomicUsize>> =
    std::sync::OnceLock::new();

pub(crate) fn shared_connection_counter() -> Arc<std::sync::atomic::AtomicUsize> {
    SHARED_CONN_COUNT
        .get_or_init(|| Arc::new(std::sync::atomic::AtomicUsize::new(0)))
        .clone()
}

pub(crate) fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
}

pub fn validate_auto_settle_at(value: &str) -> Result<String, String> {
    let mut parts = value.trim().split(':');
    let hour = parts.next().and_then(|v| v.parse::<u32>().ok());
    let minute = parts.next().and_then(|v| v.parse::<u32>().ok());
    if parts.next().is_some() || hour.is_none() || minute.is_none() {
        return Err("auto settle time must be HH:MM".into());
    }
    let (hour, minute) = (hour.unwrap(), minute.unwrap());
    if hour > 23 || minute > 59 {
        return Err("auto settle time must be within 00:00..23:59".into());
    }
    Ok(format!("{hour:02}:{minute:02}"))
}

pub(crate) fn configured_max_conns() -> usize {
    parse_max_conns(std::env::var(MAX_CONNS_ENV).ok().as_deref())
}

pub(crate) fn parse_max_conns(v: Option<&str>) -> usize {
    v.and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(DEFAULT_MAX_CONNS)
}

/// Decrements the live-connection counter when the reader thread ends.
struct ConnGuard(Arc<std::sync::atomic::AtomicUsize>);
impl Drop for ConnGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

fn accept_loop(listener: TcpListener, tx: Sender<WorldMsg>, is_admin: bool, connect_freq: u32) {
    // The counter is process-global so TCP and ZeroMQ listeners share one cap.
    let live = shared_connection_counter();
    let limits = ConnLimits {
        max_conns: configured_max_conns(),
        connect_freq,
        write_timeout: WRITE_TIMEOUT,
        live,
    };
    accept_loop_with(listener, tx, is_admin, limits);
}

fn accept_loop_with(listener: TcpListener, tx: Sender<WorldMsg>, is_admin: bool, lim: ConnLimits) {
    let max_conns = lim.max_conns;
    let mut connections = flow::WindowBudget::default();
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                if !flow::connect_gate(
                    &mut connections,
                    Instant::now(),
                    lim.connect_freq,
                    &tx,
                    "tcp",
                ) {
                    continue;
                }
                if lim.live.fetch_add(1, Ordering::SeqCst) >= max_conns {
                    lim.live.fetch_sub(1, Ordering::SeqCst);
                    eprintln!("[ctpbuddy] connection limit {max_conns} reached, rejecting peer");
                    continue; // dropping `s` closes it
                }
                let guard = ConnGuard(lim.live.clone());
                let _ = s.set_nodelay(true);
                let _ = s.set_write_timeout(Some(lim.write_timeout));
                let id = allocate_conn_id();
                let (writer_tx, writer_rx) = mpsc::channel::<Frame>();
                let _ = tx.send(WorldMsg::Opened {
                    id,
                    writer: writer_tx,
                    is_admin,
                });
                let reader = match s.try_clone() {
                    Ok(r) => r,
                    Err(_) => {
                        let _ = tx.send(WorldMsg::Closed { id });
                        continue;
                    }
                };
                let dead = Arc::new(AtomicBool::new(false));
                let tx_r = tx.clone();
                let dead_r = dead.clone();
                thread::spawn(move || {
                    let _guard = guard;
                    reader_loop(id, reader, tx_r, dead_r)
                });
                let tx_w = tx.clone();
                thread::spawn(move || writer_loop(id, s, writer_rx, tx_w, dead));
            }
            Err(e) => {
                eprintln!("[ctpbuddy] accept error: {e}");
                break;
            }
        }
    }
}

/// How often an idle reader re-checks whether its writer gave up.
const READER_POLL: Duration = Duration::from_millis(500);

/// Reader that rides out the poll timeout mid-frame (a slow sender is never
/// cut off) but gives up once the connection has been flagged dead.
struct PatientReader<'a> {
    stream: &'a mut TcpStream,
    dead: &'a AtomicBool,
}

impl std::io::Read for PatientReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match self.stream.read(buf) {
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    if self.dead.load(Ordering::SeqCst) {
                        return Err(e);
                    }
                }
                other => return other,
            }
        }
    }
}

fn reader_loop(id: u64, mut stream: TcpStream, tx: Sender<WorldMsg>, dead: Arc<AtomicBool>) {
    // Set once per connection (not per frame).
    let _ = stream.set_read_timeout(Some(READER_POLL));
    loop {
        // Wait for the first byte with a short timeout so a connection whose
        // writer died is torn down even where `shutdown` does not wake a
        // blocked `recv` (observed on Windows). Mid-frame timeouts are retried
        // by `PatientReader`, so a slow sender is never cut off.
        let mut probe = [0u8; 1];
        match stream.peek(&mut probe) {
            Ok(0) => {
                let _ = tx.send(WorldMsg::Closed { id });
                break;
            }
            Ok(_) => {}
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                if dead.load(Ordering::SeqCst) {
                    let _ = tx.send(WorldMsg::Closed { id });
                    break;
                }
                continue;
            }
            Err(e) => {
                eprintln!("[ctpbuddy] conn {id} read error: {e}");
                let _ = tx.send(WorldMsg::Closed { id });
                break;
            }
        }
        let mut patient = PatientReader {
            stream: &mut stream,
            dead: &dead,
        };
        match Frame::read_from(&mut patient) {
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

fn writer_loop(
    id: u64,
    mut stream: TcpStream,
    rx: Receiver<Frame>,
    tx: Sender<WorldMsg>,
    dead: Arc<AtomicBool>,
) {
    // Ends when the world drops our sender (conn removed) or the socket dies.
    for frame in rx {
        if frame.write_to(&mut stream).is_err() {
            // Write timeout / broken pipe: flag the connection dead, shut the
            // socket and tell the world loop directly (`Closed` is idempotent),
            // so the connection is never left half-dead.
            dead.store(true, Ordering::SeqCst);
            let _ = stream.shutdown(std::net::Shutdown::Both);
            let _ = tx.send(WorldMsg::Closed { id });
            break;
        }
    }
}

pub(crate) enum WorldMsg {
    FlowConnect {
        scope: &'static str,
        clock: Instant,
        allowed: bool,
    },
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
    /// A transport failed after startup; stop the world instead of reporting
    /// a healthy core with a dead TD endpoint.
    TransportClosed,
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
    /// SubscribePrivateTopic resume type announced in AUTH: 0 RESTART /
    /// 1 RESUME / 2 QUICK (default: live events only, no replay).
    private_resume: u8,
    /// 查询流控 window (CTP docs: per-session QryFreq per second).
    qry_window: Option<Instant>,
    qry_count: u32,
    ftd_budget: flow::WindowBudget,
    ftd_pending: VecDeque<Frame>,
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
    security: security::Security,
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
    /// Private flow in emission order: `(is_trade, index)` into
    /// `trades_today` / `orders_today`. Source for RESTART/RESUME replay.
    private_seq: Vec<(bool, usize)>,
    /// Per (broker, investor): `private_seq` length at the last moment the
    /// investor had a live session receiving the flow (RESUME start point).
    private_cursor: HashMap<(String, String), usize>,
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
    exchange_freq_windows: HashMap<(String, String, String, GateStream), flow::WindowBudget>,
    flow_clock: Instant,
    flow_state: flow_replay::ReplayState,
    settlement_confirmed: HashMap<(String, String), String>,
    settlement_reports: Vec<settlement::Report>,
    /// Last ledger snapshot written to `<data_dir>/state/` and when.
    last_snapshot: Option<SerdeValue>,
    last_private_flow_snapshot: Option<SerdeValue>,
    /// Journal sequence covered by the last durable recovery snapshot.
    snapshot_journal_seq: Option<u64>,
    last_snapshot_at: Instant,
    shutdown: bool,
    broadcast_tx: Option<tokio::sync::mpsc::Sender<Broadcast>>,
    broadcast_published: u64,
    broadcast_dropped: u64,
    auto_settlement: auto_settlement::AutoSettlement,
}

pub(crate) struct Broadcast {
    pub(crate) topic: String,
    pub(crate) frame: Frame,
}

struct PrivateNotification {
    broker: String,
    investor: String,
    frame: Frame,
}

impl World {
    pub fn new(mut cfg: Config) -> std::io::Result<Self> {
        let security = security::Security::load(&cfg.td_endpoint)?;
        if cfg.exchange_limits.keys().any(|key| {
            !matches!(
                key.as_str(),
                "SHFE" | "INE" | "DCE" | "CZCE" | "CFFEX" | "GFEX"
            )
        }) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "exchange limits require SHFE/INE/DCE/CZCE/CFFEX/GFEX keys",
            ));
        }
        if let Some(at) = cfg.auto_settle_at.as_mut() {
            *at = validate_auto_settle_at(at)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        }
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
        // A scenario owns the virtual trading day.  Do not consult the
        // wall-clock calendar before loading it: a perfectly valid historical
        // scenario may be paired with a snapshot that only covers the
        // scenario's dates, rather than today's date.  Calendar derivation is
        // therefore reserved for scenario-less startup, where there is no
        // tick-authored TradingDay to take precedence.
        let mut vt_day = if cfg.scenario_dir.is_none() {
            if let Some(calendar) = cfg.calendar_snapshot.as_ref() {
                calendar
                    .trading_day_for_date(&dtime::today_natural_date())
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
            } else {
                dtime::today_trading_day()
            }
        } else {
            dtime::today_trading_day()
        };
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
            let mut start_data = vec![
                ("flow_config".into(), flow_replay::config(&cfg)),
                ("settings".into(), settings::values(&cfg)),
                ("version".into(), json::s(SERVER_VERSION)),
                (
                    "scenario".into(),
                    json::s(cfg.scenario_dir.as_deref().unwrap_or("")),
                ),
            ];
            if let Some(calendar) = &cfg.calendar {
                start_data.push(("calendar_version".into(), json::s(&calendar.version)));
                start_data.push(("calendar_sha256".into(), json::s(&calendar.sha256)));
            }
            j.record(
                &vt_day,
                vt_ms,
                "server_start",
                cfg.primary_broker(),
                "",
                json::obj_sorted(start_data),
            );
            j.flush()?;
        }

        let mut engine = MatchingEngine::new(engine_catalog);
        engine.set_self_trade_prevention(cfg.self_trade_prevention);
        engine.set_maker_fill_at_limit(maker_at_limit_enabled());
        let mut world = World {
            security,
            engine,
            ledger: Ledger::new(cfg.initial_funds),
            playback,
            journal,
            conns: HashMap::new(),
            vt_trading_day: vt_day,
            vt_now_ms: vt_ms,
            orders_today: Vec::new(),
            trades_today: Vec::new(),
            private_seq: Vec::new(),
            private_cursor: HashMap::new(),
            scenario_name,
            scenario_t0_ms,
            assertions,
            order_freq_windows: HashMap::new(),
            exchange_freq_windows: HashMap::new(),
            flow_clock: flow::origin(),
            flow_state: flow_replay::ReplayState::default(),
            settlement_confirmed: HashMap::new(),
            settlement_reports: settlement::load_reports(&cfg.data_dir, &cfg.broker_ids())
                .unwrap_or_else(|e| {
                    eprintln!("[ctpbuddy] settlement reports disabled: {e}");
                    Vec::new()
                }),
            last_snapshot: None,
            last_private_flow_snapshot: None,
            snapshot_journal_seq: None,
            last_snapshot_at: Instant::now(),
            shutdown: false,
            broadcast_tx: None,
            broadcast_published: 0,
            broadcast_dropped: 0,
            auto_settlement: auto_settlement::AutoSettlement::default(),
            cfg,
        };
        if world.cfg.recover && !world.cfg.data_dir.is_empty() {
            match state::read_with_flow_meta(&world.cfg.data_dir) {
                Ok(Some((day, ledger, private_flow, journal_seq))) => {
                    println!(
                        "[ctpbuddy] recovered ledger: {} account(s), trading day {day}",
                        ledger.accounts().count()
                    );
                    if world.playback.is_none() {
                        world.vt_trading_day = day.clone();
                    } else if world.vt_trading_day != day {
                        eprintln!(
                            "[ctpbuddy] WARNING: snapshot day {day} != scenario day {}",
                            world.vt_trading_day
                        );
                    }
                    world.ledger = ledger;
                    world.snapshot_journal_seq = journal_seq;
                    if world.vt_trading_day == day && !private_flow.is_null() {
                        if let Err(e) = world.restore_private_flow(&private_flow) {
                            eprintln!("[ctpbuddy] private-flow recovery skipped: {e}");
                        }
                    }
                    if let Some(seq) = journal_seq {
                        let data_dir = world.cfg.data_dir.clone();
                        world
                            .replay_archived_tail(&data_dir, seq)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                    }
                    // accounts and initial positions are already in the snapshot
                    startup_accounts.clear();
                    world.journal_record_json(
                        "ledger_recovered",
                        "",
                        "",
                        json::obj_sorted(vec![
                            ("trading_day".into(), json::s(&day)),
                            ("working_orders_voided".into(), json::b(true)),
                            (
                                "reason".into(),
                                json::s("服务重启，journal 中此前无终态的在途挂单作废，冻结已清零"),
                            ),
                            (
                                "snapshot_journal_seq".into(),
                                journal_seq
                                    .map(|seq| json::n(seq as f64))
                                    .unwrap_or(Value::Null),
                            ),
                        ]),
                    );
                }
                Ok(None) => eprintln!("[ctpbuddy] --recover: no snapshot found, starting fresh"),
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("ledger recovery failed: {e}"),
                    ))
                }
            }
        }
        let day = world.vt_trading_day.clone();
        let accounts = startup_accounts;
        let broker = world.cfg.primary_broker().to_string();
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
        world.persist_ledger();
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
                    catalog.margin_for(
                        broker,
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
                    ctpbuddy_ledger::Money::from_f64(margin),
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
            self.cfg.primary_broker(),
            spec.map(|s| s.accounts.as_slice()).unwrap_or(&[]),
            &day,
        )?;
        staged_ledger.refresh(&catalog);

        self.engine = MatchingEngine::new(catalog);
        self.engine
            .set_self_trade_prevention(self.cfg.self_trade_prevention);
        self.engine
            .set_maker_fill_at_limit(maker_at_limit_enabled());
        self.clear_day_flow();
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
            "balance" => acct(&|a| a.dynamic_equity().to_f64())?,
            "available" => acct(&|a| a.available().to_f64())?,
            "close_profit" => acct(&|a| a.close_profit.to_f64())?,
            "commission" => acct(&|a| a.commission.to_f64())?,
            "position_profit" => acct(&|a| a.position_profit.to_f64())?,
            "used_margin" => acct(&|a| a.used_margin.to_f64())?,
            "frozen_margin" => acct(&|a| a.frozen_margin.to_f64())?,
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
                    // same reason vocabulary as the journal `assertion` row
                    ("status".into(), json::s(a.status)),
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
        let broker = self.cfg.primary_broker().to_string();
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
                a.status = status;
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
                    WorldMsg::FlowConnect {
                        scope,
                        clock,
                        allowed,
                    } => self.record_connect(scope, clock, allowed),
                    WorldMsg::Opened {
                        id,
                        writer,
                        is_admin,
                    } => self.on_conn_opened(id, writer, is_admin),
                    WorldMsg::Inbound { id, frame } => self.on_frame(id, frame),
                    WorldMsg::Closed { id } => self.on_conn_closed(id),
                    WorldMsg::TransportClosed => self.shutdown = true,
                },
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            if self.shutdown {
                break;
            }
        }
        self.persist_ledger();
    }

    /// Apply the durable portion of the previous run that was written after
    /// the recovery snapshot. Output events are used as facts because working
    /// orders and the matching book are intentionally not part of a snapshot.
    fn replay_archived_tail(&mut self, data_dir: &str, snapshot_seq: u64) -> Result<(), String> {
        for event in journal::read_archived_tail(data_dir, snapshot_seq)? {
            if !journal_contract::validate_event(&event) {
                return Err(
                    "journal tail contains an event that violates the canonical contract".into(),
                );
            }
            let event_type = event
                .get_str("type")
                .ok_or_else(|| "recovery journal event missing type".to_string())?;
            match event_type.as_str() {
                "fill" => self.replay_fill_event(&event)?,
                "order_update" => self.replay_order_update_event(&event)?,
                "settlement" => self.replay_settlement_event(&event)?,
                "settings_updated" => self.replay_settings_event(&event)?,
                "reset_account" => {
                    let broker = event.get_str("broker").unwrap_or_default();
                    let all = event
                        .get("data")
                        .and_then(|data| data.get_bool("all"))
                        .unwrap_or(false);
                    if all {
                        let investors: Vec<String> = self
                            .ledger
                            .accounts()
                            .filter(|account| account.broker_id == broker)
                            .map(|account| account.investor_id.clone())
                            .collect();
                        for investor in investors {
                            self.ledger.reset_account(&broker, &investor);
                        }
                    } else if let Some(investor) = event.get_str("investor") {
                        self.ledger.reset_account(&broker, &investor);
                    } else {
                        return Err("reset_account recovery event missing investor".into());
                    }
                }
                "server_start"
                | "server_stop"
                | "scenario_loaded"
                | "session_auth"
                | "session_login"
                | "session_logout"
                | "md_watermark"
                | "assertion"
                | "order_insert"
                | "order_cancel"
                | "settlement_report"
                | "ledger_recovered"
                | "system_announcement" => {}
                "flow_input" | "flow_output" => {}
                "deposit" | "withdraw" => self.replay_cash_event(&event, event_type.as_str())?,
                "settle" | "admin" => {
                    return Err(format!(
                        "journal tail event '{event_type}' cannot be reconstructed from the snapshot"
                    ));
                }
                other => return Err(format!("unknown recovery event '{other}'")),
            }
        }
        Ok(())
    }

    fn replay_cash_event(&mut self, event: &Value, event_type: &str) -> Result<(), String> {
        let broker = event
            .get_str("broker")
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("{event_type} recovery event missing broker"))?;
        let investor = event
            .get_str("investor")
            .filter(|value| !value.is_empty())
            .ok_or_else(|| format!("{event_type} recovery event missing investor"))?;
        let amount = event
            .get("data")
            .and_then(|data| data.get_num("amount"))
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| format!("{event_type} recovery event has invalid amount"))?;
        let result = match event_type {
            "deposit" => self.ledger.apply_deposit(&broker, &investor, amount),
            "withdraw" => self.ledger.apply_withdraw(&broker, &investor, amount),
            _ => return Err(format!("unknown cash recovery event '{event_type}'")),
        };
        result.map_err(|error| format!("{event_type} recovery failed: {error:?}"))
    }

    fn replay_settlement_event(&mut self, event: &Value) -> Result<(), String> {
        let data = event
            .get("data")
            .ok_or_else(|| "settlement recovery event missing data".to_string())?;
        let next_day = data
            .get_str("next_trading_day")
            .ok_or_else(|| "settlement recovery event missing next_trading_day".to_string())?;
        let prices = data
            .get("settlement_prices")
            .and_then(Value::as_obj)
            .ok_or_else(|| "settlement recovery event missing settlement_prices".to_string())?
            .iter()
            .map(|(instrument, value)| {
                let price = value
                    .as_num()
                    .filter(|value| value.is_finite())
                    .ok_or_else(|| format!("settlement price for {instrument} is invalid"))?;
                Ok((instrument.clone(), price))
            })
            .collect::<Result<std::collections::HashMap<_, _>, String>>()?;
        self.ledger
            .settle_trading_day(
                self.engine.catalog(),
                &prices,
                &self.vt_trading_day,
                &next_day,
            )
            .map_err(|error| format!("settlement recovery failed: {error}"))?;
        self.engine.advance_trading_day();
        if let Some(playback) = self.playback.as_mut() {
            playback.advance_trading_day(&next_day);
        }
        self.clear_day_flow();
        self.settlement_confirmed.clear();
        self.vt_trading_day = next_day;
        self.vt_now_ms = 0.0;
        Ok(())
    }

    fn replay_settings_event(&mut self, event: &Value) -> Result<(), String> {
        let after = event
            .get("data")
            .and_then(|data| data.get("after"))
            .ok_or_else(|| "settings_updated recovery event missing after".to_string())?;
        let next = settings::patch(&self.cfg, after)?;
        self.ledger.set_initial_funds(next.initial_funds);
        self.engine
            .set_self_trade_prevention(next.self_trade_prevention);
        self.cfg = next;
        Ok(())
    }

    fn replay_fill_event(&mut self, event: &Value) -> Result<(), String> {
        let broker = event.get_str("broker").unwrap_or_default();
        let investor = event.get_str("investor").unwrap_or_default();
        let data = event
            .get("data")
            .ok_or_else(|| "fill recovery event missing data".to_string())?;
        let instrument = data
            .get_str("instrument")
            .ok_or_else(|| "fill recovery event missing instrument".to_string())?;
        let direction = Direction::from_ctp(
            data.get_num("direction")
                .ok_or_else(|| "fill recovery event missing direction".to_string())?
                as u8,
        )
        .ok_or_else(|| "fill recovery event has invalid direction".to_string())?;
        let offset = OffsetFlag::from_ctp(
            data.get_num("offset")
                .ok_or_else(|| "fill recovery event missing offset".to_string())? as u8,
        )
        .ok_or_else(|| "fill recovery event has invalid offset".to_string())?;
        let price = data
            .get_num("price")
            .filter(|value| value.is_finite())
            .ok_or_else(|| "fill recovery event has invalid price".to_string())?;
        let volume = data
            .get_num("volume")
            .filter(|value| value.is_finite() && value.fract() == 0.0 && *value > 0.0)
            .map(|value| value as i32)
            .ok_or_else(|| "fill recovery event has invalid volume".to_string())?;
        let trade_id = data
            .get_str("trade_id")
            .ok_or_else(|| "fill recovery event missing trade_id".to_string())?;
        let order_sys_id = data.get_str("order_sys_id").unwrap_or_default();
        let order_ref = data.get_str("order_ref").unwrap_or_default();
        let exchange = self
            .engine
            .catalog()
            .get(&instrument)
            .map(|item| item.exchange_id.clone())
            .ok_or_else(|| format!("fill recovery instrument '{instrument}' is unknown"))?;
        self.ledger.ensure_account(&broker, &investor);
        let side = if offset == OffsetFlag::Open {
            PositionSide::of(direction)
        } else {
            PositionSide::of(direction).opposite()
        };
        let pre_settlement = data
            .get_num("pre_settlement")
            .filter(|value| value.is_finite())
            .or_else(|| {
                self.ledger
                    .position(&broker, &investor, &instrument, side)
                    .map(|position| position.pre_settlement_price)
            })
            .unwrap_or(0.0);
        let mut fill = Fill {
            broker_id: [0; 11],
            investor_id: [0; 13],
            user_id: [0; 16],
            instrument_id: instrument.clone(),
            exchange_id: exchange,
            direction,
            offset,
            hedge_flag: b'1',
            price,
            volume,
            volume_total_original: data
                .get_num("volume_total_original")
                .filter(|value| {
                    value.is_finite() && value.fract() == 0.0 && *value >= volume as f64
                })
                .map(|value| value as i32)
                .unwrap_or(volume),
            order_sys_id: [0; 21],
            order_ref: [0; 13],
            trade_id: [0; 21],
            order_key: String::new(),
        };
        set_cstr(&mut fill.broker_id, &broker);
        set_cstr(&mut fill.investor_id, &investor);
        set_cstr(&mut fill.order_sys_id, &order_sys_id);
        set_cstr(&mut fill.order_ref, &order_ref);
        set_cstr(&mut fill.trade_id, &trade_id);
        self.ledger.on_fill(
            &fill,
            self.engine.catalog(),
            pre_settlement,
            &self.vt_trading_day,
        );
        let mut field = CThostFtdcTradeField::zeroed();
        set_cstr(&mut field.BrokerID, &broker);
        set_cstr(&mut field.InvestorID, &investor);
        set_cstr(&mut field.OrderRef, &order_ref);
        set_cstr(&mut field.OrderSysID, &order_sys_id);
        set_cstr(&mut field.InstrumentID, &instrument);
        set_cstr(&mut field.ExchangeID, &fill.exchange_id);
        set_cstr(&mut field.TradeID, &trade_id);
        field.Direction = direction.as_ctp();
        field.OffsetFlag = offset.as_ctp();
        field.Price = price;
        field.Volume = volume;
        self.private_seq.push((true, self.trades_today.len()));
        self.trades_today.push(field);
        Ok(())
    }

    fn replay_order_update_event(&mut self, event: &Value) -> Result<(), String> {
        let broker = event.get_str("broker").unwrap_or_default();
        let investor = event.get_str("investor").unwrap_or_default();
        let data = event
            .get("data")
            .ok_or_else(|| "order_update recovery event missing data".to_string())?;
        let instrument = data
            .get_str("instrument")
            .ok_or_else(|| "order_update recovery event missing instrument".to_string())?;
        let order_ref = data.get_str("order_ref").unwrap_or_default();
        let order_sys_id = data.get_str("order_sys_id").unwrap_or_default();
        let status = data
            .get_str("status")
            .and_then(|value| value.as_bytes().first().copied())
            .ok_or_else(|| "order_update recovery event missing status".to_string())?;
        let mut field = CThostFtdcOrderField::zeroed();
        set_cstr(&mut field.BrokerID, &broker);
        set_cstr(&mut field.InvestorID, &investor);
        set_cstr(&mut field.InstrumentID, &instrument);
        set_cstr(&mut field.OrderRef, &order_ref);
        set_cstr(&mut field.OrderSysID, &order_sys_id);
        let update_time = data.get_str("update_time").unwrap_or_default();
        set_cstr(&mut field.UpdateTime, &update_time);
        field.OrderStatus = status;
        field.VolumeTraded = data.get_num("volume_traded").unwrap_or(0.0) as i32;
        field.VolumeTotal = data.get_num("volume_total").unwrap_or(0.0) as i32;
        field.Direction = data.get_num("direction").unwrap_or(0.0) as u8;
        field.CombOffsetFlag[0] = data.get_num("offset").unwrap_or(0.0) as u8;
        field.LimitPrice = data.get_num("price").unwrap_or(0.0);
        field.ForceCloseReason = data.get_num("force_close_reason").unwrap_or(0.0) as u8;
        self.private_seq.push((false, self.orders_today.len()));
        self.orders_today.push(field);
        Ok(())
    }

    /// Write the recovery snapshot if the ledger changed since the last one.
    pub(crate) fn persist_ledger(&mut self) {
        if self.cfg.data_dir.is_empty() {
            return;
        }
        let snap = self.ledger.to_snapshot();
        let flow = self.private_flow_snapshot();
        let journal_seq = self.journal.as_ref().map(|j| j.seq());
        if self.last_snapshot.as_ref() == Some(&snap)
            && self.last_private_flow_snapshot.as_ref() == Some(&flow)
            && self.snapshot_journal_seq == journal_seq
        {
            return;
        }
        // Snapshot ordering is journal -> state: a recovery snapshot must
        // never claim a durable state while its corresponding audit prefix is
        // still buffered in the journal writer.
        if let Some(j) = self.journal.as_mut() {
            if let Err(e) = j.flush() {
                eprintln!("[ctpbuddy] journal flush before snapshot failed: {e}");
                return;
            }
        }
        match state::write_with_flow_meta(
            &self.cfg.data_dir,
            &self.vt_trading_day,
            &snap,
            &flow,
            journal_seq,
        ) {
            Ok(()) => {
                self.last_snapshot = Some(snap);
                self.last_private_flow_snapshot = Some(flow);
                self.snapshot_journal_seq = journal_seq;
            }
            Err(e) => eprintln!("[ctpbuddy] ledger snapshot failed: {e}"),
        }
    }

    /// Timer pulse: release due ticks, settle fills, mark to market, flush journal.
    fn pulse(&mut self) {
        let now = Instant::now();
        if !self.cfg.flow_replay {
            self.drain_ftd(now);
        }
        self.poll_auto_settlement(now);
        let ticks = self.poll_playback(now);
        if !ticks.is_empty() {
            self.process_ticks(&ticks);
            self.journal_record_json(
                "md_watermark",
                "",
                "",
                json::obj_sorted(vec![("idx".into(), json::n(self.playback_progress()))]),
            );
        }
        // scenario assertions fire at deterministic virtual times (§7.4)
        self.eval_assertions();
        self.mark_to_market();
        if let Some(j) = self.journal.as_mut() {
            j.flush_if_due(now);
        }
        // Hard-fail mode (review R1): a journal that cannot write leaves the
        // ledger ahead of the authoritative stream -- stop instead of running
        // on when the operator opted into strict consistency.
        let journal_failures = self
            .journal
            .as_ref()
            .map(|j| j.consecutive_failures())
            .unwrap_or(0);
        if self.cfg.journal_hard_fail && journal_failures > 0 {
            eprintln!(
                "[ctpbuddy] journal write failed and CTPBUDDY_JOURNAL_HARD_FAIL is set: stopping"
            );
            self.shutdown = true;
            return;
        }
        if now.duration_since(self.last_snapshot_at) >= SNAPSHOT_EVERY {
            self.last_snapshot_at = now;
            self.persist_ledger();
        }
    }

    pub(crate) fn admit_td_frame(&mut self, conn_id: u64, frame: Frame) -> Option<Frame> {
        let conn = self.conns.get_mut(&conn_id)?;
        if conn.ftd_pending.is_empty()
            && conn
                .ftd_budget
                .take(self.flow_clock, self.cfg.ftd_max_comm_flux)
        {
            return Some(frame);
        }
        if conn.ftd_pending.len() >= 1024 {
            self.send_error(conn_id, frame.req_id, 90, "CTP：查询未就绪，请稍后重试");
        } else {
            conn.ftd_pending.push_back(frame);
        }
        None
    }

    fn drain_ftd(&mut self, now: Instant) {
        self.flow_clock = now;
        let mut recorded = false;
        let mut ids: Vec<_> = self.conns.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            for _ in 0..1024 {
                let frame = {
                    let Some(conn) = self.conns.get_mut(&id) else {
                        break;
                    };
                    if conn.ftd_pending.is_empty()
                        || !conn.ftd_budget.take(now, self.cfg.ftd_max_comm_flux)
                    {
                        break;
                    }
                    conn.ftd_pending.pop_front().unwrap()
                };
                if !recorded {
                    self.record_flow("drain", 0, None);
                    recorded = true;
                }
                self.dispatch_td_frame(id, frame);
            }
        }
    }

    pub(crate) fn exchange_order_gate(
        &mut self,
        broker: &str,
        investor: &str,
        exchange: &str,
        stream: GateStream,
    ) -> bool {
        let limit = self
            .cfg
            .exchange_limits
            .get(exchange)
            .copied()
            .unwrap_or(self.cfg.exchange_freq);
        if limit == 0 {
            return true;
        }
        let key = (
            broker.to_string(),
            investor.to_string(),
            exchange.to_string(),
            stream,
        );
        self.exchange_freq_windows
            .entry(key)
            .or_default()
            .take(self.flow_clock, limit)
    }

    fn poll_playback(&mut self, now: Instant) -> Vec<Tick> {
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
        ticks
    }

    fn process_ticks(&mut self, ticks: &[Tick]) {
        for tick in ticks {
            self.push_market_data(tick);
            let events = self.engine.on_tick(tick);
            for ev in events {
                self.dispatch_event(ev);
            }
        }
    }

    fn mark_to_market(&mut self) {
        // mark to market (unrealized PnL drives account queries)
        let prices = self.engine.last_prices();
        let pre_settlements = self.engine.pre_settlements();
        self.ledger.mark_to_market(
            self.engine.catalog(),
            &prices,
            &pre_settlements,
            &self.vt_trading_day,
        );
    }

    fn push_market_data(&mut self, tick: &Tick) {
        let field = tick.to_depth_md();
        let frame = Frame::new(msgs::RTN_DEPTH_MD, 0, struct_to_bytes(&field));
        self.publish(Broadcast {
            topic: format!("md/{}/{}", tick.exchange_id, tick.instrument_id),
            frame: frame.clone(),
        });
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

    /// Apply an engine event to durable/account state, then hand the resulting
    /// notification to the transport fanout. Keeping those phases separate
    /// prevents connection delivery from becoming part of ledger semantics.
    fn dispatch_event(&mut self, ev: EngineEvent) {
        let notification = self.apply_engine_event(ev);
        self.emit_private_notification(notification);
    }

    fn apply_engine_event(&mut self, ev: EngineEvent) -> PrivateNotification {
        match ev {
            EngineEvent::Order(field) => self.apply_order_event(field),
            EngineEvent::Trade { field, fill } => self.apply_trade_event(field, fill),
        }
    }

    fn apply_order_event(&mut self, field: CThostFtdcOrderField) -> PrivateNotification {
        if field.OrderStatus == b'5' && field.OrderSubmitStatus != b'4' {
            let key = format!(
                "{}/{}/{}",
                field.FrontID,
                field.SessionID,
                cstr(&field.OrderRef)
            );
            self.ledger.unfreeze_order(&key, self.engine.catalog());
        }
        self.private_seq.push((false, self.orders_today.len()));
        self.orders_today.push(field);
        let field = self.orders_today.last().expect("order just appended");
        let field = *field;
        let frame = Frame::new(msgs::RTN_ORDER, 0, struct_to_bytes(&field));
        let (b, i) = (cstr(&field.BrokerID), cstr(&field.InvestorID));
        let day = self.vt_trading_day.clone();
        let now = self.vt_now_ms;
        self.journal_record_order_update(&day, now, &field);
        PrivateNotification {
            broker: b,
            investor: i,
            frame,
        }
    }

    fn apply_trade_event(
        &mut self,
        field: CThostFtdcTradeField,
        fill: ctpbuddy_matching::Fill,
    ) -> PrivateNotification {
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
        self.private_seq.push((true, self.trades_today.len()));
        self.trades_today.push(field);
        let field = self.trades_today.last().expect("trade just appended");
        let field = *field;
        let frame = Frame::new(msgs::RTN_TRADE, 0, struct_to_bytes(&field));
        let (b, i) = (cstr(&field.BrokerID), cstr(&field.InvestorID));
        let day = self.vt_trading_day.clone();
        let now = self.vt_now_ms;
        self.journal_record_fill(&day, now, &fill, pre_settle);
        PrivateNotification {
            broker: b,
            investor: i,
            frame,
        }
    }

    fn emit_private_notification(&mut self, notification: PrivateNotification) {
        self.publish(Broadcast {
            topic: format!("td/{}/{}", notification.broker, notification.investor),
            frame: notification.frame.clone(),
        });
        let targets = self.investor_conns(&notification.broker, &notification.investor);
        self.mark_delivered(
            &notification.broker,
            &notification.investor,
            !targets.is_empty(),
        );
        for id in targets {
            self.send_frame(id, notification.frame.clone());
        }
    }
    fn on_conn_opened(&mut self, id: u64, writer: Sender<Frame>, is_admin: bool) {
        if !is_admin && !self.cfg.flow_replay {
            self.flow_clock = Instant::now();
            self.record_flow("open", id, None);
        }
        self.conns.insert(
            id,
            Conn {
                writer,
                is_admin,
                authenticated: is_admin && self.security.admin_token.is_none(),
                front_id: id as i32,
                session_id: 0,
                logins: 0,
                broker_id: None,
                investor_id: None,
                user_id: [0u8; 16],
                order_local_seq: 0,
                md_subs: HashSet::new(),
                private_resume: 2,
                qry_window: None,
                qry_count: 0,
                ftd_budget: flow::WindowBudget::default(),
                ftd_pending: VecDeque::new(),
            },
        );
    }

    fn on_conn_closed(&mut self, id: u64) {
        if self.conns.get(&id).is_some_and(|c| !c.is_admin) {
            if !self.cfg.flow_replay {
                self.flow_clock = Instant::now();
            }
            self.record_flow("close", id, None);
        }
        self.conns.remove(&id);
    }

    // ---- helpers shared by handlers / admin ----

    pub(crate) fn send_frame(&mut self, conn_id: u64, frame: Frame) {
        self.record_flow_output(conn_id, &frame);
        if let Some(c) = self.conns.get(&conn_id) {
            let _ = c.writer.send(frame);
        }
    }

    fn publish(&mut self, broadcast: Broadcast) {
        if let Some(tx) = &self.broadcast_tx {
            match tx.try_send(broadcast) {
                Ok(()) => self.broadcast_published = self.broadcast_published.saturating_add(1),
                Err(_) => self.broadcast_dropped = self.broadcast_dropped.saturating_add(1),
            }
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
            ("direction".into(), json::n(field.Direction as f64)),
            ("offset".into(), json::n(field.CombOffsetFlag[0] as f64)),
            ("price".into(), json::n(field.LimitPrice)),
            (
                "force_close_reason".into(),
                json::n(field.ForceCloseReason as f64),
            ),
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

    fn journal_record_fill(
        &mut self,
        day: &str,
        ms: f64,
        fill: &ctpbuddy_matching::Fill,
        pre_settlement: f64,
    ) {
        let data = json::obj_sorted(vec![
            ("trade_id".into(), json::s(&cstr(&fill.trade_id))),
            ("order_sys_id".into(), json::s(&cstr(&fill.order_sys_id))),
            ("order_ref".into(), json::s(&cstr(&fill.order_ref))),
            ("instrument".into(), json::s(&fill.instrument_id)),
            ("direction".into(), json::n(fill.direction.as_ctp() as f64)),
            ("offset".into(), json::n(fill.offset.as_ctp() as f64)),
            ("price".into(), json::n(fill.price)),
            ("volume".into(), json::n(fill.volume as f64)),
            (
                "volume_total_original".into(),
                json::n(fill.volume_total_original as f64),
            ),
            ("pre_settlement".into(), json::n(pre_settlement)),
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

    /// Fuzz: hostile frames on a TD and an ADMIN connection must never panic
    /// the world loop (a panic would kill every client).
    #[test]
    fn world_survives_hostile_frames() {
        let mut s: u64 = 0x1234_5678_9ABC_DEF1;
        let mut next = move || {
            s ^= s >> 12;
            s ^= s << 25;
            s ^= s >> 27;
            s.wrapping_mul(0x2545F4914F6CDD1D)
        };
        let mut world = World::new(Config::default()).expect("world");
        let (td_tx, td_rx) = mpsc::channel::<Frame>();
        let (ad_tx, ad_rx) = mpsc::channel::<Frame>();
        world.on_conn_opened(1, td_tx, false);
        world.on_conn_opened(2, ad_tx, true);
        let seeds: [&[u8]; 6] = [
            b"{\"broker_id\":\"9999\",\"user_id\":\"u\",\"app_id\":\"a\",\"auth_code\":\"x\"}",
            b"{\"cmd\":\"status\"}",
            b"{\"cmd\":null}",
            b"[1,2,3]",
            b"{\"user_id\":\"\\ud800\",\"n\":1e999}",
            b"",
        ];
        for i in 0..4000u32 {
            let conn = 1 + (next() % 2);
            // concentrate on defined id ranges, with some wild values
            let msg_type = match next() % 4 {
                0 => (next() % 0x300) as u16,
                1 => 0x1000 + (next() % 0x100) as u16,
                2 => 0x2000 + (next() % 0x100) as u16,
                _ => next() as u16,
            };
            let mut payload = if next() % 2 == 0 {
                seeds[(next() % seeds.len() as u64) as usize].to_vec()
            } else {
                (0..(next() % 300)).map(|_| next() as u8).collect()
            };
            if !payload.is_empty() && next() % 2 == 0 {
                let k = (next() % payload.len() as u64) as usize;
                payload[k] ^= 1 << (next() % 8);
            }
            world.on_frame(conn, Frame::new(msg_type, i, payload));
            while td_rx.try_recv().is_ok() {}
            while ad_rx.try_recv().is_ok() {}
        }
    }

    fn test_limits(max_conns: usize, write_timeout: Duration) -> ConnLimits {
        ConnLimits {
            max_conns,
            connect_freq: 0,
            write_timeout,
            live: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    fn start_listener(lim: ConnLimits) -> (std::net::SocketAddr, Receiver<WorldMsg>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = mpsc::channel::<WorldMsg>();
        thread::spawn(move || accept_loop_with(listener, tx, false, lim));
        (addr, rx)
    }

    fn wait_until(what: &str, mut f: impl FnMut() -> bool) {
        let t0 = Instant::now();
        while !f() {
            assert!(t0.elapsed() < Duration::from_secs(10), "timeout: {what}");
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// Over-limit peers are closed immediately and never reach the world loop;
    /// a slot frees up once an accepted peer disconnects.
    #[test]
    fn connection_limit_rejects_excess_and_recovers() {
        use std::io::Read;
        let lim = test_limits(2, Duration::from_secs(5));
        let live = lim.live.clone();
        let (addr, rx) = start_listener(lim);
        let c1 = TcpStream::connect(addr).unwrap();
        let c2 = TcpStream::connect(addr).unwrap();
        wait_until("two accepted", || live.load(Ordering::SeqCst) == 2);
        let mut c3 = TcpStream::connect(addr).unwrap();
        c3.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        let mut b = [0u8; 1];
        // rejected peer sees EOF (or a reset), not a hang
        match c3.read(&mut b) {
            Ok(0) | Err(_) => {}
            Ok(n) => panic!("unexpected {n} bytes"),
        }
        assert_eq!(live.load(Ordering::SeqCst), 2);
        let opened = rx
            .try_iter()
            .filter(|m| matches!(m, WorldMsg::Opened { .. }))
            .count();
        assert_eq!(opened, 2, "rejected peer must not be opened in the world");
        // free a slot: the reader sees EOF, reports Closed and returns the slot
        drop(c1);
        wait_until("slot released", || live.load(Ordering::SeqCst) == 1);
        let _c4 = TcpStream::connect(addr).unwrap();
        wait_until("new peer accepted", || live.load(Ordering::SeqCst) == 2);
        drop(c2);
    }

    /// A peer that never reads: the writer hits the write timeout, shuts the
    /// socket, the reader reports `Closed` and the slot is released.
    #[test]
    fn write_timeout_drops_stalled_peer() {
        let lim = test_limits(4, Duration::from_millis(300));
        let live = lim.live.clone();
        let (addr, rx) = start_listener(lim);
        let _stalled = TcpStream::connect(addr).unwrap(); // never reads
        let writer = match rx.recv_timeout(Duration::from_secs(5)).unwrap() {
            WorldMsg::Opened { writer, .. } => writer,
            _ => panic!("expected Opened"),
        };
        let payload = vec![0u8; 256 * 1024];
        let t0 = Instant::now();
        let mut saw_closed = false;
        while t0.elapsed() < Duration::from_secs(20) && !saw_closed {
            // keep the send buffer full until the writer gives up; once the
            // writer exits, the reader must report `Closed` to the world loop
            let _ = writer.send(Frame::new(1, 0, payload.clone()));
            while let Ok(m) = rx.try_recv() {
                if matches!(m, WorldMsg::Closed { .. }) {
                    saw_closed = true;
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(saw_closed, "stalled peer was never reported Closed");
        assert!(
            writer.send(Frame::new(1, 0, vec![0u8; 1])).is_err(),
            "writer thread should have exited"
        );
        wait_until("slot released", || live.load(Ordering::SeqCst) == 0);
    }

    #[test]
    fn max_conns_parsing() {
        assert_eq!(parse_max_conns(None), DEFAULT_MAX_CONNS);
        assert_eq!(parse_max_conns(Some("8")), 8);
        assert_eq!(parse_max_conns(Some("0")), DEFAULT_MAX_CONNS);
        assert_eq!(parse_max_conns(Some("abc")), DEFAULT_MAX_CONNS);
    }

    #[test]
    fn broker_allowlist_accepts_comma_separated_ids() {
        let mut cfg = Config::default();
        cfg.broker_id = "8888, B2".into();
        assert!(cfg.supports_broker("8888"));
        assert!(cfg.supports_broker("B2"));
        assert!(!cfg.supports_broker("B3"));
        assert_eq!(cfg.broker_ids(), vec!["8888", "B2"]);
    }

    /// Review R1: in hard-fail mode a journal that cannot write stops the
    /// world loop instead of running on with a ledger ahead of the
    /// authoritative event stream. (The failed-write state itself is injected
    /// via the journal's test helper; detecting a real I/O failure is covered
    /// by the journal's own tests.)
    #[test]
    fn journal_hard_fail_stops_the_world_loop() {
        let mut cfg = Config::default();
        // separate data dirs: a journal lock admits one writer per directory
        let dir_a = std::env::temp_dir().join(format!("ctpbuddy-jhard-a-{}", std::process::id()));
        let dir_b = std::env::temp_dir().join(format!("ctpbuddy-jhard-b-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir_a);
        let _ = std::fs::remove_dir_all(&dir_b);
        cfg.data_dir = dir_a.to_str().unwrap().to_string();
        let mut world = World::new(cfg.clone()).expect("world");
        world.journal.as_mut().unwrap().simulate_write_failure();
        world.pulse();
        assert!(
            !world.shutdown,
            "default mode keeps running (failure visible in status only)"
        );
        drop(world);

        cfg.journal_hard_fail = true;
        cfg.data_dir = dir_b.to_str().unwrap().to_string();
        let mut world = World::new(cfg).expect("world");
        world.journal.as_mut().unwrap().simulate_write_failure();
        world.pulse();
        assert!(world.shutdown, "hard-fail mode must stop the world loop");
        let _ = std::fs::remove_dir_all(&dir_a);
        let _ = std::fs::remove_dir_all(&dir_b);
    }

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

    #[test]
    fn scenario_calendar_snapshot_need_not_cover_wall_clock_date() {
        let dir = std::env::temp_dir().join(format!(
            "ctpbuddy-calendar-scenario-{}-{}",
            std::process::id(),
            NEXT_CONN_ID.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scenarios/dsl_demo/ticks.csv");
        std::fs::copy(source, dir.join("ticks.csv")).unwrap();
        let calendar = calendar::CalendarSnapshot::from_value(&serde_json::json!({
            "schema": "ctpbuddy.trading-calendar/v1",
            "version": "historical-fixture",
            "days": [{
                "date": "2099-01-01",
                "is_trading_day": true,
                "trading_day": "20990101"
            }]
        }))
        .unwrap();

        let mut cfg = Config::default();
        cfg.scenario_dir = Some(dir.to_string_lossy().into_owned());
        cfg.calendar_snapshot = Some(calendar);
        let world = World::new(cfg).expect("scenario startup should use tick TradingDay");
        assert_eq!(world.vt_trading_day, "20261002");
        drop(world);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn calendar_metadata_requires_explicit_matching_provenance() {
        let hash = "A".repeat(64);
        assert_eq!(
            calendar_metadata(Some(" fixture-r1 ".into()), Some(hash)).unwrap(),
            Some(CalendarMetadata {
                version: "fixture-r1".into(),
                sha256: "a".repeat(64),
            })
        );
        for (version, sha256) in [
            (Some("fixture".into()), None),
            (None, Some("a".repeat(64))),
            (Some("fixture".into()), Some("bad".into())),
            (Some("".into()), Some("a".repeat(64))),
        ] {
            assert!(calendar_metadata(version, sha256).is_err());
        }
        assert!(calendar_metadata(None, None).unwrap().is_none());
    }

    #[test]
    fn admin_status_exposes_explicit_calendar_provenance() {
        let mut cfg = Config::default();
        cfg.calendar = calendar_metadata(Some("fixture-r1".into()), Some("b".repeat(64))).unwrap();
        let mut world = World::new(cfg).unwrap();
        let (tx, rx) = mpsc::channel::<Frame>();
        world.on_conn_opened(1, tx, true);
        world.on_frame(
            1,
            Frame::new(msgs::ADMIN_REQ, 7, br#"{"cmd":"status"}"#.to_vec()),
        );
        let response = rx.try_iter().next().expect("ADMIN status response");
        let payload = String::from_utf8(response.payload).unwrap();
        let value = json::parse(&payload).unwrap();
        assert_eq!(
            value.get_str("calendar_version").as_deref(),
            Some("fixture-r1")
        );
        assert_eq!(value.get_str("calendar_sha256"), Some("b".repeat(64)));
        assert_eq!(value.get_str("contract_version"), Some("3".into()));
        assert!(matches!(value.get("positions"), Some(json::Value::Arr(rows)) if rows.is_empty()));
    }

    #[test]
    fn admin_announcement_publishes_sys_topic() {
        let mut world = World::new(Config::default()).expect("world");
        let (broadcast_tx, mut broadcast_rx) = tokio::sync::mpsc::channel(1);
        world.broadcast_tx = Some(broadcast_tx);
        let (tx, rx) = mpsc::channel::<Frame>();
        world.on_conn_opened(1, tx, true);
        world.on_frame(
            1,
            Frame::new(
                msgs::ADMIN_REQ,
                8,
                br#"{"cmd":"announce","severity":"warning","message":"maintenance"}"#.to_vec(),
            ),
        );
        let response = rx.try_iter().next().expect("announce response");
        let response_value = json::parse(&String::from_utf8(response.payload).unwrap()).unwrap();
        assert_eq!(response_value.get_bool("ok"), Some(true));
        let broadcast = broadcast_rx.try_recv().expect("sys broadcast");
        assert_eq!(broadcast.topic, "sys/announcement");
        assert_eq!(broadcast.frame.msg_type, msgs::ADMIN_RSP);
        let payload = json::parse(&String::from_utf8(broadcast.frame.payload).unwrap()).unwrap();
        assert_eq!(payload.get_str("type"), Some("announcement".into()));
        assert_eq!(payload.get_str("severity"), Some("warning".into()));
        assert_eq!(payload.get_str("message"), Some("maintenance".into()));
        world.on_frame(
            1,
            Frame::new(msgs::ADMIN_REQ, 9, br#"{"cmd":"status"}"#.to_vec()),
        );
        let status = rx.try_iter().next().expect("status response");
        let status_value = json::parse(&String::from_utf8(status.payload).unwrap()).unwrap();
        assert_eq!(
            status_value
                .get("pubsub")
                .and_then(|v| v.get_bool("enabled")),
            Some(true)
        );
        assert_eq!(
            status_value
                .get("pubsub")
                .and_then(|v| v.get_num("published")),
            Some(1.0)
        );
        assert_eq!(
            status_value
                .get("pubsub")
                .and_then(|v| v.get_num("dropped")),
            Some(0.0)
        );
    }

    #[test]
    fn settle_day_force_closes_expired_position_with_auditable_events() {
        let mut world = World::new(Config::default()).expect("world");
        world.vt_trading_day = "20261012".into();
        let instrument = "sc2603";
        assert!(world.engine.catalog().get(instrument).is_some());
        world.ledger.ensure_account("8888", "u");
        let position =
            world
                .ledger
                .position_mut_or_create("8888", "u", instrument, PositionSide::Long);
        position.today_position = 1;
        position.open_volume = 1;
        position.open_amount = ctpbuddy_ledger::Money::from_f64(700.0);
        position.position_cost = ctpbuddy_ledger::Money::from_f64(690.0);
        position.settlement_price = 690.0;
        position.add_detail(
            "20261012",
            "seed",
            700.0,
            1,
            ctpbuddy_ledger::Money::ZERO,
            690.0,
        );

        let (broadcast_tx, mut broadcast_rx) = tokio::sync::mpsc::channel(4);
        world.broadcast_tx = Some(broadcast_tx);
        let (tx, rx) = mpsc::channel::<Frame>();
        world.on_conn_opened(1, tx, true);
        world.on_frame(
            1,
            Frame::new(
                msgs::ADMIN_REQ,
                9,
                br#"{"cmd":"settle_day","next_trading_day":"20261013","settlement_prices":{"sc2603":700.0}}"#.to_vec(),
            ),
        );
        let response = rx.try_iter().next().expect("settle response");
        let value = json::parse(&String::from_utf8(response.payload).unwrap()).unwrap();
        assert_eq!(value.get_bool("ok"), Some(true));
        assert_eq!(value.get_num("forced_liquidations"), Some(1.0));
        let broadcast = loop {
            let next = broadcast_rx.try_recv().expect("settlement sys broadcast");
            if next.topic == "sys/settlement" {
                break next;
            }
        };
        assert_eq!(broadcast.topic, "sys/settlement");
        let broadcast_value =
            json::parse(&String::from_utf8(broadcast.frame.payload).unwrap()).unwrap();
        assert_eq!(
            broadcast_value.get_str("type"),
            Some("settlement_complete".into())
        );
        assert!(world
            .ledger
            .position("8888", "u", instrument, PositionSide::Long)
            .is_none());
    }

    #[test]
    fn recovery_replays_fill_into_ledger_and_private_flow() {
        let mut world = World::new(Config::default()).expect("world");
        let event = json::parse(
            r#"{"type":"fill","broker":"9999","investor":"u","data":{"trade_id":"T1","order_sys_id":"S1","order_ref":"1","instrument":"sc2603","direction":48,"offset":48,"price":700.0,"volume":1,"volume_total_original":1,"pre_settlement":690.0}}"#,
        )
        .unwrap();
        world.replay_fill_event(&event).unwrap();
        let position = world
            .ledger
            .position("9999", "u", "sc2603", PositionSide::Long)
            .expect("replayed position");
        assert_eq!(position.today_position, 1);
        assert_eq!(world.trades_today.len(), 1);
        assert_eq!(world.private_seq, vec![(true, 0)]);
    }

    #[test]
    fn recovery_replays_settlement_and_advances_trading_day() {
        let mut world = World::new(Config::default()).expect("world");
        world.vt_trading_day = "20261003".into();
        let event = json::parse(
            r#"{"type":"settlement","data":{"next_trading_day":"20261004","settlement_prices":{},"accounts":[]}}"#,
        )
        .unwrap();
        world.replay_settlement_event(&event).unwrap();
        assert_eq!(world.vt_trading_day, "20261004");
        assert_eq!(world.vt_now_ms, 0.0);
    }

    #[test]
    fn recovery_replays_settings_without_persisting_again() {
        let mut world = World::new(Config::default()).expect("world");
        let event = json::parse(
            r#"{"type":"settings_updated","data":{"after":{"qry_freq":7,"order_freq":31,"max_user_sessions":4,"settlement_required":false,"self_trade_prevention":true,"initial_funds":12345.0}}}"#,
        )
        .unwrap();
        world.replay_settings_event(&event).unwrap();
        assert_eq!(world.cfg.qry_freq, 7);
        assert_eq!(world.cfg.order_freq, 31);
        assert_eq!(world.cfg.max_user_sessions, 4);
        assert!(!world.cfg.settlement_required);
        assert!(world.cfg.self_trade_prevention);
        assert_eq!(world.cfg.initial_funds, 12345.0);
    }

    #[test]
    fn recovery_replays_external_cash_events() {
        let mut world = World::new(Config::default()).expect("world");
        let deposit = json::parse(
            r#"{"type":"deposit","broker":"8888","investor":"u","data":{"amount":125.5}}"#,
        )
        .unwrap();
        let withdraw = json::parse(
            r#"{"type":"withdraw","broker":"8888","investor":"u","data":{"amount":25.5}}"#,
        )
        .unwrap();
        world.replay_cash_event(&deposit, "deposit").unwrap();
        world.replay_cash_event(&withdraw, "withdraw").unwrap();
        let account = world.ledger.account("8888", "u").unwrap();
        assert_eq!(account.deposit.to_f64(), 125.5);
        assert_eq!(account.withdraw.to_f64(), 25.5);
    }

    #[test]
    fn admin_cash_transfer_updates_account_and_journals_identity() {
        let dir = std::env::temp_dir().join(format!("ctpbuddy-cash-admin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut cfg = Config::default();
        cfg.data_dir = dir.to_string_lossy().into_owned();
        let mut world = World::new(cfg).expect("world");
        let (tx, rx) = mpsc::channel::<Frame>();
        world.on_conn_opened(1, tx, true);
        world.on_frame(
            1,
            Frame::new(
                msgs::ADMIN_REQ,
                1,
                br#"{"cmd":"deposit","broker":"8888","investor":"u","amount":125.5}"#.to_vec(),
            ),
        );
        let response = rx.try_iter().next().expect("deposit response");
        let value = json::parse(&String::from_utf8(response.payload).unwrap()).unwrap();
        assert!(matches!(value.get("ok"), Some(Value::Bool(true))));
        assert_eq!(
            world.ledger.account("8888", "u").unwrap().deposit.to_f64(),
            125.5
        );

        world.on_frame(
            1,
            Frame::new(
                msgs::ADMIN_REQ,
                2,
                br#"{"cmd":"withdraw","broker":"8888","investor":"u","amount":25.5}"#.to_vec(),
            ),
        );
        let _ = rx.try_iter().next().expect("withdraw response");
        assert_eq!(
            world.ledger.account("8888", "u").unwrap().withdraw.to_f64(),
            25.5
        );
        drop(world);

        let journal = std::fs::read_to_string(dir.join("journal").join("20261003.jsonl"))
            .or_else(|_| {
                let journal_dir = std::fs::read_dir(dir.join("journal"))?;
                let path = journal_dir
                    .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                    .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("jsonl"))
                    .ok_or_else(|| std::io::Error::other("journal file missing"))?;
                std::fs::read_to_string(path)
            })
            .expect("cash journal");
        assert!(journal.contains("\"type\":\"deposit\""));
        assert!(journal.contains("\"type\":\"withdraw\""));
        assert!(journal.contains("\"broker\":\"8888\""));
        assert!(journal.contains("\"investor\":\"u\""));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn multi_broker_admin_settlement_and_reports_are_scoped() {
        let dir = std::env::temp_dir().join(format!(
            "ctpbuddy-multi-broker-settlement-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let mut cfg = Config::default();
        cfg.broker_id = "B1,B2".into();
        cfg.data_dir = dir.to_string_lossy().into_owned();
        let mut world = World::new(cfg).expect("world");
        world.ledger.ensure_account("B2", "u2");
        world.ledger.ensure_account("B1", "u1");
        let (tx, rx) = mpsc::channel::<Frame>();
        world.on_conn_opened(1, tx, true);
        world.on_frame(
            1,
            Frame::new(msgs::ADMIN_REQ, 0, br#"{"cmd":"status"}"#.to_vec()),
        );
        let status = rx.try_iter().next().expect("status response");
        let status_value = json::parse(&String::from_utf8(status.payload).unwrap()).unwrap();
        assert_eq!(
            status_value.get("brokers").map(Value::to_json).as_deref(),
            Some("[\"B1\",\"B2\"]")
        );
        let accounts = match status_value.get("accounts") {
            Some(Value::Arr(rows)) => rows,
            _ => panic!("status accounts"),
        };
        let identities: Vec<_> = accounts
            .iter()
            .map(|row| {
                (
                    row.get_str("broker").unwrap(),
                    row.get_str("investor").unwrap(),
                    row.get_str("currency").unwrap(),
                )
            })
            .collect();
        assert_eq!(
            identities,
            vec![
                ("B1".into(), "u1".into(), "CNY".into()),
                ("B2".into(), "u2".into(), "CNY".into())
            ]
        );
        world.on_frame(
            1,
            Frame::new(
                msgs::ADMIN_REQ,
                1,
                br#"{"cmd":"settlement_report","reports":[{"broker":"B2","investor":"u2","trading_day":"20261012","settlement_id":1,"account_id":"u2","currency_id":"CNY","content_bytes":[65]}]}"#.to_vec(),
            ),
        );
        let report = rx.try_iter().next().expect("report response");
        let report_value = json::parse(&String::from_utf8(report.payload).unwrap()).unwrap();
        assert!(matches!(report_value.get("ok"), Some(Value::Bool(true))));
        assert_eq!(world.settlement_reports.len(), 1);
        assert_eq!(world.settlement_reports[0].broker, "B2");

        let next_day = "29990101";
        world.on_frame(
            1,
            Frame::new(
                msgs::ADMIN_REQ,
                2,
                format!(
                    "{{\"cmd\":\"settle_day\",\"settlement_prices\":{{}},\"next_trading_day\":\"{next_day}\"}}"
                )
                .into_bytes(),
            ),
        );
        let settle = rx.try_iter().next().expect("settle response");
        let settle_value = json::parse(&String::from_utf8(settle.payload).unwrap()).unwrap();
        assert!(matches!(settle_value.get("ok"), Some(Value::Bool(true))));
        assert_eq!(world.vt_trading_day, next_day);
        drop(world);

        let journal_dir = std::fs::read_dir(dir.join("journal")).unwrap();
        let journal = journal_dir
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("jsonl"))
            .and_then(|path| std::fs::read_to_string(path).ok())
            .expect("settlement journal");
        assert!(journal.contains("\"brokers\":[\"B1\",\"B2\"]"), "{journal}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn recover_startup_applies_tail_from_previous_archived_run() {
        let dir =
            std::env::temp_dir().join(format!("ctpbuddy-recover-tail-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut first_cfg = Config::default();
        first_cfg.data_dir = dir.to_string_lossy().into_owned();
        let mut first = World::new(first_cfg).expect("first world");
        first.journal_record_json(
            "fill",
            "9999",
            "u",
            json::parse(
                r#"{"trade_id":"T1","order_sys_id":"S1","order_ref":"1","instrument":"sc2603","direction":48,"offset":48,"price":700.0,"volume":1,"volume_total_original":1,"pre_settlement":690.0}"#,
            )
            .unwrap(),
        );
        first.journal_record_json(
            "deposit",
            "9999",
            "u",
            json::parse(r#"{"amount":125.5}"#).unwrap(),
        );
        first.journal_record_json(
            "withdraw",
            "9999",
            "u",
            json::parse(r#"{"amount":25.5}"#).unwrap(),
        );
        drop(first);

        let mut second_cfg = Config::default();
        second_cfg.data_dir = dir.to_string_lossy().into_owned();
        second_cfg.recover = true;
        let second = World::new(second_cfg).expect("recovered world");
        assert_eq!(
            second
                .ledger
                .position("9999", "u", "sc2603", PositionSide::Long)
                .map(|position| position.today_position),
            Some(1)
        );
        assert_eq!(second.trades_today.len(), 1);
        let account = second
            .ledger
            .account("9999", "u")
            .expect("replayed account");
        assert_eq!(account.deposit.to_f64(), 125.5);
        assert_eq!(account.withdraw.to_f64(), 25.5);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
