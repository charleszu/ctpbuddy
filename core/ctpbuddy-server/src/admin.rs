//! Admin control plane (JSON over ADMIN_REQ/ADMIN_RSP frames, port 5561).
//!
//! Commands: ping / status / settings_get / settings_update / start_scenario /
//! pause / resume / step / set_speed / seek / loop / reset_account /
//! settle_day / settlement_report / shutdown. State-changing commands a replay
//! must reproduce are journaled (scenario_loaded, reset_account, settlement,
//! settlement_report, settings_updated, server_stop); playback controls show
//! up through `md_watermark` instead.

use ctpbuddy_market::format_hhmmss;
use ctpbuddy_wire::generated::cstr;
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
            "settle_day" => self.admin_settle_day(conn_id, frame.req_id, &v),
            "settlement_report" => self.admin_settlement_report(conn_id, frame.req_id, &v),
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
                    ("balance".into(), json::n(a.dynamic_equity().to_f64())),
                    ("available".into(), json::n(a.available().to_f64())),
                    ("used_margin".into(), json::n(a.used_margin.to_f64())),
                    (
                        "position_profit".into(),
                        json::n(a.position_profit.to_f64()),
                    ),
                    ("commission".into(), json::n(a.commission.to_f64())),
                    ("risk".into(), json::n(a.risk())),
                ])
            })
            .collect();
        let v = json::obj_sorted(vec![
            ("ok".into(), json::b(true)),
            ("cmd".into(), json::s("status")),
            (
                "server".into(),
                json::s(&format!("ctpbuddy/{SERVER_VERSION}")),
            ),
            ("broker_id".into(), json::s(&self.cfg.broker_id)),
            ("scenario".into(), json::s(&self.scenario_name)),
            ("playback".into(), pb),
            (
                "instruments".into(),
                json::n(self.engine.catalog().len() as f64),
            ),
            (
                "open_orders".into(),
                json::n(self.engine.open_order_count() as f64),
            ),
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
        let bootstrap_positions = spec
            .as_ref()
            .map(|s| s.accounts.iter().map(|a| a.positions.len()).sum::<usize>())
            .unwrap_or(0);
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
                (
                    "bootstrap_positions".into(),
                    json::n(bootstrap_positions as f64),
                ),
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
        if !matches!(v, Value::Obj(fields) if fields.len() == 2 && fields.iter().filter(|(key, _)| key == "cmd").count() == 1 && fields.iter().filter(|(key, _)| key == "speed").count() == 1)
        {
            return self.admin_error(conn_id, req_id, "set_speed 仅允许 cmd 与 speed");
        }
        let Some(speed) = v.get("speed").and_then(Value::as_num) else {
            return self.admin_error(conn_id, req_id, "speed 必须为数字");
        };
        if !speed.is_finite() || !(0.0..=1000.0).contains(&speed) {
            return self.admin_error(
                conn_id,
                req_id,
                "speed 必须为 0–1000 的有限数字（0 为不限速）",
            );
        }
        let Some(pb) = self.playback.as_mut() else {
            return self.admin_error(conn_id, req_id, "未加载场景（先 start_scenario）");
        };
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

    fn admin_settlement_report(&mut self, conn_id: u64, req_id: u32, v: &Value) {
        let Some(Value::Arr(items)) = v.get("reports") else {
            return self.admin_error(conn_id, req_id, "settlement_report 需要 reports 数组");
        };
        let mut reports = Vec::new();
        for item in items {
            match crate::settlement::Report::parse(item, &self.cfg.broker_id) {
                Ok(r) => reports.push(r),
                Err(e) => return self.admin_error(conn_id, req_id, &e),
            }
        }
        let count = reports.len();
        if let Err(e) = self.save_reports(reports) {
            return self.admin_error(conn_id, req_id, &e);
        }
        self.admin_reply(
            conn_id,
            req_id,
            json::obj_sorted(vec![
                ("ok".into(), json::b(true)),
                ("cmd".into(), json::s("settlement_report")),
                ("saved".into(), json::n(count as f64)),
            ]),
        );
    }

    fn admin_settle_day(&mut self, conn_id: u64, req_id: u32, v: &Value) {
        let Some(next_day) = v.get_str("next_trading_day") else {
            return self.admin_error(conn_id, req_id, "settle_day 需要 next_trading_day");
        };
        let Some(prices) = v.get("settlement_prices").and_then(Value::as_obj) else {
            return self.admin_error(conn_id, req_id, "settle_day 需要 settlement_prices 对象");
        };
        if self.playback.as_ref().map(|p| !p.paused()).unwrap_or(false) {
            return self.admin_error(conn_id, req_id, "日结前必须暂停 playback");
        }
        if self.engine.open_order_count() != 0 {
            return self.admin_error(conn_id, req_id, "日结前不能存在活动订单");
        }
        let Some(fields) = v.as_obj() else {
            return self.admin_error(conn_id, req_id, "settle_day 请求必须为对象");
        };
        let mut seen = std::collections::HashSet::new();
        for (key, _) in fields {
            if !matches!(
                key.as_str(),
                "cmd" | "settlement_prices" | "next_trading_day"
            ) || !seen.insert(key)
            {
                return self.admin_error(conn_id, req_id, "settle_day 含未知或重复字段");
            }
        }
        let mut map = std::collections::HashMap::new();
        for (instrument, value) in prices {
            let Some(price) = value.as_num() else {
                return self.admin_error(
                    conn_id,
                    req_id,
                    &format!("结算价 {instrument} 必须为数字"),
                );
            };
            if map.insert(instrument.clone(), price).is_some() {
                return self.admin_error(conn_id, req_id, "结算价合约重复");
            }
        }
        let current_day = self.vt_trading_day.clone();
        let mut staged_ledger = self.ledger.clone();
        if let Err(e) =
            staged_ledger.settle_trading_day(self.engine.catalog(), &map, &current_day, &next_day)
        {
            return self.admin_error(conn_id, req_id, &e);
        }
        let mut staged_engine = self.engine.clone();
        staged_engine.advance_trading_day();
        let mut staged_playback = self.playback.clone();
        if let Some(pb) = staged_playback.as_mut() {
            pb.advance_trading_day(&next_day);
        }
        let mut accounts: Vec<_> = staged_ledger.accounts().collect();
        accounts.sort_by(|a, b| {
            a.broker_id
                .cmp(&b.broker_id)
                .then(a.investor_id.cmp(&b.investor_id))
        });
        let account_results = Value::Arr(
            accounts
                .iter()
                .map(|a| {
                    json::obj_sorted(vec![
                        ("broker".into(), json::s(&a.broker_id)),
                        ("investor".into(), json::s(&a.investor_id)),
                        ("pre_balance".into(), json::n(a.pre_balance.to_f64())),
                        ("used_margin".into(), json::n(a.used_margin.to_f64())),
                        (
                            "positions".into(),
                            Value::Arr(
                                staged_ledger
                                    .positions_of_ordered(&a.broker_id, &a.investor_id)
                                    .iter()
                                    .map(|(p, d)| {
                                        json::obj_sorted(vec![
                                            ("instrument".into(), json::s(&p.instrument_id)),
                                            (
                                                "side".into(),
                                                json::s(
                                                    if p.side == ctpbuddy_ledger::PositionSide::Long
                                                    {
                                                        "long"
                                                    } else {
                                                        "short"
                                                    },
                                                ),
                                            ),
                                            ("open_date".into(), json::s(&d.open_date)),
                                            ("trade_id".into(), json::s(&d.trade_id)),
                                            ("open_price".into(), json::n(d.open_price)),
                                            ("volume".into(), json::n(d.volume as f64)),
                                            (
                                                "last_settlement_price".into(),
                                                json::n(d.last_settlement_price),
                                            ),
                                        ])
                                    })
                                    .collect(),
                            ),
                        ),
                    ])
                })
                .collect(),
        );
        let event = json::obj_sorted(vec![
            ("from_trading_day".into(), json::s(&current_day)),
            ("next_trading_day".into(), json::s(&next_day)),
            (
                "settlement_prices".into(),
                json::obj_sorted(prices.to_vec()),
            ),
            ("accounts".into(), account_results),
            (
                "cleared_orders".into(),
                json::n(self.orders_today.len() as f64),
            ),
            (
                "cleared_trades".into(),
                json::n(self.trades_today.len() as f64),
            ),
            (
                "cleared_confirmations".into(),
                json::n(self.settlement_confirmed.len() as f64),
            ),
        ]);
        let generated = World::minimal_reports(&staged_ledger, &current_day)
            .into_iter()
            .filter(|r| {
                !self.settlement_reports.iter().any(|old| {
                    old.broker == r.broker && old.investor == r.investor && old.day == r.day
                })
            })
            .collect();
        if let Err(e) = self.save_reports(generated) {
            return self.admin_error(conn_id, req_id, &format!("保存结算报告失败: {e}"));
        }
        self.ledger = staged_ledger;
        self.engine = staged_engine;
        self.playback = staged_playback;
        self.journal_record_json("settlement", &self.cfg.broker_id.clone(), "", event);
        self.clear_day_flow();
        self.settlement_confirmed.clear();
        self.vt_trading_day = next_day.clone();
        self.persist_ledger();
        self.admin_reply(
            conn_id,
            req_id,
            json::obj_sorted(vec![
                ("ok".into(), json::b(true)),
                ("cmd".into(), json::s("settle_day")),
                ("trading_day".into(), json::s(&next_day)),
            ]),
        );
    }

    /// Active orders of `investor` still resting on the engine books
    /// (`""` = any investor).
    fn active_order_count_for(&self, investor: &str) -> usize {
        if investor.is_empty() {
            return self.engine.open_order_count();
        }
        self.engine
            .active_order_fields(&self.vt_trading_day)
            .iter()
            .filter(|o| cstr(&o.InvestorID) == investor)
            .count()
    }

    fn admin_reset_account(&mut self, conn_id: u64, req_id: u32, v: &Value) {
        let investor = v.get_str("investor").unwrap_or_default();
        let broker = self.cfg.broker_id.clone();
        // Same gate as `settle_day` / `start_scenario`: the ledger is wiped
        // but the engine books are not, so a later fill against a resting
        // close order would hit a position the ledger no longer knows.
        let active = self.active_order_count_for(&investor);
        if active != 0 {
            return self.admin_error(
                conn_id,
                req_id,
                &format!(
                    "重置账户前必须先撤销{}全部活动订单（当前 {active} 笔挂单仍在撮合簿上，重置后其成交将无法入账）",
                    if investor.is_empty() {
                        String::new()
                    } else {
                        format!("账户 {investor} 的")
                    }
                ),
            );
        }
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
                (
                    "reset".into(),
                    Value::Arr(ids.iter().map(|id| json::s(id)).collect()),
                ),
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
