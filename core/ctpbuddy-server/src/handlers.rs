//! CTP message handlers: frame -> SPI-equivalent callbacks.
//!
//! Layout: this file dispatches frames and handles session lifecycle (AUTH,
//! login/logout, settlement confirm); order insert/cancel live in order,
//! market-data subscription and every ReqQry* in query.
//!
//! Wire conventions implemented here (documented in `ctpbuddy-wire::msgs`):
//! - success `RSP_*` = response struct payload, `RspInfo` implied zero;
//! - failed request = `RSP_ERROR` (RspInfoField), completing the pending
//!   request as `OnRsp*(NULL, rsp, is_last=true)`;
//! - order-insert failures = `ERR_RTN_ORDER_INSERT` (OnErrRtnOrderInsert);
//! - query streams terminate with `QRY_LAST` (CTP `bIsLast`).
//!
//! Rejection push surfaces (#42, DESIGN §8.12 / notes/09): real CTP splits an
//! order rejection between two callbacks by layer — the 报盘机 half answers the
//! pending request itself (`OnRspOrderInsert`/`OnRspOrderAction`, pInputOrder
//! NULL, via RSP_ERROR), the exchange half arrives *after* a successful front
//! response as `OnErrRtnOrderInsert`/`OnErrRtnOrderAction` with the client's
//! own input struct. Cancel rejections push both halves (官方报单回调规则
//! 场景 6/7: 先响应后回报). The `ERR_RTN_*` payload therefore carries the
//! client's input struct ++ RspInfoField, because the success response has
//! already consumed the shim's pending entry by then.

use std::collections::HashMap;
use std::mem::size_of;
use std::time::{Duration, Instant};

use ctpbuddy_ledger::{PositionSide, ERR_FUNDS, ERR_NO_CLOSE_TODAY_LEDGER, ERR_NO_CLOSE_YD_LEDGER};
use ctpbuddy_matching::{
    CancelQuery, ClockCtx, Direction, OffsetFlag, OrderIntent, SubmitOutcome, ERR_BAD_FIELD,
    ERR_DUPLICATE_ORDER, ERR_EXCHANGE_ID_INVALID, ERR_INSTRUMENT_NOT_FOUND,
    ERR_INSTRUMENT_NOT_TRADING, ERR_ORDER_FREQ, HEDGE_FLAG_SPECULATION,
};
use ctpbuddy_wire::generated::{
    cstr, set_cstr, CThostFtdcInputOrderActionField, CThostFtdcInputOrderField,
    CThostFtdcQryBrokerTradingParamsField, CThostFtdcQryInstrumentCommissionRateField,
    CThostFtdcQryInstrumentField, CThostFtdcQryInstrumentMarginRateField,
    CThostFtdcQryInstrumentOrderCommRateField, CThostFtdcQryInvestorPositionDetailField,
    CThostFtdcQryInvestorPositionField, CThostFtdcQryInvestorProductGroupMarginField,
    CThostFtdcQryOrderField, CThostFtdcQrySettlementInfoField, CThostFtdcQryTradeField,
    CThostFtdcReqUserLoginField, CThostFtdcRspUserLoginField, CThostFtdcSettlementInfoConfirmField,
    CThostFtdcSettlementInfoField, CThostFtdcSpecificInstrumentField, CThostFtdcUserLogoutField,
};
use ctpbuddy_wire::msgs;
use ctpbuddy_wire::{struct_from_bytes, struct_to_bytes};

use crate::json::{self, Value};
use crate::{Frame, GateStream, World, SERVER_NAME, SERVER_VERSION};

mod order;
mod query;

/// error.xml: `UNSUPPORTED_FUNCTION` — 不支持的 TimeCondition/VolumeCondition/条件单。
const ERR_UNSUPPORTED_FUNCTION: i32 = 27;
const ERR_UNSUPPORTED_FUNCTION_MSG: &str = "CTP:不支持的功能";

/// Env var: when set (non-empty), the TD handshake's `auth_code` must equal it.
pub const TD_TOKEN_ENV: &str = "CTPBUDDY_TD_TOKEN";

/// Unset/empty `expected` disables the check (backward compatible).
pub(crate) fn auth_code_ok(expected: Option<&str>, presented: &str) -> bool {
    match expected {
        None | Some("") => true,
        Some(e) => {
            let (a, b) = (e.as_bytes(), presented.as_bytes());
            a.len() == b.len() && a.iter().zip(b).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
        }
    }
}

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
            msgs::PING => self.send_frame(
                conn_id,
                Frame::new(msgs::PONG, frame.req_id, frame.payload.clone()),
            ),
            msgs::AUTH => self.on_auth(conn_id, &frame),
            msgs::REQ_USER_LOGIN => self.on_login(conn_id, &frame),
            msgs::REQ_USER_LOGOUT => self.on_logout(conn_id, &frame),
            msgs::REQ_SETTLE_CONFIRM => self.on_settle_confirm(conn_id, &frame),
            msgs::REQ_QRY_SETTLEMENT_INFO => self.on_qry_settlement_info(conn_id, &frame),
            msgs::REQ_ORDER_INSERT => self.on_order_insert(conn_id, &frame),
            msgs::REQ_ORDER_ACTION => self.on_order_action(conn_id, &frame),
            msgs::SUB_MD => self.on_sub_md(conn_id, &frame, true),
            msgs::UNSUB_MD => self.on_sub_md(conn_id, &frame, false),
            msgs::REQ_QRY_INSTRUMENT => self.on_qry_instrument(conn_id, &frame),
            msgs::REQ_QRY_TRADING_ACCOUNT => self.on_qry_trading_account(conn_id, &frame),
            msgs::REQ_QRY_INVESTOR_POSITION => self.on_qry_investor_position(conn_id, &frame),
            msgs::REQ_QRY_ORDER => self.on_qry_order(conn_id, &frame),
            msgs::REQ_QRY_TRADE => self.on_qry_trade(conn_id, &frame),
            msgs::REQ_QRY_INSTRUMENT_MARGIN_RATE => {
                self.on_qry_instrument_margin_rate(conn_id, &frame)
            }
            msgs::REQ_QRY_INSTRUMENT_COMMISSION_RATE => {
                self.on_qry_instrument_commission_rate(conn_id, &frame)
            }
            msgs::REQ_QRY_INSTRUMENT_ORDER_COMM_RATE => {
                self.on_qry_instrument_order_comm_rate(conn_id, &frame)
            }
            msgs::REQ_QRY_BROKER_TRADING_PARAMS => {
                self.on_qry_broker_trading_params(conn_id, &frame)
            }
            msgs::REQ_QRY_INVESTOR_PRODUCT_GROUP_MARGIN => {
                self.on_qry_investor_product_group_margin(conn_id, &frame)
            }
            msgs::REQ_QRY_INVESTOR_POSITION_DETAIL => {
                self.on_qry_investor_position_detail(conn_id, &frame)
            }
            other => {
                eprintln!("[ctpbuddy] conn {conn_id}: unsupported msg 0x{other:04x}");
                self.send_error(conn_id, frame.req_id, -1, "不支持的消息类型")
            }
        }
    }

    // ---- session helpers ----

    pub(crate) fn session(&self, conn_id: u64) -> Option<(String, String, [u8; 16], i32, i32)> {
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

    /// 错单回报半面 (OnErrRtnOrderInsert): the exchange-side rejection that
    /// lands *after* the order was accepted at the front office. Payload =
    /// the client's own `CThostFtdcInputOrderField` ++ `CThostFtdcRspInfoField`
    /// — the success response has already consumed the shim's pending entry,
    /// so the input rides along (real CTP passes the client's input back too).
    pub(crate) fn send_err_rtn(
        &mut self,
        conn_id: u64,
        req_id: u32,
        error_id: i32,
        msg: &str,
        input: &[u8],
    ) {
        self.send_err_rtn_generic(
            conn_id,
            req_id,
            msgs::ERR_RTN_ORDER_INSERT,
            error_id,
            msg,
            input,
        );
    }

    /// 错单回报半面 (OnErrRtnOrderAction): cancel rejection, always paired
    /// with the `RSP_ERROR` response half (官方报单回调规则 场景 6/7:
    /// 先响应后回报). Payload = `CThostFtdcInputOrderActionField` ++
    /// `CThostFtdcRspInfoField`; the shim synthesizes the
    /// `CThostFtdcOrderActionField` it passes to the callback.
    pub(crate) fn send_err_rtn_action(
        &mut self,
        conn_id: u64,
        req_id: u32,
        error_id: i32,
        msg: &str,
        input: &[u8],
    ) {
        self.send_err_rtn_generic(
            conn_id,
            req_id,
            msgs::ERR_RTN_ORDER_ACTION,
            error_id,
            msg,
            input,
        );
    }

    pub(crate) fn send_err_rtn_generic(
        &mut self,
        conn_id: u64,
        req_id: u32,
        msg_type: u16,
        error_id: i32,
        msg: &str,
        input: &[u8],
    ) {
        let mut f = ctpbuddy_wire::generated::CThostFtdcRspInfoField::zeroed();
        f.ErrorID = error_id;
        set_cstr(&mut f.ErrorMsg, msg);
        // wire order: the client's input struct first, RspInfo last -- both
        // the shim rows and the py SDK decode it that way.
        let mut payload = input.to_vec();
        payload.extend_from_slice(&struct_to_bytes(&f));
        self.send_frame(conn_id, Frame::new(msg_type, req_id, payload));
    }

    // ---- AUTH / LOGIN / LOGOUT / SETTLE ----

    /// CTPBuddy-level handshake: the shim identifies itself and its broker.
    /// If `CTPBUDDY_TD_TOKEN` is set, `auth_code` must equal it.
    /// Payload JSON: {"broker_id","user_id","app_id","auth_code"}.
    pub(crate) fn on_auth(&mut self, conn_id: u64, frame: &Frame) {
        let v = match json::parse(&String::from_utf8_lossy(&frame.payload)) {
            Ok(v) => v,
            Err(e) => {
                return self.send_auth_error(
                    conn_id,
                    frame.req_id,
                    &format!("AUTH 负载解析失败: {e}"),
                )
            }
        };
        let broker = v.get_str("broker_id").unwrap_or_default();
        let user = v.get_str("user_id").unwrap_or_default();
        let app = v
            .get_str("app_id")
            .unwrap_or_else(|| "ctpbuddy-client".into());
        if broker != self.cfg.broker_id {
            return self.send_auth_error(
                conn_id,
                frame.req_id,
                &format!(
                    "未知 BrokerID '{broker}'（本核心仅服务 {}）",
                    self.cfg.broker_id
                ),
            );
        }
        if user.is_empty() {
            return self.send_auth_error(conn_id, frame.req_id, "user_id 不能为空");
        }
        let presented = v.get_str("auth_code").unwrap_or_default();
        if !auth_code_ok(std::env::var(TD_TOKEN_ENV).ok().as_deref(), &presented) {
            eprintln!("[ctpbuddy] AUTH rejected for user '{user}': bad auth_code");
            return self.send_auth_error(conn_id, frame.req_id, "auth_code 校验失败");
        }
        let already = self
            .conns
            .get(&conn_id)
            .map(|c| c.authenticated)
            .unwrap_or(false);
        if already {
            return self.send_auth_error(conn_id, frame.req_id, "连接已认证");
        }
        {
            let c = self.conns.get_mut(&conn_id).unwrap();
            if let Some(r) = v.get_num("private_resume") {
                if r == 0.0 || r == 1.0 || r == 2.0 {
                    c.private_resume = r as u8;
                }
            }
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
            (
                "server".into(),
                json::s(&format!("{SERVER_NAME}/{SERVER_VERSION}")),
            ),
        ])
        .to_json()
        .into_bytes();
        self.send_frame(conn_id, Frame::new(msgs::AUTH_RSP, frame.req_id, payload));
    }

    pub(crate) fn send_auth_error(&mut self, conn_id: u64, req_id: u32, msg: &str) {
        let payload = json::obj_sorted(vec![
            ("ok".into(), json::b(false)),
            ("error".into(), json::s(msg)),
        ])
        .to_json()
        .into_bytes();
        self.send_frame(conn_id, Frame::new(msgs::AUTH_RSP, req_id, payload));
    }

    pub(crate) fn on_login(&mut self, conn_id: u64, frame: &Frame) {
        let authed = self
            .conns
            .get(&conn_id)
            .map(|c| c.authenticated)
            .unwrap_or(false);
        if !authed {
            return self.send_error(conn_id, frame.req_id, 64, "未认证：请先发送 AUTH");
        }
        let req: CThostFtdcReqUserLoginField = match struct_from_bytes(&frame.payload) {
            Some(f) => f,
            None => {
                return self.send_error(conn_id, frame.req_id, ERR_BAD_FIELD, "登录字段长度错误")
            }
        };
        let broker = cstr(&req.BrokerID);
        let user = cstr(&req.UserID);
        // 错误码全集.md: 63 AUTH_FAILED ← BrokerID 与核心不一致；3 INVALID_LOGIN ← UserID 为空。
        if broker != self.cfg.broker_id {
            return self.send_error(
                conn_id,
                frame.req_id,
                63,
                &format!(
                    "CTP:客户端认证失败：BrokerID '{broker}' 与本核心服务的不一致（{}）",
                    self.cfg.broker_id
                ),
            );
        }
        if user.is_empty() {
            return self.send_error(conn_id, frame.req_id, 3, "CTP:不合法的登录：UserID 为空");
        }
        let (auth_broker, auth_user) = {
            let c = self.conns.get(&conn_id).unwrap();
            (c.broker_id.clone().unwrap_or_default(), cstr(&c.user_id))
        };
        if broker != auth_broker || user != auth_user {
            return self.send_error(
                conn_id,
                frame.req_id,
                15,
                "登录 BrokerID/UserID 与连接认证身份不一致",
            );
        }

        let online = self
            .investor_conns(&broker, &user)
            .into_iter()
            .filter(|id| *id != conn_id)
            .count();
        if self.cfg.max_user_sessions > 0 && online >= self.cfg.max_user_sessions as usize {
            return self.send_error(conn_id, frame.req_id, 60, "CTP:用户在线会话超出上限");
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
        set_cstr(
            &mut f.SysVersion,
            &format!("{SERVER_NAME}/{SERVER_VERSION}"),
        );
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
        // SubscribePrivateTopic replay (RESTART / RESUME); QUICK is a no-op.
        self.replay_private_flow(conn_id, &broker, &user);
    }

    pub(crate) fn on_logout(&mut self, conn_id: u64, frame: &Frame) {
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

    pub(crate) fn on_qry_settlement_info(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQrySettlementInfoField = struct_from_bytes(&frame.payload)
            .unwrap_or_else(CThostFtdcQrySettlementInfoField::zeroed);
        if (!cstr(&q.BrokerID).is_empty() && cstr(&q.BrokerID) != broker)
            || (!cstr(&q.InvestorID).is_empty() && cstr(&q.InvestorID) != investor)
        {
            return self.send_qry_empty(conn_id, frame.req_id);
        }
        let requested = cstr(&q.TradingDay);
        let day = if requested.is_empty() {
            self.settlement_reports
                .iter()
                .filter(|r| {
                    r.broker == broker
                        && r.investor == investor
                        && r.day.len() == 8
                        && r.day < self.vt_trading_day
                })
                .map(|r| r.day.clone())
                .max()
                .unwrap_or_default()
        } else {
            requested
        };
        let account = cstr(&q.AccountID);
        let currency = cstr(&q.CurrencyID);
        let mut rows = Vec::new();
        for report in self.settlement_reports.iter().filter(|r| {
            r.broker == broker
                && r.investor == investor
                && !day.is_empty()
                && r.day == day
                && (account.is_empty() || r.account == account)
                && (currency.is_empty() || r.currency == currency)
        }) {
            for (seq, part) in report.content.chunks(500).enumerate() {
                let mut f = CThostFtdcSettlementInfoField::zeroed();
                set_cstr(&mut f.TradingDay, &report.day);
                f.SettlementID = report.settlement_id;
                set_cstr(&mut f.BrokerID, &report.broker);
                set_cstr(&mut f.InvestorID, &report.investor);
                f.SequenceNo = (seq + 1) as i32;
                f.Content[..part.len()].copy_from_slice(part);
                set_cstr(&mut f.AccountID, &report.account);
                set_cstr(&mut f.CurrencyID, &report.currency);
                rows.push(f);
            }
        }
        for f in rows {
            self.send_frame(
                conn_id,
                Frame::new(
                    msgs::RSP_QRY_SETTLEMENT_INFO,
                    frame.req_id,
                    struct_to_bytes(&f),
                ),
            );
        }
        self.send_qry_empty(conn_id, frame.req_id);
    }

    pub(crate) fn on_settle_confirm(&mut self, conn_id: u64, frame: &Frame) {
        if self.session(conn_id).is_none() {
            return self.send_error(conn_id, frame.req_id, -3, "用户未登录");
        }
        let req: CThostFtdcSettlementInfoConfirmField = match struct_from_bytes(&frame.payload) {
            Some(f) => f,
            None => return self.send_error(conn_id, frame.req_id, -2, "确认字段长度错误"),
        };
        let (broker, user, _, _, _) = self.session(conn_id).unwrap();
        if cstr(&req.BrokerID) != broker || cstr(&req.InvestorID) != user {
            return self.send_error(conn_id, frame.req_id, 3, "结算确认身份与登录会话不一致");
        }
        self.settlement_confirmed
            .insert((broker, user), self.vt_day());
        self.send_frame(
            conn_id,
            Frame::new(
                msgs::RSP_SETTLE_CONFIRM,
                frame.req_id,
                struct_to_bytes(&req),
            ),
        );
    }
}

#[cfg(test)]
mod auth_tests {
    use super::auth_code_ok;

    #[test]
    fn token_check() {
        assert!(auth_code_ok(None, "anything"));
        assert!(auth_code_ok(Some(""), ""));
        assert!(auth_code_ok(Some("s3cret"), "s3cret"));
        assert!(!auth_code_ok(Some("s3cret"), "s3creT"));
        assert!(!auth_code_ok(Some("s3cret"), ""));
        assert!(!auth_code_ok(Some("s3cret"), "s3cret!"));
    }
}
