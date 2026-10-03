//! CTP message handlers: frame -> SPI-equivalent callbacks.
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

use ctpbuddy_ledger::{
    PositionSide, ERR_FUNDS, ERR_NO_CLOSE_TODAY_LEDGER, ERR_NO_CLOSE_YD_LEDGER,
};
use ctpbuddy_matching::{
    CancelQuery, ClockCtx, Direction, OffsetFlag, OrderIntent, SubmitOutcome, ERR_BAD_FIELD,
    ERR_DUPLICATE_ORDER, ERR_EXCHANGE_ID_INVALID, ERR_INSTRUMENT_NOT_FOUND,
    ERR_INSTRUMENT_NOT_TRADING, ERR_ORDER_FREQ, HEDGE_FLAG_SPECULATION,
};
use ctpbuddy_wire::generated::{
    cstr, set_cstr, CThostFtdcInputOrderActionField, CThostFtdcInputOrderField,
    CThostFtdcQryBrokerTradingParamsField, CThostFtdcQryInstrumentCommissionRateField,
    CThostFtdcQryInstrumentField, CThostFtdcQryInstrumentMarginRateField,
    CThostFtdcQryInvestorPositionDetailField,
    CThostFtdcQryInvestorProductGroupMarginField,
    CThostFtdcQryInstrumentOrderCommRateField, CThostFtdcQryInvestorPositionField,
    CThostFtdcQryOrderField, CThostFtdcQryTradeField, CThostFtdcReqUserLoginField,
    CThostFtdcRspUserLoginField, CThostFtdcSettlementInfoConfirmField,
    CThostFtdcSpecificInstrumentField, CThostFtdcUserLogoutField,
};
use ctpbuddy_wire::msgs;
use ctpbuddy_wire::{struct_from_bytes, struct_to_bytes};

use crate::json::{self, Value};
use crate::{Frame, GateStream, SERVER_NAME, SERVER_VERSION, World};

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

    /// 错单回报半面 (OnErrRtnOrderInsert): the exchange-side rejection that
    /// lands *after* the order was accepted at the front office. Payload =
    /// the client's own `CThostFtdcInputOrderField` ++ `CThostFtdcRspInfoField`
    /// — the success response has already consumed the shim's pending entry,
    /// so the input rides along (real CTP passes the client's input back too).
    fn send_err_rtn(&mut self, conn_id: u64, req_id: u32, error_id: i32, msg: &str, input: &[u8]) {
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
    fn send_err_rtn_action(&mut self, conn_id: u64, req_id: u32, error_id: i32, msg: &str, input: &[u8]) {
        self.send_err_rtn_generic(
            conn_id,
            req_id,
            msgs::ERR_RTN_ORDER_ACTION,
            error_id,
            msg,
            input,
        );
    }

    fn send_err_rtn_generic(
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
            return self.send_error(conn_id, frame.req_id, 64, "未认证：请先发送 AUTH");
        }
        let req: CThostFtdcReqUserLoginField = match struct_from_bytes(&frame.payload) {
            Some(f) => f,
            None => return self.send_error(conn_id, frame.req_id, ERR_BAD_FIELD, "登录字段长度错误"),
        };
        let broker = cstr(&req.BrokerID);
        let user = cstr(&req.UserID);
        if broker != self.cfg.broker_id {
            return self.send_error(
                conn_id,
                frame.req_id,
                15,
                &format!("BrokerID '{broker}' 与本核心服务的不一致（{}）", self.cfg.broker_id),
            );
        }
        if user.is_empty() {
            return self.send_error(conn_id, frame.req_id, 15, "UserID 为空");
        }
        let (auth_broker, auth_user) = {
            let c = self.conns.get(&conn_id).unwrap();
            (
                c.broker_id.clone().unwrap_or_default(),
                cstr(&c.user_id),
            )
        };
        if broker != auth_broker || user != auth_user {
            return self.send_error(
                conn_id,
                frame.req_id,
                15,
                "登录 BrokerID/UserID 与连接认证身份不一致",
            );
        }

        let online = self.investor_conns(&broker, &user).into_iter().filter(|id| *id != conn_id).count();
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
        let (broker, user, _, _, _) = self.session(conn_id).unwrap();
        if cstr(&req.BrokerID) != broker || cstr(&req.InvestorID) != user {
            return self.send_error(conn_id, frame.req_id, 3, "结算确认身份与登录会话不一致");
        }
        self.settlement_confirmed.insert((broker, user), self.vt_day());
        self.send_frame(
            conn_id,
            Frame::new(msgs::RSP_SETTLE_CONFIRM, frame.req_id, struct_to_bytes(&req)),
        );
    }

    // ---- ORDER INSERT ----

    /// Rejection surface split for an engine (exchange-side) insert rejection
    /// (notes/01 §B, 知识库 §5.1; #42 / DESIGN §8.12). Real CTP answers
    /// parameter-validation and session-static refusals at the 报盘机 itself:
    /// the pending ReqOrderInsert completes as `OnRspOrderInsert(NULL, pRspInfo)`
    /// and no status return follows. Exchange-regulation refusals (涨跌停板价 /
    /// 交易所数量规范 / 非最小变动价位) arrive *after* a successful front
    /// response — `OnRspOrderInsert(input, {0})` then
    /// `OnErrRtnOrderInsert(input, pRspInfo)`; a client that hooks only
    /// OnRspOrderInsert misses those (notes/01 §5 归纳). `input` is the client's
    /// raw request payload, echoed back on the rtn half.
    fn reject_insert(&mut self, conn_id: u64, req_id: u32, code: i32, msg: &str, input: &[u8]) {
        if matches!(
            code,
            ERR_BAD_FIELD
                | ERR_INSTRUMENT_NOT_FOUND
                | ERR_INSTRUMENT_NOT_TRADING
                | ERR_DUPLICATE_ORDER
        ) {
            self.send_error(conn_id, req_id, code, msg);
        } else {
            // front office accepted first (OnRspOrderInsert {0}); the
            // exchange then refuses as the 错单回报 half.
            self.send_frame(conn_id, Frame::new(msgs::RSP_ORDER_INSERT, req_id, Vec::new()));
            self.send_err_rtn(conn_id, req_id, code, msg, input);
        }
    }

    fn on_order_insert(&mut self, conn_id: u64, frame: &Frame) {
        let (broker, investor, user_id, front_id, session_id) = match self.session(conn_id) {
            Some(v) => v,
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let input: CThostFtdcInputOrderField = match struct_from_bytes(&frame.payload) {
            Some(f) => f,
            None => return self.send_error(conn_id, frame.req_id, -2, "报单字段长度错误"),
        };
        if cstr(&input.BrokerID) != broker || cstr(&input.InvestorID) != investor {
            return self.send_error(
                conn_id,
                frame.req_id,
                3,
                "报单 BrokerID/InvestorID 与登录会话不一致",
            );
        }
        let direction = match Direction::from_ctp(input.Direction) {
            Some(d) => d,
            None => return self.send_error(conn_id, frame.req_id, ERR_BAD_FIELD, "CTP:报单字段有误"),
        };
        let offset = match OffsetFlag::from_ctp(input.CombOffsetFlag[0]) {
            Some(o) => o,
            None => return self.send_error(conn_id, frame.req_id, ERR_BAD_FIELD, "CTP:报单字段有误"),
        };
        let instrument = cstr(&input.InstrumentID);
        let mut exchange = cstr(&input.ExchangeID);
        if instrument.is_empty() {
            return self.send_error(conn_id, frame.req_id, ERR_BAD_FIELD, "CTP:报单字段有误");
        }
        if let Some(info) = self.engine.catalog().get(&instrument) {
            if exchange.is_empty() {
                // backfill from the catalog: the per-exchange rule tables
                // (DCE self-completion, 平今归一化, ...) key off ExchangeID,
                // so a client that leaves it empty must not disable them
                exchange = info.exchange_id.clone();
        } else if info.exchange_id != exchange {
            return self.send_error(
                conn_id,
                frame.req_id,
                ERR_EXCHANGE_ID_INVALID,
                "CTP:无效的ExchangeID字段，请填入正确的ExchangeID",
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
        Err((code, msg)) => return self.send_error(conn_id, frame.req_id, code, &msg),
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

        if self.cfg.settlement_required && self.settlement_confirmed.get(&(broker.clone(), investor.clone())).map(String::as_str) != Some(self.vt_day().as_str()) {
            return self.send_error(conn_id, frame.req_id, 42, "CTP:结算结果未确认");
        }

        // 报单流控 (§8.3): the front-office per-investor每秒报单 budget —
        // inserts are their own stream, separate from cancels (notes/14
        // §A.3-02「这两个函数流控是分开计算的」).
        // Placed after the local field validation (a malformed order is a
        // field error, not a frequency one) and before any risk check —
        // an over-budget order never reaches the engine or the ledger.
        if !self.order_gate(&broker, &investor, GateStream::Insert) {
            self.journal_rejected_order(&broker, &investor, &intent, ERR_ORDER_FREQ, "CTP:下单频率限制");
            return self.send_error(conn_id, frame.req_id, ERR_ORDER_FREQ, "CTP:下单频率限制");
        }

        // 1) static validation (contract / price / volume)
        if let Err((code, msg)) = self.engine.check(&intent) {
            self.journal_rejected_order(&broker, &investor, &intent, code, &msg);
            return self.reject_insert(conn_id, frame.req_id, code, &msg, &frame.payload);
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
                return self.send_error(conn_id, frame.req_id, code, &msg);
            }
        }
        // Funds frozen at insert time. A close order reserves **no** margin —
        // it consumes an existing position, and the margin it will release is
        // only known once the fill tells us the leg (notes/04 附表: 平仓冻结的
        // 是持仓量，不是资金). Commission is estimated at the dearer of
        // 开仓 / 平今 so the freeze is never short; the release path stays
        // symmetric because it is pro-rata on this same number.
        let catalog = self.engine.catalog();
        let est_margin = if offset == OffsetFlag::Open {
            // The freeze must be priced on the **昨结算价**, which is the
            // `MarginPriceType == '1'` basis (notes/04 C2). Using the latest
            // print here instead would make the frozen amount drift with the
            // market, and the drift would not match what the position is
            // actually charged once it fills. Before the first tick of a
            // session there is no 昨结算 to be had, so the quote stands in.
            let basis = self
                .engine
                .pre_settlement(&instrument)
                .or_else(|| self.engine.last_price(&instrument))
                .unwrap_or(price_est);
            catalog.margin(
                &instrument,
                direction,
                catalog.margin_price_for(true, basis, price_est),
                input.VolumeTotalOriginal,
            )
        } else {
            0.0
        };
        let est_comm = catalog.estimated_commission(&instrument, price_est, input.VolumeTotalOriginal);
        if let Err(code) = self
            .ledger
            .freeze(&order_key, &broker, &investor, &instrument, PositionSide::of(direction), est_margin, est_comm, catalog)
        {
            // roll back the position reservation made above (close path)
            self.ledger.unfreeze_order(&order_key, self.engine.catalog());
            let msg = if code == ERR_FUNDS { "CTP:资金不足" } else { "报单被拒绝" };
            self.journal_rejected_order(&broker, &investor, &intent, code, msg);
            return self.send_error(conn_id, frame.req_id, code, msg);
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
                self.ledger.unfreeze_order(&order_key, self.engine.catalog());
                self.journal_rejected_order(&broker, &investor, &intent, error_id, &msg);
                self.reject_insert(conn_id, frame.req_id, error_id, &msg, &frame.payload);
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
                            if o.FrontID == front_id && o.SessionID == session_id
                                && cstr(&o.OrderRef) == order_ref
                            {
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
                        ("order_key".into(), json::s(&order_key)),
                        ("summary_kind".into(), json::s("request_with_final_outcome")),
                        ("final_order_sys_id".into(), json::s(&sys_id)),
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
                        // the instruction's own lifecycle: accepted at the
                        // front office = OSS '0' 报单已提交 (notes/05 §2.2);
                        // the rejected path journals '4', a cancel '3'/'5'
                        ("submit_status".into(), json::s("0")),
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
                // the instruction's own lifecycle: the insert was rejected
                ("submit_status".into(), json::s("4")),
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
            instrument_id: cstr(&action.InstrumentID),
        };

        // 报单流控 (§8.3): 撤单是与报单**分开计算**的独立预算流（notes/14
        // §A.3-02），混做报撤的策略不会被对方的流量挤占。撤单侧超限的显式
        // 症状正是文档口径的 OnRspOrderAction「CTP:下单频率限制」。
        if !self.order_gate(&broker, &investor, GateStream::Cancel) {
            self.journal_cancel_rejected(&broker, &investor, &q, ERR_ORDER_FREQ, "CTP:下单频率限制");
            // 撤单拒绝双面（官方报单回调规则 场景 6/7：先响应后回报）：
            // OnRspOrderAction（RSP_ERROR 完成挂起请求）紧接
            // OnErrRtnOrderAction（错单回报，带回客户端 InputOrderActionField）。
            self.send_error(conn_id, frame.req_id, ERR_ORDER_FREQ, "CTP:下单频率限制");
            return self.send_err_rtn_action(
                conn_id,
                frame.req_id,
                ERR_ORDER_FREQ,
                "CTP:下单频率限制",
                &frame.payload,
            );
        }

        // SDK ReqOrderAction：两条定位路线均必填合约；sysid 路线还必填交易所。
        // 错误码使用 error.xml 的 BAD_ORDER_ACTION_FIELD，而非报单字段错误 15。
        if q.instrument_id.is_empty()
            || (!q.order_sys_id.is_empty() && cstr(&action.ExchangeID).is_empty())
        {
            let code = 23; // BAD_ORDER_ACTION_FIELD
            let msg = "CTP:错误的报单操作字段";
            self.journal_cancel_rejected(&broker, &investor, &q, code, msg);
            self.send_error(conn_id, frame.req_id, code, msg);
            return self.send_err_rtn_action(conn_id, frame.req_id, code, msg, &frame.payload);
        }

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
                            // the instruction's own lifecycle: accepted
                            ("submit_status".into(), json::s("3")),
                            ("outcome".into(), json::obj_sorted(vec![("accepted".into(), json::b(true))])),
                        ]),
                    );
                }
            }
            Err((code, msg)) => {
                self.journal_cancel_rejected(&broker, &investor, &q, code, &msg);
                // 撤单拒绝双面（官方报单回调规则 场景 6/7：先响应后回报）：
                // OnRspOrderAction（RSP_ERROR 完成挂起请求）紧接
                // OnErrRtnOrderAction（错单回报，带回客户端 InputOrderActionField）。
                self.send_error(conn_id, frame.req_id, code, &msg);
                self.send_err_rtn_action(conn_id, frame.req_id, code, &msg, &frame.payload);
            }
        }
    }

    /// Journal a rejected cancel instruction (CTP 报盘拒绝 or the frequency
    /// gate): OrderSubmitStatus '5' (撤单已被拒绝) plus the outcome a
    /// replay re-checks against its own error_id.
    fn journal_cancel_rejected(
        &mut self,
        broker: &str,
        investor: &str,
        q: &CancelQuery,
        error_id: i32,
        msg: &str,
    ) {
        self.journal_record_json(
            "order_cancel",
            broker,
            investor,
            json::obj_sorted(vec![
                ("order_ref".into(), json::s(&q.order_ref)),
                ("order_sys_id".into(), json::s(&q.order_sys_id)),
                ("instrument".into(), json::s(&q.instrument_id)),
                ("submit_status".into(), json::s("5")),
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
        let pre_settlements = self.engine.pre_settlements();
        self.ledger
            .mark_to_market(self.engine.catalog(), &prices, &pre_settlements, &self.vt_trading_day);
    }

    /// 报单流控 (DESIGN §8.3, docs: 报单流控、查询流控和会话数控制):
    /// per-(broker, investor) budget per second, **one budget per stream** —
    /// `ReqOrderInsert` and `ReqOrderAction` are counted separately
    /// (notes/14 §A.3-02「这两个函数流控是分开计算的」), so a mixed
    /// insert/cancel strategy is limited per stream, never by their sum.
    /// This is the front-office half of CTP's 【程序化交易频繁报撤单管理】.
    ///
    /// Over-budget requests are rejected **outright** with
    /// 「CTP:下单频率限制」 — the modern front-office behavior (contrast the
    /// 2009 FAQ era default of 6 orders/second/session with over-limit
    /// requests silently queued, which the rules table can inject for
    /// testing legacy downstreams). The caller picks the reply surface:
    /// inserts answer `OnErrRtnOrderInsert`, cancels answer
    /// `OnRspOrderAction` (both must be hooked, §5.1).
    ///
    /// Wall-clock window, same as `qry_gate` (real CTP throttles on real
    /// time); a drive that exceeds the budget is replay-deterministic only
    /// when the replay reproduces the recording's pacing.
    ///
    /// Returns true when the request may proceed.
    fn order_gate(&mut self, broker: &str, investor: &str, stream: GateStream) -> bool {
        let now = Instant::now();
        let quota = self.cfg.order_freq.max(1);
        let entry = self
            .order_freq_windows
            .entry((broker.to_string(), investor.to_string(), stream))
            .or_insert((now, 0));
        let used = if now.duration_since(entry.0) < Duration::from_secs(1) {
            entry.1 += 1;
            entry.1
        } else {
            entry.0 = now;
            entry.1 = 1;
            1
        };
        used <= quota
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
                p.to_query_fields(&broker, &investor, &ex, &day, mult)
            })
            .flatten()
            .collect();
        for f in &positions {
            self.send_frame(
                conn_id,
                Frame::new(msgs::RSP_QRY_INVESTOR_POSITION, frame.req_id, struct_to_bytes(f)),
            );
        }
        self.send_frame(conn_id, Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()));
    }

    /// `ReqQryInvestorPositionDetail` — one row per open lot (notes/04 B2).
    ///
    /// This is the surface a client uses to check the core's 逐日盯市
    /// arithmetic: each row carries its own `OpenPrice`, `LastSettlementPrice`
    /// and `CloseProfitByDate`, so 昨仓 and 今仓 lots are visibly priced on
    /// different bases. Rows come out oldest-first per position (先开先平
    /// order), which is the order a client consumes them in.
    fn on_qry_investor_position_detail(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        self.mark_to_market_now();
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQryInvestorPositionDetailField =
            struct_from_bytes(&frame.payload).unwrap_or_else(CThostFtdcQryInvestorPositionDetailField::zeroed);
        let filter = cstr(&q.InstrumentID);
        let day = self.vt_day();
        let catalog = self.engine.catalog();
        let rows: Vec<_> = self
            .ledger
            .positions_of_ordered(&broker, &investor)
            .into_iter()
            .filter(|(p, _)| filter.is_empty() || p.instrument_id == filter)
            .map(|(p, d)| {
                let inst = catalog.get(&p.instrument_id);
                let ex = inst.map(|i| i.exchange_id.clone()).unwrap_or_default();
                let mult = inst.map(|i| i.volume_multiple).unwrap_or(1);
                let last = self
                    .engine
                    .last_price(&p.instrument_id)
                    .unwrap_or(p.settlement_price);
                d.to_field(
                    &broker,
                    &investor,
                    &ex,
                    &day,
                    p.side,
                    &p.instrument_id,
                    p.settlement_price,
                    last,
                    mult,
                )
            })
            .collect();
        for f in &rows {
            self.send_frame(
                conn_id,
                Frame::new(msgs::RSP_QRY_INVESTOR_POSITION_DETAIL, frame.req_id, struct_to_bytes(f)),
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
            // 首条 OrderSysID 为空，按稳定的前置/会话/引用关联整个生命周期。
            let key = format!("{}/{}/{}", o.FrontID, o.SessionID, cstr(&o.OrderRef));
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

    // ---- reference-data queries (notes/04 G) ----
    //
    // These four read the *same* rows the ledger computes from. That is the
    // whole point: a client that cross-checks `ReqQryInstrumentMarginRate`
    // against `ReqQryTradingAccount.CurrMargin` must see numbers that agree,
    // and on a real desk they do because there is only one rate table.
    //
    // Two official behaviours are reproduced deliberately (6.7.13 API docs,
    // "请求查询合约保证金率"):
    //
    //   1. `InstrumentID` empty → return the rates of the contracts the
    //      investor **currently holds**, not the whole market. The docs are
    //      explicit: "目前无法通过一次查询得到所有合约保证金率，如果要查询
    //      所有，则需要通过多次查询得到". A client that walks all contracts
    //      must issue one query per contract — so answering with all 789
    //      bundled contracts here would be wrong, not generous.
    //   2. `BrokerID` / `InvestorID` are mandatory and "不填的话返回值就为空".
    //
    /// The set of instruments a rate query may return, applying (1).
    fn rate_query_scope(
        &self,
        broker: &str,
        investor: &str,
        instrument_filter: &str,
    ) -> Vec<String> {
        if !instrument_filter.is_empty() {
            return vec![instrument_filter.to_string()];
        }
        // empty InstrumentID == "持仓对应的合约", deduplicated and ordered so
        // the row stream is reproducible across runs.
        let mut ids: Vec<String> = self
            .ledger
            .positions_of(broker, investor)
            .into_iter()
            .map(|p| p.instrument_id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

/// Empty result set: still a well-formed query stream — the client learns
    /// "nothing matched" from a clean QRY_LAST, not from a timeout. This is a
    /// distinct path because `Vec::new()` cannot infer the row type, and
    /// because it is genuinely the common case (no positions → no rows).
    fn send_qry_empty(&mut self, conn_id: u64, req_id: u32) {
        self.send_frame(conn_id, Frame::new(msgs::QRY_LAST, req_id, Vec::new()));
    }

    /// Common tail of the four reference-data queries: emit one frame per row,
    /// then terminate the stream. Generic because each query answers with its own
    /// CTP response struct — the shared part really is only "rows then QRY_LAST".
    fn send_rate_rows<T: Copy>(
        &mut self,
        conn_id: u64,
        req_id: u32,
        rsp_msg: u16,
        rows: Vec<T>,
    ) {
        for f in &rows {
            self.send_frame(conn_id, Frame::new(rsp_msg, req_id, struct_to_bytes(f)));
        }
        self.send_frame(conn_id, Frame::new(msgs::QRY_LAST, req_id, Vec::new()));
    }

    fn on_qry_instrument_margin_rate(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQryInstrumentMarginRateField = struct_from_bytes(&frame.payload)
            .unwrap_or_else(CThostFtdcQryInstrumentMarginRateField::zeroed);
        // (2) mandatory fields empty → empty result, not an error.
        if cstr(&q.BrokerID).is_empty() || cstr(&q.InvestorID).is_empty() {
            return self.send_qry_empty(conn_id, frame.req_id);
        }
        let filter = cstr(&q.InstrumentID);
        let hedge = if q.HedgeFlag == 0 {
            HEDGE_FLAG_SPECULATION
        } else {
            q.HedgeFlag
        };
        let scope = self.rate_query_scope(&broker, &investor, &filter);
        let catalog = self.engine.catalog();
        let rows: Vec<_> = scope
            .iter()
            .filter_map(|id| catalog.margin_rate(id))
            .map(|r| {
                let mut f = r.to_field();
                // Echo the session's identity and the requested hedge flag:
                // the client asked with its own account, not ours.
                set_cstr(&mut f.BrokerID, &broker);
                set_cstr(&mut f.InvestorID, &investor);
                set_cstr(&mut f.InstrumentID, &r.instrument_id);
                f.HedgeFlag = hedge;
                f
            })
            .collect();
        self.send_rate_rows(conn_id, frame.req_id, msgs::RSP_QRY_INSTRUMENT_MARGIN_RATE, rows);
    }

    fn on_qry_instrument_commission_rate(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQryInstrumentCommissionRateField = struct_from_bytes(&frame.payload)
            .unwrap_or_else(CThostFtdcQryInstrumentCommissionRateField::zeroed);
        if cstr(&q.BrokerID).is_empty() || cstr(&q.InvestorID).is_empty() {
            return self.send_qry_empty(conn_id, frame.req_id);
        }
        let filter = cstr(&q.InstrumentID);
        let scope = self.rate_query_scope(&broker, &investor, &filter);
        let catalog = self.engine.catalog();
        let rows: Vec<_> = scope
            .iter()
            .filter_map(|id| catalog.commission_rate(id))
            .map(|r| {
                let mut f = r.to_field();
                set_cstr(&mut f.BrokerID, &broker);
                set_cstr(&mut f.InvestorID, &investor);
                set_cstr(&mut f.InstrumentID, &r.instrument_id);
                f
            })
            .collect();
        self.send_rate_rows(
            conn_id,
            frame.req_id,
            msgs::RSP_QRY_INSTRUMENT_COMMISSION_RATE,
            rows,
        );
    }

    fn on_qry_instrument_order_comm_rate(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQryInstrumentOrderCommRateField = struct_from_bytes(&frame.payload)
            .unwrap_or_else(CThostFtdcQryInstrumentOrderCommRateField::zeroed);
        if cstr(&q.BrokerID).is_empty() || cstr(&q.InvestorID).is_empty() {
            return self.send_qry_empty(conn_id, frame.req_id);
        }
        let filter = cstr(&q.InstrumentID);
        let scope = self.rate_query_scope(&broker, &investor, &filter);
        let catalog = self.engine.catalog();
        // 申报费 (报单 + 撤单各一笔). P4 calls it 中金所特有, but the rule
        // here is deliberately **not** an exchange whitelist: a row exists
        // exactly when the supplied `order_comm_rates.jsonl` has one for that
        // instrument. So a desk that does charge 申报费 on SHFE gets SHFE
        // rows, and one that doesn't gets an empty stream — both correct.
        // An empty result is a real answer here, not "unimplemented".
        let rows: Vec<_> = scope
            .iter()
            .filter_map(|id| catalog.order_comm_rate(id))
            .map(|r| {
                let mut f = r.to_field();
                set_cstr(&mut f.BrokerID, &broker);
                set_cstr(&mut f.InvestorID, &investor);
                set_cstr(&mut f.InstrumentID, &r.instrument_id);
                f
            })
            .collect();
        self.send_rate_rows(
            conn_id,
            frame.req_id,
            msgs::RSP_QRY_INSTRUMENT_ORDER_COMM_RATE,
            rows,
        );
    }

    fn on_qry_investor_product_group_margin(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) { return; }
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQryInvestorProductGroupMarginField = struct_from_bytes(&frame.payload)
            .unwrap_or_else(CThostFtdcQryInvestorProductGroupMarginField::zeroed);
        if cstr(&q.BrokerID).is_empty() || cstr(&q.InvestorID).is_empty() {
            return self.send_qry_empty(conn_id, frame.req_id);
        }
        if cstr(&q.BrokerID) != broker || cstr(&q.InvestorID) != investor {
            return self.send_qry_empty(conn_id, frame.req_id);
        }
        // 官方查询页：ExchangeID/InvestUnitID 不是过滤条件，HedgeFlag 不需要填写。
        self.mark_to_market_now();
        let filter = cstr(&q.ProductGroupID);
        let day = self.clock_owned().0;
        let rows = self.ledger.product_group_margin(&broker, &investor, self.engine.catalog(), &day)
            .into_iter().filter(|f| filter.is_empty() || cstr(&f.ProductGroupID) == filter).collect();
        self.send_rate_rows(conn_id, frame.req_id, msgs::RSP_QRY_INVESTOR_PRODUCT_GROUP_MARGIN, rows);
    }

    fn on_qry_broker_trading_params(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQryBrokerTradingParamsField = struct_from_bytes(&frame.payload)
            .unwrap_or_else(CThostFtdcQryBrokerTradingParamsField::zeroed);
        // BrokerID / InvestorID / CurrencyID are all mandatory here.
        let currency = cstr(&q.CurrencyID);
        if cstr(&q.BrokerID).is_empty() || cstr(&q.InvestorID).is_empty() || currency.is_empty() {
            return self.send_qry_empty(conn_id, frame.req_id);
        }
        // One row per broker — this is where MarginPriceType comes from, and it
        // is the field the whole margin calculation hangs on.
        let catalog = self.engine.catalog();
        let p = catalog.trading_params().clone();
        let mut f = p.to_field();
        set_cstr(&mut f.BrokerID, &broker);
        set_cstr(&mut f.InvestorID, &investor);
        set_cstr(&mut f.CurrencyID, &currency);
        self.send_rate_rows(
            conn_id,
            frame.req_id,
            msgs::RSP_QRY_BROKER_TRADING_PARAMS,
            vec![f],
        );
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
        ERR_NO_CLOSE_TODAY_LEDGER => "CTP:平今仓位不足",
        ERR_NO_CLOSE_YD_LEDGER => "CTP:平昨仓位不足",
        _ => "CTP:平仓量超过持仓量",
    }
}
