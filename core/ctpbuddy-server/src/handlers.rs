//! CTP message handlers: frame -> SPI-equivalent callbacks.
//!
//! Wire conventions implemented here (documented in `ctpbuddy-wire::msgs`):
//! - success `RSP_*` = response struct payload, `RspInfo` implied zero;
//! - failed request = `RSP_ERROR` (RspInfoField), completing the pending
//!   request as `OnRsp*(NULL, rsp, is_last=true)`;
//! - order-insert failures = `ERR_RTN_ORDER_INSERT` (OnErrRtnOrderInsert);
//! - query streams terminate with `QRY_LAST` (CTP `bIsLast`).

use std::collections::HashMap;
use std::mem::size_of;
use std::time::{Duration, Instant};

use ctpbuddy_ledger::PositionSide;
use ctpbuddy_matching::{CancelQuery, ClockCtx, Direction, OffsetFlag, OrderIntent, SubmitOutcome};
use ctpbuddy_wire::generated::{
    cstr, set_cstr, CThostFtdcInputOrderActionField, CThostFtdcInputOrderField,
    CThostFtdcQryInstrumentField, CThostFtdcQryInvestorPositionField, CThostFtdcQryOrderField,
    CThostFtdcQryTradeField, CThostFtdcReqUserLoginField,
    CThostFtdcRspUserLoginField, CThostFtdcSettlementInfoConfirmField,
    CThostFtdcSpecificInstrumentField, CThostFtdcUserLogoutField,
};
use ctpbuddy_wire::msgs;
use ctpbuddy_wire::{struct_from_bytes, struct_to_bytes};

use crate::json::{self, Value};
use crate::{Frame, SERVER_NAME, SERVER_VERSION, World};

impl World {
    pub(crate) fn on_frame(&mut self, conn_id: u64, frame: Frame) {
        let is_admin = match self.conns.get(&conn_id) {
            Some(c) => c.is_admin,
            None => return,
        };
        if is_admin {
            self.on_admin(conn_id, &frame);
            return;
        }
        match frame.msg_type {
            msgs::PING => {
                self.send_frame(conn_id, Frame::new(msgs::PONG, frame.req_id, frame.payload.clone()))
            }
            msgs::AUTH => self.on_auth(conn_id, &frame),
            msgs::REQ_USER_LOGIN => self.on_login(conn_id, &frame),
            msgs::REQ_USER_LOGOUT => self.on_logout(conn_id, &frame),
            msgs::REQ_SETTLE_CONFIRM => self.on_settle_confirm(conn_id, &frame),
            msgs::REQ_ORDER_INSERT => self.on_order_insert(conn_id, &frame),
            msgs::REQ_ORDER_ACTION => self.on_order_action(conn_id, &frame),
            msgs::SUB_MD => self.on_sub_md(conn_id, &frame, true),
            msgs::UNSUB_MD => self.on_sub_md(conn_id, &frame, false),
            msgs::REQ_QRY_INSTRUMENT => self.on_qry_instrument(conn_id, &frame),
            msgs::REQ_QRY_TRADING_ACCOUNT => self.on_qry_trading_account(conn_id, &frame),
            msgs::REQ_QRY_INVESTOR_POSITION => self.on_qry_investor_position(conn_id, &frame),
            msgs::REQ_QRY_ORDER => self.on_qry_order(conn_id, &frame),
            msgs::REQ_QRY_TRADE => self.on_qry_trade(conn_id, &frame),
            other => {
                eprintln!("[ctpbuddy] conn {conn_id}: unsupported msg 0x{other:04x}");
                self.send_error(conn_id, frame.req_id, -1, "不支持的消息类型")
            }
        }
    }

    // ---- session helpers ----

    fn session(&self, conn_id: u64) -> Option<(String, String, [u8; 16], i32, i32)> {
        let c = self.conns.get(&conn_id)?;
        if !c.authenticated || c.investor_id.is_none() {
            return None;
        }
        Some((
            c.broker_id.clone()?,
            c.investor_id.clone()?,
            c.user_id,
            c.front_id,
            c.session_id,
        ))
    }

    fn send_err_rtn(&mut self, conn_id: u64, req_id: u32, error_id: i32, msg: &str) {
        let mut f = ctpbuddy_wire::generated::CThostFtdcRspInfoField::zeroed();
        f.ErrorID = error_id;
        set_cstr(&mut f.ErrorMsg, msg);
        self.send_frame(
            conn_id,
            Frame::new(msgs::ERR_RTN_ORDER_INSERT, req_id, struct_to_bytes(&f)),
        );
    }

    // ---- AUTH / LOGIN / LOGOUT / SETTLE ----

    /// CTPBuddy-level handshake: the shim identifies itself and its broker.
    /// Payload JSON: {"broker_id","user_id","app_id","auth_code"}.
    fn on_auth(&mut self, conn_id: u64, frame: &Frame) {
        let v = match json::parse(&String::from_utf8_lossy(&frame.payload)) {
            Ok(v) => v,
            Err(e) => {
                return self.send_auth_error(conn_id, frame.req_id, &format!("AUTH 负载解析失败: {e}"))
            }
        };
        let broker = v.get_str("broker_id").unwrap_or_default();
        let user = v.get_str("user_id").unwrap_or_default();
        let app = v.get_str("app_id").unwrap_or_else(|| "ctpbuddy-client".into());
        if broker != self.cfg.broker_id {
            return self.send_auth_error(
                conn_id,
                frame.req_id,
                &format!("未知 BrokerID '{broker}'（本核心仅服务 {}）", self.cfg.broker_id),
            );
        }
        if user.is_empty() {
            return self.send_auth_error(conn_id, frame.req_id, "user_id 不能为空");
        }
        let already = self.conns.get(&conn_id).map(|c| c.authenticated).unwrap_or(false);
        if already {
            return self.send_auth_error(conn_id, frame.req_id, "连接已认证");
        }
        {
            let c = self.conns.get_mut(&conn_id).unwrap();
            c.authenticated = true;
            c.broker_id = Some(broker.clone());
            c.user_id = {
                let mut u = [0u8; 16];
                set_cstr(&mut u, &user);
                u
            };
        }
        self.journal_record_json(
            "session_auth",
            &broker,
            &user,
            json::obj_sorted(vec![
                ("front_id".into(), json::n(conn_id as f64)),
                ("app_id".into(), json::s(&app)),
            ]),
        );
        let payload = json::obj_sorted(vec![
            ("ok".into(), json::b(true)),
            ("broker_id".into(), json::s(&broker)),
            ("user_id".into(), json::s(&user)),
            ("server".into(), json::s(&format!("{SERVER_NAME}/{SERVER_VERSION}"))),
        ])
        .to_json()
        .into_bytes();
        self.send_frame(conn_id, Frame::new(msgs::AUTH_RSP, frame.req_id, payload));
    }

    fn send_auth_error(&mut self, conn_id: u64, req_id: u32, msg: &str) {
        let payload = json::obj_sorted(vec![
            ("ok".into(), json::b(false)),
            ("error".into(), json::s(msg)),
        ])
        .to_json()
        .into_bytes();
        self.send_frame(conn_id, Frame::new(msgs::AUTH_RSP, req_id, payload));
    }

    fn on_login(&mut self, conn_id: u64, frame: &Frame) {
        let authed = self
            .conns
            .get(&conn_id)
            .map(|c| c.authenticated)
            .unwrap_or(false);
        if !authed {
            return self.send_error(conn_id, frame.req_id, -2, "未认证：请先发送 AUTH");
        }
        let req: CThostFtdcReqUserLoginField = match struct_from_bytes(&frame.payload) {
            Some(f) => f,
            None => return self.send_error(conn_id, frame.req_id, -2, "登录字段长度错误"),
        };
        let broker = cstr(&req.BrokerID);
        let user = cstr(&req.UserID);
        if broker != self.cfg.broker_id {
            return self.send_error(
                conn_id,
                frame.req_id,
                63,
                &format!("BrokerID '{broker}' 与本核心服务的不一致（{}）", self.cfg.broker_id),
            );
        }
        if user.is_empty() {
            return self.send_error(conn_id, frame.req_id, 3, "UserID 为空");
        }

        // auto-open the account on first login (SimNow-style)
        self.ledger.ensure_account(&broker, &user);
        let (front_id, session_id) = {
            let c = self.conns.get_mut(&conn_id).unwrap();
            c.logins += 1;
            c.session_id = c.logins as i32;
            c.investor_id = Some(user.clone());
            (c.front_id, c.session_id)
        };
        let max_ref = self
            .orders_today
            .iter()
            .filter(|o| cstr(&o.InvestorID) == user)
            .filter_map(|o| cstr(&o.OrderRef).trim().parse::<i64>().ok())
            .max()
            .unwrap_or(0);

        let day = self.vt_day();
        let tstr = self.now_str();
        let mut f = CThostFtdcRspUserLoginField::zeroed();
        set_cstr(&mut f.TradingDay, &day);
        set_cstr(&mut f.LoginTime, &tstr);
        set_cstr(&mut f.BrokerID, &broker);
        set_cstr(&mut f.UserID, &user);
        set_cstr(&mut f.SystemName, SERVER_NAME);
        set_cstr(&mut f.SysVersion, &format!("{SERVER_NAME}/{SERVER_VERSION}"));
        f.FrontID = front_id;
        f.SessionID = session_id;
        set_cstr(&mut f.MaxOrderRef, &max_ref.to_string());
        for slot in [
            &mut f.SHFETime,
            &mut f.DCETime,
            &mut f.CZCETime,
            &mut f.FFEXTime,
            &mut f.INETime,
            &mut f.GFEXTime,
        ] {
            set_cstr(slot, &tstr);
        }
        self.journal_record_json(
            "session_login",
            &broker,
            &user,
            json::obj_sorted(vec![
                ("front_id".into(), json::n(front_id as f64)),
                ("session_id".into(), json::n(session_id as f64)),
            ]),
        );
        self.send_frame(
            conn_id,
            Frame::new(msgs::RSP_USER_LOGIN, frame.req_id, struct_to_bytes(&f)),
        );
        // NOTE: DESIGN §6.4's login snapshot push (orders/positions/account) is
        // deferred to M2 — clients issue ReqQry* after login as usual.
    }

    fn on_logout(&mut self, conn_id: u64, frame: &Frame) {
        let req: CThostFtdcUserLogoutField = match struct_from_bytes(&frame.payload) {
            Some(f) => f,
            None => return self.send_error(conn_id, frame.req_id, -2, "登出字段长度错误"),
        };
        let broker = cstr(&req.BrokerID);
        let user = cstr(&req.UserID);
        if let Some(c) = self.conns.get_mut(&conn_id) {
            c.investor_id = None;
        }
        self.journal_record_json("session_logout", &broker, &user, Value::Null);
        self.send_frame(
            conn_id,
            Frame::new(msgs::RSP_USER_LOGOUT, frame.req_id, struct_to_bytes(&req)),
        );
    }

    fn on_settle_confirm(&mut self, conn_id: u64, frame: &Frame) {
        if self.session(conn_id).is_none() {
            return self.send_error(conn_id, frame.req_id, -3, "用户未登录");
        }
        let req: CThostFtdcSettlementInfoConfirmField = match struct_from_bytes(&frame.payload) {
            Some(f) => f,
            None => return self.send_error(conn_id, frame.req_id, -2, "确认字段长度错误"),
        };
        // M1: no real settlement cycle — accept and echo (DESIGN §8.7 TODO).
        self.send_frame(
            conn_id,
            Frame::new(msgs::RSP_SETTLE_CONFIRM, frame.req_id, struct_to_bytes(&req)),
        );
    }

    // ---- ORDER INSERT ----

    fn on_order_insert(&mut self, conn_id: u64, frame: &Frame) {
        let (broker, investor, user_id, front_id, session_id) = match self.session(conn_id) {
            Some(v) => v,
            None => return self.send_err_rtn(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let input: CThostFtdcInputOrderField = match struct_from_bytes(&frame.payload) {
            Some(f) => f,
            None => return self.send_err_rtn(conn_id, frame.req_id, -2, "报单字段长度错误"),
        };
        if cstr(&input.BrokerID) != broker || cstr(&input.InvestorID) != investor {
            return self.send_err_rtn(
                conn_id,
                frame.req_id,
                3,
                "报单 BrokerID/InvestorID 与登录会话不一致",
            );
        }
        let direction = match Direction::from_ctp(input.Direction) {
            Some(d) => d,
            None => return self.send_err_rtn(conn_id, frame.req_id, 40, "买卖方向非法"),
        };
        let offset = match OffsetFlag::from_ctp(input.CombOffsetFlag[0]) {
            Some(o) => o,
            None => return self.send_err_rtn(conn_id, frame.req_id, 40, "开平标志非法"),
        };
        let instrument = cstr(&input.InstrumentID);
        let exchange = cstr(&input.ExchangeID);
        if instrument.is_empty() {
            return self.send_err_rtn(conn_id, frame.req_id, 40, "InstrumentID 为空");
        }
        if let Some(info) = self.engine.catalog().get(&instrument) {
            if !exchange.is_empty() && info.exchange_id != exchange {
                return self.send_err_rtn(
                    conn_id,
                    frame.req_id,
                    22,
                    &format!("合约 {instrument} 不属于交易所 {exchange}"),
                );
            }
        }
        let (tc, vc) = match normalize_conditions(
            input.OrderPriceType,
            input.TimeCondition,
            input.VolumeCondition,
            input.ContingentCondition,
        ) {
            Ok(v) => v,
            Err((code, msg)) => return self.send_err_rtn(conn_id, frame.req_id, code, &msg),
        };

        let (order_ref, order_local) = {
            let c = self.conns.get_mut(&conn_id).unwrap();
            c.order_local_seq += 1;
            let local = c.order_local_seq;
            let r = if cstr(&input.OrderRef).is_empty() {
                local.to_string()
            } else {
                cstr(&input.OrderRef)
            };
            (r, local.to_string())
        };
        let order_key = format!("{front_id}/{session_id}/{order_ref}");

        let intent = OrderIntent {
            broker_id: {
                let mut b = [0u8; 11];
                set_cstr(&mut b, &broker);
                b
            },
            investor_id: {
                let mut i = [0u8; 13];
                set_cstr(&mut i, &investor);
                i
            },
            user_id,
            order_ref: {
                let mut r = [0u8; 13];
                set_cstr(&mut r, &order_ref);
                r
            },
            order_local_id: {
                let mut l = [0u8; 13];
                set_cstr(&mut l, &order_local);
                l
            },
            instrument_id: instrument.clone(),
            exchange_id: exchange.clone(),
            direction,
            offset,
            hedge_flag: if input.CombHedgeFlag[0] == 0 { b'1' } else { input.CombHedgeFlag[0] },
            price_type: input.OrderPriceType,
            limit_price: input.LimitPrice,
            volume: input.VolumeTotalOriginal,
            time_condition: tc,
            volume_condition: vc,
            min_volume: input.MinVolume,
            contingent_condition: input.ContingentCondition,
            stop_price: input.StopPrice,
            force_close_reason: input.ForceCloseReason,
            request_id: input.RequestID,
            front_id,
            session_id,
        };

        // 1) static validation (contract / price / volume)
        if let Err((code, msg)) = self.engine.check(&intent) {
            self.journal_rejected_order(&broker, &investor, &intent, code, &msg);
            return self.send_err_rtn(conn_id, frame.req_id, code, &msg);
        }
        // 2) funds / position reservation at the estimate price
        let price_est = if input.OrderPriceType == b'2' {
            input.LimitPrice
        } else {
            self.engine.last_price(&instrument).unwrap_or(input.LimitPrice)
        };
        if offset.is_close() {
            // a sell closes a long position (and vice versa)
            let side = PositionSide::of(direction).opposite();
            if let Err(code) = self.ledger.freeze_close_position(
                &order_key,
                &broker,
                &investor,
                &instrument,
                side,
                offset,
                input.VolumeTotalOriginal,
            ) {
                let msg = close_reject_msg(code);
                self.journal_rejected_order(&broker, &investor, &intent, code, &msg);
                return self.send_err_rtn(conn_id, frame.req_id, code, &msg);
            }
        }
        let (est_margin, est_comm) = if offset == OffsetFlag::Open {
            let info = self.engine.catalog().get(&instrument);
            let m = info
                .map(|i| i.margin(direction, price_est, input.VolumeTotalOriginal))
                .unwrap_or(0.0);
            let c = info
                .map(|i| i.commission(price_est, input.VolumeTotalOriginal))
                .unwrap_or(input.VolumeTotalOriginal as f64);
            (m, c)
        } else {
            let c = self
                .engine
                .catalog()
                .get(&instrument)
                .map(|i| i.commission(price_est, input.VolumeTotalOriginal))
                .unwrap_or(input.VolumeTotalOriginal as f64);
            (0.0, c)
        };
        if let Err(code) = self
            .ledger
            .freeze(&order_key, &broker, &investor, est_margin, est_comm)
        {
            // roll back the position reservation made above (close path)
            self.ledger.unfreeze_order(&order_key);
            let msg = if code == 50 { "可用资金不足" } else { "报单被拒绝" };
            self.journal_rejected_order(&broker, &investor, &intent, code, msg);
            return self.send_err_rtn(conn_id, frame.req_id, code, msg);
        }

        // 3) engine accept + immediate fill / rest / cancel
        let (day, now) = self.clock_owned();
        let ctx = ClockCtx {
            trading_day: &day,
            now_ms: now,
        };
        let outcome = self.engine.submit(&intent, &ctx);
        match outcome {
            SubmitOutcome::Rejected { error_id, msg } => {
                self.ledger.unfreeze_order(&order_key);
                self.journal_rejected_order(&broker, &investor, &intent, error_id, &msg);
                self.send_err_rtn(conn_id, frame.req_id, error_id, &msg);
            }
            SubmitOutcome::Accepted { events } => {
                // OnRspOrderInsert (success) fires before the Rtn callbacks
                self.send_frame(
                    conn_id,
                    Frame::new(msgs::RSP_ORDER_INSERT, frame.req_id, Vec::new()),
                );
                let mut fills: Vec<Value> = Vec::new();
                let mut sys_id = String::new();
                for ev in events {
                    match &ev {
                        ctpbuddy_matching::EngineEvent::Order(o) => {
                            if sys_id.is_empty() {
                                sys_id = cstr(&o.OrderSysID);
                            }
                        }
                        ctpbuddy_matching::EngineEvent::Trade { fill, .. } => {
                            // only the submitter's own fills: a book match
                            // also emits the resting counterparty's fill, which
                            // dispatch_event journals under its own account
                            if fill.order_key == order_key {
                                fills.push(json::obj_sorted(vec![
                                    ("trade_id".into(), json::s(&cstr(&fill.trade_id))),
                                    ("price".into(), json::n(fill.price)),
                                    ("volume".into(), json::n(fill.volume as f64)),
                                ]));
                            }
                        }
                    }
                    self.dispatch_event(ev);
                }
                self.journal_record_json(
                    "order_insert",
                    &broker,
                    &investor,
                    json::obj_sorted(vec![
                        ("order_ref".into(), json::s(&order_ref)),
                        ("order_sys_id".into(), json::s(&sys_id)),
                        ("instrument".into(), json::s(&instrument)),
                        ("exchange".into(), json::s(&exchange)),
                        ("direction".into(), json::n(direction.as_ctp() as f64)),
                        ("offset".into(), json::n(offset.as_ctp() as f64)),
                        ("price_type".into(), json::s(&(input.OrderPriceType as char).to_string())),
                        ("limit_price".into(), json::n(input.LimitPrice)),
                        ("volume".into(), json::n(input.VolumeTotalOriginal as f64)),
                        // TC/VC/MinVolume complete the request so a journal
                        // replay can reconstruct FAK/FOK exactly (DESIGN §11.4)
                        ("time_condition".into(), json::s(&(tc as char).to_string())),
                        ("volume_condition".into(), json::s(&(vc as char).to_string())),
                        ("min_volume".into(), json::n(input.MinVolume as f64)),
                        (
                            "outcome".into(),
                            json::obj_sorted(vec![
                                ("accepted".into(), json::b(true)),
                                ("fills".into(), Value::Arr(fills)),
                            ]),
                        ),
                    ]),
                );
            }
        }
    }

    fn journal_rejected_order(
        &mut self,
        broker: &str,
        investor: &str,
        intent: &OrderIntent,
        error_id: i32,
        msg: &str,
    ) {
        self.journal_record_json(
            "order_insert",
            broker,
            investor,
            json::obj_sorted(vec![
                ("order_ref".into(), json::s(&intent.order_ref_s())),
                ("instrument".into(), json::s(&intent.instrument_id)),
                ("exchange".into(), json::s(&intent.exchange_id)),
                ("direction".into(), json::n(intent.direction.as_ctp() as f64)),
                ("offset".into(), json::n(intent.offset.as_ctp() as f64)),
                ("price_type".into(), json::s(&(intent.price_type as char).to_string())),
                ("limit_price".into(), json::n(intent.limit_price)),
                ("volume".into(), json::n(intent.volume as f64)),
                ("time_condition".into(), json::s(&(intent.time_condition as char).to_string())),
                ("volume_condition".into(), json::s(&(intent.volume_condition as char).to_string())),
                ("min_volume".into(), json::n(intent.min_volume as f64)),
                (
                    "outcome".into(),
                    json::obj_sorted(vec![
                        ("accepted".into(), json::b(false)),
                        ("error_id".into(), json::n(error_id as f64)),
                        ("msg".into(), json::s(msg)),
                    ]),
                ),
            ]),
        );
    }

    // ---- ORDER ACTION (cancel) ----

    fn on_order_action(&mut self, conn_id: u64, frame: &Frame) {
        let (broker, investor, _user, _front, _sess) = match self.session(conn_id) {
            Some(v) => v,
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let action: CThostFtdcInputOrderActionField = match struct_from_bytes(&frame.payload) {
            Some(f) => f,
            None => return self.send_error(conn_id, frame.req_id, -2, "撤单字段长度错误"),
        };
        let q = CancelQuery {
            front_id: action.FrontID,
            session_id: action.SessionID,
            order_ref: cstr(&action.OrderRef),
            order_sys_id: cstr(&action.OrderSysID),
            investor_id: investor.clone(),
        };
        let (day, now) = self.clock_owned();
        let ctx = ClockCtx {
            trading_day: &day,
            now_ms: now,
        };
        match self.engine.cancel(&q, &ctx) {
            Ok(events) => {
                // OnRspOrderAction (success) fires before the Rtn callbacks
                self.send_frame(
                    conn_id,
                    Frame::new(msgs::RSP_ORDER_ACTION, frame.req_id, Vec::new()),
                );
                // §8.9: 前态 + 新态('5'). dispatch_event fans out, journals and
                // releases the order's remaining freeze on the terminal '5'.
                let mut final_field = None;
                for ev in events {
                    if let ctpbuddy_matching::EngineEvent::Order(ref f) = ev {
                        final_field = Some(f.clone());
                    }
                    self.dispatch_event(ev);
                }
                if let Some(field) = final_field {
                    self.journal_record_json(
                        "order_cancel",
                        &broker,
                        &investor,
                        json::obj_sorted(vec![
                            ("order_ref".into(), json::s(&cstr(&field.OrderRef))),
                            ("order_sys_id".into(), json::s(&cstr(&field.OrderSysID))),
                            ("instrument".into(), json::s(&cstr(&field.InstrumentID))),
                            ("cancel_time".into(), json::s(&cstr(&field.CancelTime))),
                        ]),
                    );
                }
            }
            Err((code, msg)) => self.send_error(conn_id, frame.req_id, code, &msg),
        }
    }

    // ---- MD SUBSCRIBE / UNSUBSCRIBE ----

    fn on_sub_md(&mut self, conn_id: u64, frame: &Frame, subscribe: bool) {
        let sz = size_of::<CThostFtdcSpecificInstrumentField>();
        if frame.payload.is_empty() || frame.payload.len() % sz != 0 {
            return self.send_error(conn_id, frame.req_id, -2, "订阅字段长度错误");
        }
        let n = frame.payload.len() / sz;
        for i in 0..n {
            let bytes = &frame.payload[i * sz..(i + 1) * sz];
            let f: CThostFtdcSpecificInstrumentField = match struct_from_bytes(bytes) {
                Some(f) => f,
                None => continue,
            };
            let instrument = cstr(&f.InstrumentID);
            {
                let c = self.conns.get_mut(&conn_id).unwrap();
                if subscribe {
                    c.md_subs.insert(instrument.clone());
                } else {
                    c.md_subs.remove(&instrument);
                }
            }
            let rsp = if subscribe { msgs::RSP_SUB_MD } else { msgs::RSP_UNSUB_MD };
            self.send_frame(conn_id, Frame::new(rsp, frame.req_id, struct_to_bytes(&f)));
        }
    }

    // ---- QUERIES ----

    /// CTP 查询流控 (docs: 报单流控、查询流控和会话数控制).
    ///
    /// Real fronts carry a per-session `QryFreq` budget (surfaced by
    /// `GetFrontInfo` after login) and answer an over-budget `ReqQry*` with
    /// `OnRspError[90]` "CTP：查询未就绪，请稍后重试" -- the query never
    /// runs, the client waits and re-issues. Investor sessions are
    /// throttled; operators are not, and this core only serves investors.
    ///
    /// This is the server half of the flow-control pair; the shim enforces
    /// the in-flight half (one outstanding query, second gets rc -2).
    /// Returns true when the query may proceed.
    fn qry_gate(&mut self, conn_id: u64, req_id: u32) -> bool {
        let now = Instant::now();
        let quota = self.cfg.qry_freq.max(1);
        let used = {
            let c = match self.conns.get_mut(&conn_id) {
                Some(c) => c,
                None => return false,
            };
            match c.qry_window {
                // still inside the current one-second window
                Some(w) if now.duration_since(w) < Duration::from_secs(1) => {
                    c.qry_count += 1;
                    c.qry_count
                }
                // window expired (or first query): start a fresh one
                _ => {
                    c.qry_window = Some(now);
                    c.qry_count = 1;
                    1
                }
            }
        };
        if used <= quota {
            return true;
        }
        self.send_error(conn_id, req_id, 90, "CTP：查询未就绪，请稍后重试");
        false
    }

    /// Mark positions to market before answering PnL-bearing queries. The
    /// 10ms pulse is a cadence, not a guarantee: a fill immediately followed
    /// by a query must not observe the pre-fill unrealized PnL -- CTP's
    /// `Balance` is dynamic equity, so a stale mark leaks straight into it.
    fn mark_to_market_now(&mut self) {
        let prices = self.engine.last_prices();
        self.ledger.mark_to_market(self.engine.catalog(), &prices);
    }

    fn on_qry_instrument(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        // lenient: a malformed payload degrades to "no filter"
        let q: CThostFtdcQryInstrumentField = struct_from_bytes(&frame.payload)
            .unwrap_or_else(CThostFtdcQryInstrumentField::zeroed);
        let (ex, ins, prod) = (cstr(&q.ExchangeID), cstr(&q.InstrumentID), cstr(&q.ProductID));
        let fields: Vec<_> = self
            .engine
            .catalog()
            .iter()
            .filter(|i| ex.is_empty() || i.exchange_id == ex)
            .filter(|i| ins.is_empty() || i.instrument_id == ins)
            .filter(|i| prod.is_empty() || i.product_id == prod)
            .map(|i| i.to_field())
            .collect();
        for f in &fields {
            self.send_frame(
                conn_id,
                Frame::new(msgs::RSP_QRY_INSTRUMENT, frame.req_id, struct_to_bytes(f)),
            );
        }
        self.send_frame(conn_id, Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()));
    }

    fn on_qry_trading_account(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        self.mark_to_market_now();
        let sess = self.session(conn_id);
        let (broker, investor) = match sess {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: ctpbuddy_wire::generated::CThostFtdcQryTradingAccountField =
            struct_from_bytes(&frame.payload).unwrap_or_else(|| {
                ctpbuddy_wire::generated::CThostFtdcQryTradingAccountField::zeroed()
            });
        let target = {
            let t = cstr(&q.InvestorID);
            if t.is_empty() {
                investor.clone()
            } else {
                t
            }
        };
        let day = self.vt_day();
        if let Some(a) = self.ledger.account(&broker, &target) {
            let f = a.to_field(&day);
            self.send_frame(
                conn_id,
                Frame::new(msgs::RSP_QRY_TRADING_ACCOUNT, frame.req_id, struct_to_bytes(&f)),
            );
        }
        self.send_frame(conn_id, Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()));
    }

    fn on_qry_investor_position(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        self.mark_to_market_now();
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQryInvestorPositionField = struct_from_bytes(&frame.payload)
            .unwrap_or_else(CThostFtdcQryInvestorPositionField::zeroed);
        let filter = cstr(&q.InstrumentID);
        let day = self.vt_day();
        let positions: Vec<_> = self
            .ledger
            .positions_of(&broker, &investor)
            .into_iter()
            .filter(|p| filter.is_empty() || p.instrument_id == filter)
            .map(|p| {
                let ex = self
                    .engine
                    .catalog()
                    .get(&p.instrument_id)
                    .map(|i| i.exchange_id.clone())
                    .unwrap_or_default();
                let mult = self
                    .engine
                    .catalog()
                    .get(&p.instrument_id)
                    .map(|i| i.volume_multiple)
                    .unwrap_or(1);
                p.to_field(&broker, &investor, &ex, &day, mult)
            })
            .collect();
        for f in &positions {
            self.send_frame(
                conn_id,
                Frame::new(msgs::RSP_QRY_INVESTOR_POSITION, frame.req_id, struct_to_bytes(f)),
            );
        }
        self.send_frame(conn_id, Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()));
    }

    fn on_qry_order(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQryOrderField =
            struct_from_bytes(&frame.payload).unwrap_or_else(CThostFtdcQryOrderField::zeroed);
        let (ins, sys) = (cstr(&q.InstrumentID), cstr(&q.OrderSysID));
        // orders_today holds every state transition; the query surface mirrors
        // CTP semantics: one row per order, its latest state, in insert order.
        let mut latest: HashMap<String, usize> = HashMap::new();
        let mut keys: Vec<String> = Vec::new();
        for (i, o) in self.orders_today.iter().enumerate() {
            if cstr(&o.BrokerID) != broker || cstr(&o.InvestorID) != investor {
                continue;
            }
            if !ins.is_empty() && cstr(&o.InstrumentID) != ins {
                continue;
            }
            if !sys.is_empty() && cstr(&o.OrderSysID) != sys {
                continue;
            }
            let key = format!("{}/{}", cstr(&o.OrderSysID), cstr(&o.OrderRef));
            if !latest.contains_key(&key) {
                keys.push(key.clone());
            }
            latest.insert(key, i);
        }
        for key in keys {
            if let Some(&i) = latest.get(&key) {
                let f = &self.orders_today[i];
                self.send_frame(
                    conn_id,
                    Frame::new(msgs::RSP_QRY_ORDER, frame.req_id, struct_to_bytes(f)),
                );
            }
        }
        self.send_frame(conn_id, Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()));
    }

    fn on_qry_trade(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQryTradeField =
            struct_from_bytes(&frame.payload).unwrap_or_else(CThostFtdcQryTradeField::zeroed);
        let (ins, tid) = (cstr(&q.InstrumentID), cstr(&q.TradeID));
        let fields: Vec<_> = self
            .trades_today
            .iter()
            .filter(|t| cstr(&t.BrokerID) == broker && cstr(&t.InvestorID) == investor)
            .filter(|t| ins.is_empty() || cstr(&t.InstrumentID) == ins)
            .filter(|t| tid.is_empty() || cstr(&t.TradeID) == tid)
            .cloned()
            .collect();
        for f in &fields {
            self.send_frame(
                conn_id,
                Frame::new(msgs::RSP_QRY_TRADE, frame.req_id, struct_to_bytes(f)),
            );
        }
        self.send_frame(conn_id, Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()));
    }
}

/// Normalize CTP order conditions to the engine matrix (DESIGN §8.2/§8.4),
/// using the **official** encodings from `ThostFtdcUserApiDataType.h`:
/// TC_IOC='1', TC_GFS='2', TC_GFD='3', TC_GTD='4', TC_GTC='5';
/// VC_AV='1', VC_MV='2', VC_CV='3'.
///
/// FAK/FOK are TC+VC combinations, not separate fields (docs/notes/01 A3):
/// FAK = IOC+AV or IOC+MV(MinVolume), FOK = IOC+CV.
fn normalize_conditions(
    price_type: u8,
    time_condition: u8,
    volume_condition: u8,
    contingent_condition: u8,
) -> Result<(u8, u8), (i32, String)> {
    let vc = match volume_condition {
        b'1' => b'1', // any volume
        b'2' => b'2', // minimum volume (FAK 指定成交数量)
        b'3' => b'3', // all volume (FOK)
        _ => return Err((41, format!("不支持的 VolumeCondition '{}'", volume_condition as char))),
    };
    if contingent_condition != b'1' {
        return Err((
            41,
            "条件单暂不支持（ContingentCondition != 立即）".to_string(),
        ));
    }
    let tc = match time_condition {
        b'1' => b'1', // IOC
        b'2' => b'3', // GFS → GFD (section 概念不建模，退化为当日有效)
        b'3' => b'3', // GFD
        b'4' => return Err((41, "GTD（指定有效期）暂不支持".to_string())),
        b'5' => return Err((41, "GTC（撤销前有效）暂不支持".to_string())),
        _ => return Err((41, format!("不支持的 TimeCondition '{}'", time_condition as char))),
    };
    // AnyPrice market orders are immediate-or-cancel by definition.
    let tc = if price_type == b'1' { b'1' } else { tc };
    Ok((tc, vc))
}

fn close_reject_msg(code: i32) -> &'static str {
    match code {
        31 => "可平今仓不足",
        _ => "持仓不足",
    }
}
