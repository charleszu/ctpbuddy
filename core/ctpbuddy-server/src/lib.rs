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

use std::collections::{HashMap, HashSet};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use ctpbuddy_ledger::Ledger;
use ctpbuddy_market::{format_hhmmss, CsvSource, Playback, Tick};
use ctpbuddy_matching::{Catalog, EngineEvent, MatchingEngine};
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

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct Config {
    pub td_endpoint: String,
    pub admin_endpoint: String,
    /// The only BrokerID this core instance serves (DESIGN.md §6.4).
    pub broker_id: String,
    pub scenario_dir: Option<String>,
    pub initial_funds: f64,
    /// 0 = as fast as possible.
    pub playback_speed: f64,
    /// Directory for the journal / logs. Empty = disabled.
    pub data_dir: String,
    /// 查询流控 (docs: 报单流控、查询流控和会话数控制): per-session
    /// ReqQry* budget per second. Exceeding it answers OnRspError[90]
    /// "CTP：查询未就绪，请稍后重试", same as a real front (front_se QryFreq).
    pub qry_freq: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            td_endpoint: "127.0.0.1:5560".into(),
            admin_endpoint: "127.0.0.1:5561".into(),
            broker_id: "8888".into(),
            scenario_dir: None,
            initial_funds: 2_000_000.0,
            playback_speed: 0.0,
            data_dir: String::new(),
            qry_freq: 2,
        }
    }
}

/// Bind transports and run the world loop until an admin `shutdown`.
pub fn run(cfg: Config) -> std::io::Result<()> {
    let td = TcpListener::bind(&cfg.td_endpoint)?;
    let admin = TcpListener::bind(&cfg.admin_endpoint)?;
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

    let mut world = World::new(cfg);
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
                let _ = tx.send(WorldMsg::ConnOpened {
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
                thread::spawn(move || writer_loop(id, s, writer_rx));
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
                if tx.send(WorldMsg::ConnFrame { id, frame }).is_err() {
                    break;
                }
            }
            Ok(None) => {
                let _ = tx.send(WorldMsg::ConnClosed { id });
                break;
            }
            Err(e) => {
                eprintln!("[ctpbuddy] conn {id} read error: {e}");
                let _ = tx.send(WorldMsg::ConnClosed { id });
                break;
            }
        }
    }
}

fn writer_loop(id: u64, mut stream: TcpStream, rx: Receiver<Frame>) {
    for frame in rx {
        if frame.write_to(&mut stream).is_err() {
            break;
        }
    }
    // world dropped our sender (conn removed) or socket died: signal close.
    // The reader loop detects EOF on its own; this is the belt-and-braces path.
    let _ = id;
}

pub(crate) enum WorldMsg {
    ConnOpened {
        id: u64,
        writer: Sender<Frame>,
        is_admin: bool,
    },
    ConnFrame {
        id: u64,
        frame: Frame,
    },
    ConnClosed {
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
    shutdown: bool,
}

impl World {
    pub fn new(cfg: Config) -> Self {
        let mut journal = if cfg.data_dir.is_empty() {
            None
        } else {
            match Journal::new(&cfg.data_dir) {
                Ok(j) => Some(j),
                Err(e) => {
                    eprintln!("[ctpbuddy] journal disabled: {e}");
                    None
                }
            }
        };

        let mut engine_catalog = Catalog::builtin();
        let mut playback = None;
        let mut vt_day = dtime::today_trading_day();
        let mut vt_ms = dtime::now_ms_of_day();

        if let Some(dir) = &cfg.scenario_dir {
            let instruments = format!("{dir}/instruments.csv");
            if std::path::Path::new(&instruments).exists() {
                match Catalog::load_csv(&instruments) {
                    Ok(c) if !c.is_empty() => engine_catalog = c,
                    Ok(_) => {}
                    Err(e) => eprintln!("[ctpbuddy] instruments.csv load failed: {e}"),
                }
            }
            let ticks_path = format!("{dir}/ticks.csv");
            match CsvSource::load(&ticks_path) {
                Ok(ticks) => {
                    if let Some(first) = ticks.first() {
                        vt_day = first.trading_day.clone();
                        vt_ms = first.virtual_ms();
                    }
                    playback = Some(Playback::new(ticks, cfg.playback_speed));
                    println!("[ctpbuddy] ticks {}", playback.as_ref().unwrap().progress().1);
                }
                Err(e) => eprintln!("[ctpbuddy] ticks.csv load failed: {e}"),
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
                    ("scenario".into(), json::s(cfg.scenario_dir.as_deref().unwrap_or(""))),
                ]),
            );
            let _ = j.flush();
        }

        World {
            engine: MatchingEngine::new(engine_catalog),
            ledger: Ledger::new(cfg.initial_funds),
            playback,
            journal,
            conns: HashMap::new(),
            vt_trading_day: vt_day,
            vt_now_ms: vt_ms,
            orders_today: Vec::new(),
            trades_today: Vec::new(),
            shutdown: false,
            cfg,
        }
    }

    fn run_loop(&mut self, rx: Receiver<WorldMsg>) {
        loop {
            match rx.recv_timeout(PULSE) {
                Ok(msg) => match msg {
                    WorldMsg::ConnOpened { id, writer, is_admin } => self.on_conn_opened(id, writer, is_admin),
                    WorldMsg::ConnFrame { id, frame } => self.on_frame(id, frame),
                    WorldMsg::ConnClosed { id } => self.on_conn_closed(id),
                },
                Err(RecvTimeoutError::Timeout) => self.pulse(),
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
        self.ledger.mark_to_market(self.engine.catalog(), &prices);
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
                    self.ledger.unfreeze_order(&key);
                }
                self.orders_today.push(field.clone());
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
                self.ledger.on_fill(&fill, self.engine.catalog());
                self.trades_today.push(field.clone());
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
            ("status".into(), json::s(&(field.OrderStatus as char).to_string())),
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
