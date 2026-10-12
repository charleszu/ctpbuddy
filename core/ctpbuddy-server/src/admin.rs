//! Admin control plane (JSON over ADMIN_REQ/ADMIN_RSP frames, port 5561).
//!
//! Commands: ping / status / settings_get / settings_update / start_scenario /
//! pause / resume / step / set_speed / seek / loop / deposit / withdraw / reset_account /
//! settle_day / settlement_report / announce / shutdown. State-changing commands a replay
//! must reproduce are journaled (scenario_loaded, reset_account, settlement,
//! settlement_report, settings_updated, deposit, withdraw, server_stop); playback controls show
//! up through `md_watermark` instead.

use ctpbuddy_market::format_hhmmss;
use ctpbuddy_wire::generated::{cstr, set_cstr, CThostFtdcOrderField, CThostFtdcTradeField};
use ctpbuddy_wire::msgs;
use ctpbuddy_wire::{struct_to_bytes, Frame};

use crate::json::{self, Value};
use crate::scenario;
use crate::{journal_contract, PrivateNotification, World, SERVER_VERSION};
use ctpbuddy_ledger::CashTransferError;
use ctpbuddy_matching::{Direction, Fill, OffsetFlag};

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
            "deposit" => self.admin_cash_transfer(conn_id, frame.req_id, &v, false),
            "withdraw" => self.admin_cash_transfer(conn_id, frame.req_id, &v, true),
            "reset_account" => self.admin_reset_account(conn_id, frame.req_id, &v),
            "settle_day" => self.admin_settle_day(conn_id, frame.req_id, &v),
            "settlement_report" => self.admin_settlement_report(conn_id, frame.req_id, &v),
            "announce" => self.admin_announce(conn_id, frame.req_id, &v),
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
        // ADMIN status is a live read model. Revalue the ledger immediately so
        // Web/CLI consumers see the same position profit as CTP queries.
        self.mark_to_market_now();
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
        let mut account_rows: Vec<_> = self.ledger.accounts().collect();
        account_rows.sort_by(|a, b| {
            a.broker_id
                .cmp(&b.broker_id)
                .then(a.investor_id.cmp(&b.investor_id))
                .then(a.currency_id.cmp(&b.currency_id))
        });
        let accounts: Vec<Value> = account_rows
            .into_iter()
            .map(|a| {
                json::obj_sorted(vec![
                    ("broker".into(), json::s(&a.broker_id)),
                    ("investor".into(), json::s(&a.investor_id)),
                    ("currency".into(), json::s(&a.currency_id)),
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
        let mut position_rows = Vec::new();
        for account in self.ledger.accounts() {
            for (position, detail) in self
                .ledger
                .positions_of_ordered(&account.broker_id, &account.investor_id)
            {
                position_rows.push(json::obj_sorted(vec![
                    ("broker".into(), json::s(&account.broker_id)),
                    ("investor".into(), json::s(&account.investor_id)),
                    ("instrument".into(), json::s(&position.instrument_id)),
                    (
                        "side".into(),
                        json::s(if position.side == ctpbuddy_ledger::PositionSide::Long {
                            "long"
                        } else {
                            "short"
                        }),
                    ),
                    ("volume".into(), json::n(detail.volume as f64)),
                    (
                        "today_position".into(),
                        json::n(position.today_position as f64),
                    ),
                    ("yd_position".into(), json::n(position.yd_position as f64)),
                    ("open_date".into(), json::s(&detail.open_date)),
                    ("trade_id".into(), json::s(&detail.trade_id)),
                    ("open_price".into(), json::n(detail.open_price)),
                    (
                        "settlement_price".into(),
                        json::n(position.settlement_price),
                    ),
                    (
                        "position_profit".into(),
                        json::n(position.position_profit.to_f64()),
                    ),
                    ("margin".into(), json::n(position.margin.to_f64())),
                ]));
            }
        }
        let mut brokers: Vec<String> = self
            .cfg
            .broker_ids()
            .into_iter()
            .map(str::to_string)
            .collect();
        brokers.sort_unstable();
        brokers.dedup();
        let mut fields = vec![
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
            (
                "snapshot_journal_seq".into(),
                self.snapshot_journal_seq
                    .map(|seq| json::n(seq as f64))
                    .unwrap_or(Value::Null),
            ),
            (
                "contract_version".into(),
                json::s(&journal_contract::contract_version().to_string()),
            ),
            ("accounts".into(), Value::Arr(accounts)),
            ("positions".into(), Value::Arr(position_rows)),
            (
                "pubsub".into(),
                json::obj_sorted(vec![
                    ("enabled".into(), json::b(self.broadcast_tx.is_some())),
                    (
                        "queue_capacity".into(),
                        json::n(crate::ZMQ_PUB_QUEUE_CAPACITY as f64),
                    ),
                    ("published".into(), json::n(self.broadcast_published as f64)),
                    ("dropped".into(), json::n(self.broadcast_dropped as f64)),
                ]),
            ),
            (
                "brokers".into(),
                Value::Arr(brokers.iter().map(|broker| json::s(broker)).collect()),
            ),
            ("assertions".into(), self.assertions_status()),
        ];
        if let Some(calendar) = &self.cfg.calendar {
            fields.push(("calendar_version".into(), json::s(&calendar.version)));
            fields.push(("calendar_sha256".into(), json::s(&calendar.sha256)));
        }
        let v = json::obj_sorted(fields);
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
            let broker = match item.get_str("broker") {
                Some(value) if self.cfg.supports_broker(&value) => value,
                Some(value) => {
                    return self.admin_error(
                        conn_id,
                        req_id,
                        &format!("不支持的 BrokerID: {value}"),
                    )
                }
                None => return self.admin_error(conn_id, req_id, "报告缺少 broker"),
            };
            match crate::settlement::Report::parse(item, &broker) {
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

    fn admin_cash_transfer(&mut self, conn_id: u64, req_id: u32, v: &Value, withdraw: bool) {
        let command = if withdraw { "withdraw" } else { "deposit" };
        let Some(fields) = v.as_obj() else {
            return self.admin_error(conn_id, req_id, &format!("{command} 请求必须为对象"));
        };
        let mut seen = std::collections::HashSet::new();
        for (key, _) in fields {
            if !matches!(key.as_str(), "cmd" | "broker" | "investor" | "amount")
                || !seen.insert(key)
            {
                return self.admin_error(conn_id, req_id, &format!("{command} 含未知或重复字段"));
            }
        }
        let broker = match v.get_str("broker") {
            Some(value) if !value.is_empty() && self.cfg.supports_broker(&value) => value,
            Some(value) if value.is_empty() => {
                return self.admin_error(conn_id, req_id, "broker 不能为空")
            }
            Some(value) => {
                return self.admin_error(conn_id, req_id, &format!("不支持的 BrokerID: {value}"))
            }
            None => return self.admin_error(conn_id, req_id, "缺少 broker"),
        };
        let investor = match v.get_str("investor") {
            Some(value) if !value.is_empty() => value,
            Some(_) => return self.admin_error(conn_id, req_id, "investor 不能为空"),
            None => return self.admin_error(conn_id, req_id, "缺少 investor"),
        };
        let amount = match v.get("amount").and_then(Value::as_num) {
            Some(value) if value.is_finite() && value > 0.0 => value,
            _ => return self.admin_error(conn_id, req_id, "amount 必须为有限正数"),
        };
        let result = if withdraw {
            self.ledger.apply_withdraw(&broker, &investor, amount)
        } else {
            self.ledger.apply_deposit(&broker, &investor, amount)
        };
        if let Err(error) = result {
            let message = match error {
                CashTransferError::InvalidAmount => "amount 必须为有限正数",
                CashTransferError::InsufficientFunds => "可用资金不足",
            };
            return self.admin_error(conn_id, req_id, message);
        }
        self.journal_record_json(
            command,
            &broker,
            &investor,
            json::obj_sorted(vec![("amount".into(), json::n(amount))]),
        );
        self.admin_reply(
            conn_id,
            req_id,
            json::obj_sorted(vec![
                ("ok".into(), json::b(true)),
                ("cmd".into(), json::s(command)),
                ("broker".into(), json::s(&broker)),
                ("investor".into(), json::s(&investor)),
                ("amount".into(), json::n(amount)),
            ]),
        );
    }

    fn admin_settle_day(&mut self, conn_id: u64, req_id: u32, v: &Value) {
        let next_day = match v.get_str("next_trading_day") {
            Some(day) => day.to_string(),
            None => match self
                .cfg
                .calendar_snapshot
                .as_ref()
                .and_then(|calendar| calendar.next_trading_day(&self.vt_trading_day).ok())
            {
                Some(day) => day,
                None => {
                    return self.admin_error(
                        conn_id,
                        req_id,
                        "settle_day 需要 next_trading_day（或配置完整离线日历）",
                    )
                }
            },
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
        let mut brokers: Vec<String> = self
            .cfg
            .broker_ids()
            .into_iter()
            .map(str::to_string)
            .collect();
        brokers.sort_unstable();
        let mut staged_ledger = self.ledger.clone();
        let forced = match self.stage_expired_liquidations(&map, &current_day, &mut staged_ledger) {
            Ok(forced) => forced,
            Err(e) => return self.admin_error(conn_id, req_id, &e),
        };
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
            ("forced_liquidations".into(), json::n(forced.len() as f64)),
            (
                "cleared_confirmations".into(),
                json::n(self.settlement_confirmed.len() as f64),
            ),
            (
                "brokers".into(),
                Value::Arr(brokers.iter().map(|broker| json::s(broker)).collect()),
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
        self.record_forced_liquidations(&forced);
        self.engine = staged_engine;
        self.playback = staged_playback;
        let broker = self.cfg.primary_broker().to_string();
        self.journal_record_json("settlement", &broker, "", event);
        let sys_event = json::obj_sorted(vec![
            ("type".into(), json::s("settlement_complete")),
            ("from_trading_day".into(), json::s(&current_day)),
            ("trading_day".into(), json::s(&next_day)),
            ("forced_liquidations".into(), json::n(forced.len() as f64)),
            (
                "brokers".into(),
                Value::Arr(brokers.iter().map(|broker| json::s(broker)).collect()),
            ),
        ]);
        self.publish(crate::Broadcast {
            topic: "sys/settlement".into(),
            frame: Frame::new(msgs::ADMIN_RSP, 0, sys_event.to_json().into_bytes()),
        });
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
                ("forced_liquidations".into(), json::n(forced.len() as f64)),
                (
                    "brokers".into(),
                    Value::Arr(brokers.iter().map(|broker| json::s(broker)).collect()),
                ),
            ]),
        );
    }

    fn admin_announce(&mut self, conn_id: u64, req_id: u32, v: &Value) {
        let Some(fields) = v.as_obj() else {
            return self.admin_error(conn_id, req_id, "announce 请求必须为对象");
        };
        let Some(message) = v.get_str("message") else {
            return self.admin_error(conn_id, req_id, "announce 需要 message");
        };
        let severity = v.get_str("severity").unwrap_or_else(|| "info".to_string());
        let mut seen = std::collections::HashSet::new();
        for (key, _) in fields {
            if !matches!(key.as_str(), "cmd" | "message" | "severity") || !seen.insert(key) {
                return self.admin_error(conn_id, req_id, "announce 含未知或重复字段");
            }
        }
        if message.is_empty() || message.len() > 4096 {
            return self.admin_error(conn_id, req_id, "announce message 长度必须为 1..4096");
        }
        if !matches!(severity.as_str(), "info" | "warning" | "error") {
            return self.admin_error(
                conn_id,
                req_id,
                "announce severity 必须为 info/warning/error",
            );
        }
        let payload = json::obj_sorted(vec![
            ("type".into(), json::s("announcement")),
            ("severity".into(), json::s(&severity)),
            ("message".into(), json::s(&message)),
            ("trading_day".into(), json::s(&self.vt_trading_day)),
        ]);
        self.journal_record_json("system_announcement", "", "", payload.clone());
        self.publish(crate::Broadcast {
            topic: "sys/announcement".into(),
            frame: Frame::new(msgs::ADMIN_RSP, 0, payload.to_json().into_bytes()),
        });
        self.admin_reply(
            conn_id,
            req_id,
            json::obj_sorted(vec![
                ("ok".into(), json::b(true)),
                ("cmd".into(), json::s("announce")),
            ]),
        );
    }

    /// Stage expiry closes against a cloned ledger. Missing or invalid prices
    /// are rejected before the live world is changed.
    fn stage_expired_liquidations(
        &self,
        prices: &std::collections::HashMap<String, f64>,
        trading_day: &str,
        staged: &mut ctpbuddy_ledger::Ledger,
    ) -> Result<Vec<(CThostFtdcOrderField, Fill, CThostFtdcTradeField, f64)>, String> {
        struct Candidate {
            broker: String,
            investor: String,
            instrument: String,
            exchange: String,
            direction: Direction,
            offset: OffsetFlag,
            volume: i32,
            price: f64,
            pre_settlement: f64,
        }
        let accounts: Vec<(String, String)> = self
            .ledger
            .accounts()
            .map(|a| (a.broker_id.clone(), a.investor_id.clone()))
            .collect();
        let mut candidates: Vec<Candidate> = Vec::new();
        for (broker, investor) in accounts {
            for position in self.ledger.positions_of(&broker, &investor) {
                let Some(instrument) = self.engine.catalog().get(&position.instrument_id) else {
                    continue;
                };
                if position.volume() <= 0
                    || instrument.expire_date.is_empty()
                    || instrument.expire_date.as_str() > trading_day
                {
                    continue;
                }
                let price = prices
                    .get(&position.instrument_id)
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "到期合约 {} 缺少结算价，自动强平已拒绝",
                            position.instrument_id
                        )
                    })?;
                if !price.is_finite() || price <= 0.0 {
                    return Err(format!(
                        "到期合约 {} 的强平价必须为正有限数",
                        position.instrument_id
                    ));
                }
                let direction = match position.side {
                    ctpbuddy_ledger::PositionSide::Long => Direction::Sell,
                    ctpbuddy_ledger::PositionSide::Short => Direction::Buy,
                };
                let rules = ctpbuddy_matching::rules_for(&instrument.exchange_id);
                if rules.splits_today_yesterday {
                    if position.today_position > 0 {
                        candidates.push(Candidate {
                            broker: broker.clone(),
                            investor: investor.clone(),
                            instrument: position.instrument_id.clone(),
                            exchange: instrument.exchange_id.clone(),
                            direction,
                            offset: OffsetFlag::CloseToday,
                            volume: position.today_position,
                            price,
                            pre_settlement: position.pre_settlement_price,
                        });
                    }
                    if position.yd_position > 0 {
                        candidates.push(Candidate {
                            broker: broker.clone(),
                            investor: investor.clone(),
                            instrument: position.instrument_id.clone(),
                            exchange: instrument.exchange_id.clone(),
                            direction,
                            offset: OffsetFlag::CloseYesterday,
                            volume: position.yd_position,
                            price,
                            pre_settlement: position.pre_settlement_price,
                        });
                    }
                } else {
                    candidates.push(Candidate {
                        broker: broker.clone(),
                        investor: investor.clone(),
                        instrument: position.instrument_id.clone(),
                        exchange: instrument.exchange_id.clone(),
                        direction,
                        offset: OffsetFlag::Close,
                        volume: position.volume(),
                        price,
                        pre_settlement: position.pre_settlement_price,
                    });
                }
            }
        }
        candidates.sort_by(|a, b| {
            a.broker
                .cmp(&b.broker)
                .then(a.investor.cmp(&b.investor))
                .then(a.instrument.cmp(&b.instrument))
                .then(a.offset.as_ctp().cmp(&b.offset.as_ctp()))
        });
        let mut out = Vec::with_capacity(candidates.len());
        for (index, candidate) in candidates.into_iter().enumerate() {
            let Candidate {
                broker,
                investor,
                instrument,
                exchange,
                direction,
                offset,
                volume,
                price,
                pre_settlement,
            } = candidate;
            let order_ref = format!("AUTOFC-{trading_day}-{}", index + 1);
            let order_sys_id = format!("AUTOFC-SYS-{}", index + 1);
            let trade_id = format!("AUTOFC-TRD-{}", index + 1);
            let mut fill = Fill {
                broker_id: [0; 11],
                investor_id: [0; 13],
                user_id: [0; 16],
                instrument_id: instrument.clone(),
                exchange_id: exchange.clone(),
                direction,
                offset,
                hedge_flag: b'1',
                price,
                volume,
                volume_total_original: volume,
                order_sys_id: [0; 21],
                order_ref: [0; 13],
                trade_id: [0; 21],
                order_key: format!("auto-force-close/{trading_day}/{index}"),
            };
            set_cstr(&mut fill.broker_id, &broker);
            set_cstr(&mut fill.investor_id, &investor);
            set_cstr(&mut fill.order_sys_id, &order_sys_id);
            set_cstr(&mut fill.order_ref, &order_ref);
            set_cstr(&mut fill.trade_id, &trade_id);
            staged.on_fill(&fill, self.engine.catalog(), pre_settlement, trading_day);

            let mut order = CThostFtdcOrderField::zeroed();
            set_cstr(&mut order.BrokerID, &broker);
            set_cstr(&mut order.InvestorID, &investor);
            set_cstr(&mut order.OrderRef, &order_ref);
            set_cstr(&mut order.OrderSysID, &order_sys_id);
            set_cstr(&mut order.InstrumentID, &instrument);
            set_cstr(&mut order.ExchangeID, &exchange);
            set_cstr(&mut order.TradingDay, trading_day);
            order.OrderPriceType = b'2';
            order.Direction = direction.as_ctp();
            order.CombOffsetFlag[0] = offset.as_ctp();
            order.CombHedgeFlag[0] = b'1';
            order.LimitPrice = price;
            order.VolumeTotalOriginal = volume;
            order.TimeCondition = b'3';
            order.VolumeCondition = b'1';
            // CTP's PersonDeliv reason ('7') identifies a delivery/expiry
            // close; it is distinct from a user order with reason '0'.
            order.ForceCloseReason = b'7';
            order.OrderSubmitStatus = b'3';
            order.OrderStatus = b'0';
            order.OrderType = b'0';
            order.VolumeTraded = volume;
            set_cstr(&mut order.StatusMsg, "到期合约自动强平");

            let mut trade = CThostFtdcTradeField::zeroed();
            set_cstr(&mut trade.BrokerID, &broker);
            set_cstr(&mut trade.InvestorID, &investor);
            set_cstr(&mut trade.OrderRef, &order_ref);
            set_cstr(&mut trade.OrderSysID, &order_sys_id);
            set_cstr(&mut trade.InstrumentID, &instrument);
            set_cstr(&mut trade.ExchangeID, &exchange);
            set_cstr(&mut trade.TradeID, &trade_id);
            set_cstr(&mut trade.TradingDay, trading_day);
            trade.Direction = direction.as_ctp();
            trade.OffsetFlag = offset.as_ctp();
            trade.HedgeFlag = b'1';
            trade.Price = price;
            trade.Volume = volume;
            out.push((order, fill, trade, pre_settlement));
        }
        Ok(out)
    }

    fn record_forced_liquidations(
        &mut self,
        events: &[(CThostFtdcOrderField, Fill, CThostFtdcTradeField, f64)],
    ) {
        let day = self.vt_trading_day.clone();
        for (order, fill, trade, pre_settlement) in events {
            self.private_seq.push((false, self.orders_today.len()));
            self.orders_today.push(*order);
            self.journal_record_order_update(&day, self.vt_now_ms, order);
            self.emit_private_notification(PrivateNotification {
                broker: cstr(&order.BrokerID),
                investor: cstr(&order.InvestorID),
                frame: Frame::new(msgs::RTN_ORDER, 0, struct_to_bytes(order)),
            });
            self.private_seq.push((true, self.trades_today.len()));
            self.trades_today.push(*trade);
            self.journal_record_fill(&day, self.vt_now_ms, fill, *pre_settlement);
            self.emit_private_notification(PrivateNotification {
                broker: cstr(&trade.BrokerID),
                investor: cstr(&trade.InvestorID),
                frame: Frame::new(msgs::RTN_TRADE, 0, struct_to_bytes(trade)),
            });
        }
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
        let broker = self.cfg.primary_broker().to_string();
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
