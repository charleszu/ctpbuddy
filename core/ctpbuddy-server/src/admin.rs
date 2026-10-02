//! Admin control plane (JSON over ADMIN_REQ/ADMIN_RSP frames, port 5561).
//!
//! Commands: ping / status / start_scenario / pause / resume / step /
//! set_speed / reset_account / shutdown. All commands are journaled.

use ctpbuddy_market::{CsvSource, Playback};
use ctpbuddy_matching::{Catalog, MatchingEngine};
use ctpbuddy_wire::msgs;
use ctpbuddy_wire::Frame;

use crate::json::{self, Value};
use crate::{World, SERVER_VERSION};

impl World {
    pub(crate) fn on_admin(&mut self, conn_id: u64, frame: &Frame) {
        let text = String::from_utf8_lossy(&frame.payload).to_string();
        let v = match json::parse(&text) {
            Ok(v) => v,
            Err(e) => {
                return self.admin_error(conn_id, frame.req_id, &format!("JSON 解析失败: {e}"))
            }
        };
        let cmd = v.get_str("cmd").unwrap_or_default();
        match cmd.as_str() {
            "ping" => {
                self.admin_reply(
                    conn_id,
                    frame.req_id,
                    json::obj_sorted(vec![
                        ("ok".into(), json::b(true)),
                        ("cmd".into(), json::s("ping")),
                        ("pong".into(), json::b(true)),
                    ]),
                );
            }
            "status" => self.admin_status(conn_id, frame.req_id),
            "start_scenario" => self.admin_start_scenario(conn_id, frame.req_id, &v),
            "pause" | "resume" | "step" => self.admin_playback_ctl(conn_id, frame.req_id, &cmd),
            "set_speed" => self.admin_set_speed(conn_id, frame.req_id, &v),
            "reset_account" => self.admin_reset_account(conn_id, frame.req_id, &v),
            "shutdown" => {
                self.admin_reply(
                    conn_id,
                    frame.req_id,
                    json::obj_sorted(vec![
                        ("ok".into(), json::b(true)),
                        ("cmd".into(), json::s("shutdown")),
                    ]),
                );
                self.journal_record_json("server_stop", "", "", Value::Null);
                self.set_shutdown();
            }
            "" => self.admin_error(conn_id, frame.req_id, "缺少 cmd 字段"),
            other => self.admin_error(conn_id, frame.req_id, &format!("未知命令 '{other}'")),
        }
    }

    fn admin_reply(&mut self, conn_id: u64, req_id: u32, v: Value) {
        let payload = v.to_json().into_bytes();
        self.send_frame(conn_id, Frame::new(msgs::ADMIN_RSP, req_id, payload));
    }

    fn admin_error(&mut self, conn_id: u64, req_id: u32, msg: &str) {
        let v = json::obj_sorted(vec![
            ("ok".into(), json::b(false)),
            ("error".into(), json::s(msg)),
        ]);
        self.admin_reply(conn_id, req_id, v);
    }

    fn admin_status(&mut self, conn_id: u64, req_id: u32) {
        let (idx, total) = self
            .playback
            .as_ref()
            .map(|p| p.progress())
            .unwrap_or((0, 0));
        let pb = json::obj_sorted(vec![
            ("loaded".into(), json::b(self.playback.is_some())),
            ("idx".into(), json::n(idx as f64)),
            ("total".into(), json::n(total as f64)),
            (
                "paused".into(),
                json::b(self.playback.as_ref().map(|p| p.paused()).unwrap_or(false)),
            ),
            (
                "speed".into(),
                json::n(self.playback.as_ref().map(|p| p.speed()).unwrap_or(0.0)),
            ),
            ("virtual_time".into(), json::s(&self.now_str())),
            ("trading_day".into(), json::s(&self.vt_day())),
        ]);
        let accounts: Vec<Value> = self
            .ledger
            .accounts()
            .map(|a| {
                json::obj_sorted(vec![
                    ("investor".into(), json::s(&a.investor_id)),
                    ("balance".into(), json::n(a.dynamic_equity())),
                    ("available".into(), json::n(a.available())),
                    ("used_margin".into(), json::n(a.used_margin)),
                    ("position_profit".into(), json::n(a.position_profit)),
                    ("commission".into(), json::n(a.commission)),
                    ("risk".into(), json::n(a.risk())),
                ])
            })
            .collect();
        let v = json::obj_sorted(vec![
            ("ok".into(), json::b(true)),
            ("cmd".into(), json::s("status")),
            ("server".into(), json::s(&format!("ctpbuddy/{SERVER_VERSION}"))),
            ("broker_id".into(), json::s(&self.cfg.broker_id)),
            ("playback".into(), pb),
            ("instruments".into(), json::n(self.engine.catalog().len() as f64)),
            ("open_orders".into(), json::n(self.engine.open_order_count() as f64)),
            ("connections".into(), json::n(self.conns.len() as f64)),
            (
                "journal_seq".into(),
                json::n(self.journal.as_ref().map(|j| j.seq()).unwrap_or(0) as f64),
            ),
            ("accounts".into(), Value::Arr(accounts)),
        ]);
        self.admin_reply(conn_id, req_id, v);
    }

    fn admin_start_scenario(&mut self, conn_id: u64, req_id: u32, v: &Value) {
        let path = v.get_str("path").unwrap_or_default();
        if path.is_empty() {
            return self.admin_error(conn_id, req_id, "path 不能为空");
        }
        if !std::path::Path::new(&path).is_dir() {
            return self.admin_error(conn_id, req_id, &format!("场景目录不存在: {path}"));
        }
        // instruments.csv (optional; builtin fallback)
        let instruments = format!("{path}/instruments.csv");
        let mut catalog = Catalog::builtin();
        if std::path::Path::new(&instruments).exists() {
            match Catalog::load_csv(&instruments) {
                Ok(c) if !c.is_empty() => catalog = c,
                Ok(_) => {}
                Err(e) => return self.admin_error(conn_id, req_id, &format!("instruments.csv: {e}")),
            }
        }
        // ticks.csv (required)
        let ticks_path = format!("{path}/ticks.csv");
        let ticks = match CsvSource::load(&ticks_path) {
            Ok(t) => t,
            Err(e) => return self.admin_error(conn_id, req_id, &format!("ticks.csv: {e}")),
        };
        let n = ticks.len();
        let paused = v.get_bool("paused").unwrap_or(false);
        let speed = v.get_num("speed").unwrap_or(self.cfg.playback_speed);
        let (day, t0) = ticks
            .first()
            .map(|t| (t.trading_day.clone(), t.virtual_ms()))
            .unwrap_or_else(|| (self.vt_day(), 0.0));

        // NOTE: swapping the scenario drops active orders; their ledger freezes
        // are released with `reset_account` (M2: cancel-all admin command).
        self.engine = MatchingEngine::new(catalog);
        self.orders_today.clear();
        self.trades_today.clear();
        let mut pb = Playback::new(ticks, speed);
        if paused {
            pb.pause();
        }
        self.playback = Some(pb);
        self.vt_trading_day = day.clone();
        self.vt_now_ms = t0;

        self.journal_record_json(
            "scenario_loaded",
            "",
            "",
            json::obj_sorted(vec![
                ("path".into(), json::s(&path)),
                ("ticks".into(), json::n(n as f64)),
                ("paused".into(), json::b(paused)),
                ("speed".into(), json::n(speed)),
            ]),
        );
        let reply = json::obj_sorted(vec![
            ("ok".into(), json::b(true)),
            ("cmd".into(), json::s("start_scenario")),
            ("path".into(), json::s(&path)),
            ("ticks".into(), json::n(n as f64)),
            ("trading_day".into(), json::s(&day)),
            ("paused".into(), json::b(paused)),
            ("speed".into(), json::n(speed)),
        ]);
        self.admin_reply(conn_id, req_id, reply);
    }

    fn admin_playback_ctl(&mut self, conn_id: u64, req_id: u32, cmd: &str) {
        let has = self.playback.is_some();
        if !has {
            return self.admin_error(conn_id, req_id, "未加载场景（先 start_scenario）");
        }
        let paused = {
            let pb = self.playback.as_mut().unwrap();
            match cmd {
                "pause" => pb.pause(),
                "resume" => pb.resume(),
                "step" => pb.step(),
                _ => {}
            }
            pb.paused()
        };
        self.admin_reply(
            conn_id,
            req_id,
            json::obj_sorted(vec![
                ("ok".into(), json::b(true)),
                ("cmd".into(), json::s(cmd)),
                ("paused".into(), json::b(paused)),
            ]),
        );
    }

    fn admin_set_speed(&mut self, conn_id: u64, req_id: u32, v: &Value) {
        let Some(pb) = self.playback.as_mut() else {
            return self.admin_error(conn_id, req_id, "未加载场景（先 start_scenario）");
        };
        let speed = v.get_num("speed").unwrap_or(0.0);
        pb.set_speed(speed);
        self.admin_reply(
            conn_id,
            req_id,
            json::obj_sorted(vec![
                ("ok".into(), json::b(true)),
                ("cmd".into(), json::s("set_speed")),
                ("speed".into(), json::n(speed)),
            ]),
        );
    }

    fn admin_reset_account(&mut self, conn_id: u64, req_id: u32, v: &Value) {
        let investor = v.get_str("investor").unwrap_or_default();
        let broker = self.cfg.broker_id.clone();
        if investor.is_empty() {
            // reset every account
            let ids: Vec<String> = self
                .ledger
                .accounts()
                .map(|a| a.investor_id.clone())
                .collect();
            for id in &ids {
                self.ledger.reset_account(&broker, id);
            }
            self.journal_record_json(
                "reset_account",
                &broker,
                "",
                json::obj_sorted(vec![("all".into(), json::b(true))]),
            );
            let reply = json::obj_sorted(vec![
                ("ok".into(), json::b(true)),
                ("cmd".into(), json::s("reset_account")),
                ("reset".into(), Value::Arr(ids.iter().map(|id| json::s(id)).collect())),
            ]);
            self.admin_reply(conn_id, req_id, reply);
        } else {
            self.ledger.reset_account(&broker, &investor);
            self.journal_record_json(
                "reset_account",
                &broker,
                &investor,
                json::obj_sorted(vec![("all".into(), json::b(false))]),
            );
            let reply = json::obj_sorted(vec![
                ("ok".into(), json::b(true)),
                ("cmd".into(), json::s("reset_account")),
                ("investor".into(), json::s(&investor)),
            ]);
            self.admin_reply(conn_id, req_id, reply);
        }
    }
}
