//! Admin control plane (JSON over ADMIN_REQ/ADMIN_RSP frames, port 5561).
//!
//! Commands: ping / status / start_scenario / pause / resume / step /
//! set_speed / seek / loop / reset_account / shutdown. All commands are
//! journaled.

use ctpbuddy_market::format_hhmmss;
use ctpbuddy_wire::msgs;
use ctpbuddy_wire::Frame;

use crate::json::{self, Value};
use crate::scenario;
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
            "settings_get" => {
                if !matches!(&v, Value::Obj(p) if p.len() == 1) {
                    return self.admin_error(conn_id, frame.req_id, "settings_get 仅允许 cmd");
                }
                self.admin_reply(conn_id, frame.req_id, self.settings_reply());
            }
            "settings_update" => match self.update_settings(&v) {
                Ok(reply) => self.admin_reply(conn_id, frame.req_id, reply),
                Err(e) => self.admin_error(conn_id, frame.req_id, &e),
            },
            "start_scenario" => self.admin_start_scenario(conn_id, frame.req_id, &v),
            "pause" | "resume" | "step" => self.admin_playback_ctl(conn_id, frame.req_id, &cmd),
            "set_speed" => self.admin_set_speed(conn_id, frame.req_id, &v),
            "seek" => self.admin_seek(conn_id, frame.req_id, &v),
            "loop" => self.admin_loop(conn_id, frame.req_id, &v),
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
            (
                "looping".into(),
                json::b(self.playback.as_ref().map(|p| p.looping()).unwrap_or(false)),
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
            ("scenario".into(), json::s(&self.scenario_name)),
            ("playback".into(), pb),
            ("instruments".into(), json::n(self.engine.catalog().len() as f64)),
            ("open_orders".into(), json::n(self.engine.open_order_count() as f64)),
            ("connections".into(), json::n(self.conns.len() as f64)),
            (
                // Id the NEXT accepted connection will get (a global counter,
                // never reused — closed connections leave gaps). The replay
                // driver aligns its dummy connections on this to reproduce
                // recorded front_ids (DESIGN §11.2).
                "next_conn_id".into(),
                json::n(crate::next_conn_id() as f64),
            ),
            (
                "journal_seq".into(),
                json::n(self.journal.as_ref().map(|j| j.seq()).unwrap_or(0) as f64),
            ),
            ("accounts".into(), Value::Arr(accounts)),
            ("assertions".into(), self.assertions_status()),
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
        // Optional normalized spec: scenario.yaml parsed/validated by the
        // Python control plane (DESIGN §7.4). Absent = legacy ticks.csv-only.
        let spec = match v.get("spec") {
            Some(sv) => match scenario::parse_spec(sv) {
                Ok(s) => Some(s),
                Err(e) => return self.admin_error(conn_id, req_id, &e),
            },
            None => None,
        };
        let paused = v.get_bool("paused").unwrap_or(false);
        let speed = v.get_num("speed");
        let (n, day, name) = match self.apply_scenario(&path, spec.as_ref(), paused, speed) {
            Ok(r) => r,
            Err(e) => return self.admin_error(conn_id, req_id, &e),
        };
        let n_transforms = spec.as_ref().map(|s| s.transforms.len()).unwrap_or(0);
        let n_accounts = spec.as_ref().map(|s| s.accounts.len()).unwrap_or(0);
        let n_assertions = spec.as_ref().map(|s| s.assertions.len()).unwrap_or(0);
        let speed_now = self.playback.as_ref().map(|p| p.speed()).unwrap_or(0.0);
        self.journal_record_json(
            "scenario_loaded",
            "",
            "",
            json::obj_sorted(vec![
                ("path".into(), json::s(&path)),
                ("name".into(), json::s(&name)),
                ("ticks".into(), json::n(n as f64)),
                ("paused".into(), json::b(paused)),
                ("speed".into(), json::n(speed_now)),
                ("transforms".into(), json::n(n_transforms as f64)),
                ("accounts".into(), json::n(n_accounts as f64)),
                ("assertions".into(), json::n(n_assertions as f64)),
            ]),
        );
        let (passed, failed) = self.assertion_counts();
        let reply = json::obj_sorted(vec![
            ("ok".into(), json::b(true)),
            ("cmd".into(), json::s("start_scenario")),
            ("path".into(), json::s(&path)),
            ("name".into(), json::s(&name)),
            ("ticks".into(), json::n(n as f64)),
            ("trading_day".into(), json::s(&day)),
            ("paused".into(), json::b(paused)),
            ("speed".into(), json::n(speed_now)),
            ("assertions_passed".into(), json::n(passed as f64)),
            ("assertions_failed".into(), json::n(failed as f64)),
        ]);
        self.admin_reply(conn_id, req_id, reply);
    }

    fn admin_seek(&mut self, conn_id: u64, req_id: u32, v: &Value) {
        let at_ms = match v.get("at") {
            Some(Value::Num(n)) => *n,
            Some(Value::Str(s)) => match scenario::parse_hms_ms(s) {
                Some(ms) => ms,
                None => {
                    return self.admin_error(
                        conn_id,
                        req_id,
                        &format!("无法解析 at: '{s}'（期望 HH:MM:SS 或 ms 数字）"),
                    )
                }
            },
            _ => return self.admin_error(conn_id, req_id, "seek 需要 at（HH:MM:SS 或 ms 数字）"),
        };
        let (idx, total, vt) = {
            let Some(pb) = self.playback.as_mut() else {
                return self.admin_error(conn_id, req_id, "未加载场景（先 start_scenario）");
            };
            pb.seek(at_ms);
            let (idx, total) = pb.progress();
            (idx, total, pb.virtual_time())
        };
        self.vt_now_ms = vt;
        self.admin_reply(
            conn_id,
            req_id,
            json::obj_sorted(vec![
                ("ok".into(), json::b(true)),
                ("cmd".into(), json::s("seek")),
                ("at_ms".into(), json::n(at_ms)),
                ("idx".into(), json::n(idx as f64)),
                ("total".into(), json::n(total as f64)),
                ("virtual_time".into(), json::s(&format_hhmmss(vt))),
            ]),
        );
    }

    fn admin_loop(&mut self, conn_id: u64, req_id: u32, v: &Value) {
        let looping = {
            let Some(pb) = self.playback.as_mut() else {
                return self.admin_error(conn_id, req_id, "未加载场景（先 start_scenario）");
            };
            let on = v.get_bool("on").unwrap_or(true);
            pb.set_loop(on);
            pb.looping()
        };
        self.admin_reply(
            conn_id,
            req_id,
            json::obj_sorted(vec![
                ("ok".into(), json::b(true)),
                ("cmd".into(), json::s("loop")),
                ("looping".into(), json::b(looping)),
            ]),
        );
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
