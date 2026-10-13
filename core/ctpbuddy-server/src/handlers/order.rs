//! Order insert / cancel handlers and their rejection journaling.

use super::*;

const ERR_EXCHANGE_FREQ: i32 = 41; // CFFEX_OVER_REQUEST_PER_SECOND, SDK error.xml
const EXCHANGE_FREQ_MSG: &str = "CTP:交易所每秒发送请求数超过许可数";

#[derive(Debug)]
struct OrderInsertCommand {
    input: CThostFtdcInputOrderField,
    intent: OrderIntent,
    broker: String,
    investor: String,
    order_ref: String,
    order_key: String,
    instrument: String,
    exchange: String,
    direction: Direction,
    offset: OffsetFlag,
    time_condition: u8,
    volume_condition: u8,
}

#[derive(Debug)]
struct CancelCommand {
    action: CThostFtdcInputOrderActionField,
    query: CancelQuery,
}

impl World {
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
    pub(crate) fn reject_insert(
        &mut self,
        conn_id: u64,
        req_id: u32,
        code: i32,
        msg: &str,
        input: &[u8],
    ) {
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
            self.send_frame(
                conn_id,
                Frame::new(msgs::RSP_ORDER_INSERT, req_id, Vec::new()),
            );
            self.send_err_rtn(conn_id, req_id, code, msg, input);
        }
    }

    pub(crate) fn on_order_insert(&mut self, conn_id: u64, frame: &Frame) {
        let (broker, investor, user_id, front_id, session_id) = match self.session(conn_id) {
            Some(v) => v,
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let next_order_local = self
            .conns
            .get(&conn_id)
            .map(|c| c.order_local_seq + 1)
            .unwrap_or(1);
        let command = match decode_order_insert(
            &frame.payload,
            (broker, investor, user_id, front_id, session_id),
            self.engine.catalog(),
            next_order_local,
        ) {
            Ok(command) => command,
            Err((code, msg)) => return self.send_error(conn_id, frame.req_id, code, &msg),
        };
        if let Some(conn) = self.conns.get_mut(&conn_id) {
            conn.order_local_seq = next_order_local;
        }
        let OrderInsertCommand {
            input,
            intent,
            broker,
            investor,
            order_ref,
            order_key,
            instrument,
            exchange,
            direction,
            offset,
            time_condition: tc,
            volume_condition: vc,
        } = command;

        // Exchange-refused orders have already consumed their session ref even
        // though they never entered the matching engine's used-ref set.
        if self.orders_today.iter().any(|o| {
            o.FrontID == intent.front_id
                && o.SessionID == intent.session_id
                && cstr(&o.OrderRef) == order_ref
                && o.OrderSubmitStatus == b'4'
        }) {
            let msg = "CTP:报单错误：不允许重复报单";
            self.journal_rejected_order(&broker, &investor, &intent, ERR_DUPLICATE_ORDER, msg);
            return self.send_error(conn_id, frame.req_id, ERR_DUPLICATE_ORDER, msg);
        }

        if self.cfg.settlement_required
            && self
                .settlement_confirmed
                .get(&(broker.clone(), investor.clone()))
                .map(String::as_str)
                != Some(self.vt_day().as_str())
        {
            return self.send_error(conn_id, frame.req_id, 42, "CTP:结算结果未确认");
        }

        // 报单流控 (§8.3): the front-office per-investor每秒报单 budget —
        // inserts are their own stream, separate from cancels (notes/14
        // §A.3-02「这两个函数流控是分开计算的」).
        // Placed after the local field validation (a malformed order is a
        // field error, not a frequency one) and before any risk check —
        // an over-budget order never reaches the engine or the ledger.
        if !self.order_gate(&broker, &investor, GateStream::Insert) {
            self.journal_rejected_order(
                &broker,
                &investor,
                &intent,
                ERR_ORDER_FREQ,
                "CTP:下单频率限制",
            );
            return self.send_error(conn_id, frame.req_id, ERR_ORDER_FREQ, "CTP:下单频率限制");
        }

        if !self.exchange_order_gate(&broker, &investor, &exchange, GateStream::Insert) {
            self.journal_rejected_order(
                &broker,
                &investor,
                &intent,
                ERR_EXCHANGE_FREQ,
                EXCHANGE_FREQ_MSG,
            );
            self.reject_insert(
                conn_id,
                frame.req_id,
                ERR_EXCHANGE_FREQ,
                EXCHANGE_FREQ_MSG,
                &frame.payload,
            );
            // An exchange refusal reports a terminal rejected order, with no
            // reservation or fill. Never replace an already used order ref.
            if !self.orders_today.iter().any(|o| {
                o.FrontID == intent.front_id
                    && o.SessionID == intent.session_id
                    && cstr(&o.OrderRef) == order_ref
            }) {
                let mut order = CThostFtdcOrderField::zeroed();
                order.BrokerID = intent.broker_id;
                order.InvestorID = intent.investor_id;
                order.UserID = intent.user_id;
                order.OrderRef = intent.order_ref;
                order.OrderLocalID = intent.order_local_id;
                set_cstr(&mut order.InstrumentID, &instrument);
                set_cstr(&mut order.ExchangeID, &exchange);
                set_cstr(&mut order.TradingDay, &self.vt_day());
                set_cstr(&mut order.InsertDate, &self.vt_day());
                set_cstr(&mut order.InsertTime, &self.now_str());
                set_cstr(&mut order.UpdateTime, &self.now_str());
                set_cstr(&mut order.StatusMsg, EXCHANGE_FREQ_MSG);
                order.FrontID = intent.front_id;
                order.SessionID = intent.session_id;
                order.RequestID = input.RequestID;
                order.OrderPriceType = input.OrderPriceType;
                order.Direction = input.Direction;
                order.CombOffsetFlag = input.CombOffsetFlag;
                order.CombHedgeFlag = input.CombHedgeFlag;
                order.CombHedgeFlag[0] = intent.hedge_flag;
                order.LimitPrice = input.LimitPrice;
                order.VolumeTotalOriginal = input.VolumeTotalOriginal;
                order.TimeCondition = intent.time_condition;
                order.VolumeCondition = intent.volume_condition;
                order.OrderStatus = b'5';
                order.OrderSubmitStatus = b'4';
                self.dispatch_event(ctpbuddy_matching::EngineEvent::Order(order));
            }
            return;
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
            self.engine
                .last_price(&instrument)
                .unwrap_or(input.LimitPrice)
        };
        if offset.is_close() {
            // a sell closes a long position (and vice versa)
            let side = PositionSide::of(direction).opposite();
            if let Err(code) = self.ledger.freeze_close_position(
                &order_key,
                &broker,
                &investor,
                &instrument,
                &exchange,
                side,
                offset,
                input.VolumeTotalOriginal,
            ) {
                let msg = close_reject_msg(code);
                self.journal_rejected_order(&broker, &investor, &intent, code, msg);
                return self.send_error(conn_id, frame.req_id, code, msg);
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
            catalog.margin_for(
                &broker,
                &instrument,
                direction,
                catalog.margin_price_for(true, basis, price_est),
                input.VolumeTotalOriginal,
            )
        } else {
            0.0
        };
        let est_comm = catalog.estimated_commission_for(
            &broker,
            &instrument,
            price_est,
            input.VolumeTotalOriginal,
        );
        if let Err(code) = self.ledger.freeze(
            &order_key,
            &broker,
            &investor,
            &instrument,
            PositionSide::of(direction),
            est_margin,
            est_comm,
            catalog,
        ) {
            // roll back the position reservation made above (close path)
            self.ledger
                .unfreeze_order(&order_key, self.engine.catalog());
            let msg = if code == ERR_FUNDS {
                "CTP:资金不足"
            } else {
                "报单被拒绝"
            };
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
                self.ledger
                    .unfreeze_order(&order_key, self.engine.catalog());
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
                            if o.FrontID == front_id
                                && o.SessionID == session_id
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
                        (
                            "price_type".into(),
                            json::s(&(input.OrderPriceType as char).to_string()),
                        ),
                        ("limit_price".into(), json::n(input.LimitPrice)),
                        ("volume".into(), json::n(input.VolumeTotalOriginal as f64)),
                        // TC/VC/MinVolume complete the request so a journal
                        // replay can reconstruct FAK/FOK exactly (DESIGN §11.4)
                        ("time_condition".into(), json::s(&(tc as char).to_string())),
                        (
                            "volume_condition".into(),
                            json::s(&(vc as char).to_string()),
                        ),
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

    pub(crate) fn journal_rejected_order(
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
                (
                    "direction".into(),
                    json::n(intent.direction.as_ctp() as f64),
                ),
                ("offset".into(), json::n(intent.offset.as_ctp() as f64)),
                (
                    "price_type".into(),
                    json::s(&(intent.price_type as char).to_string()),
                ),
                ("limit_price".into(), json::n(intent.limit_price)),
                ("volume".into(), json::n(intent.volume as f64)),
                (
                    "time_condition".into(),
                    json::s(&(intent.time_condition as char).to_string()),
                ),
                (
                    "volume_condition".into(),
                    json::s(&(intent.volume_condition as char).to_string()),
                ),
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

    pub(crate) fn on_order_action(&mut self, conn_id: u64, frame: &Frame) {
        let (broker, investor, _user, _front, _sess) = match self.session(conn_id) {
            Some(v) => v,
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let command = match decode_order_action(&frame.payload, investor.clone()) {
            Ok(command) => command,
            Err((code, msg)) => return self.send_error(conn_id, frame.req_id, code, &msg),
        };
        let CancelCommand { action, query: q } = command;

        // 报单流控 (§8.3): 撤单是与报单**分开计算**的独立预算流（notes/14
        // §A.3-02），混做报撤的策略不会被对方的流量挤占。撤单侧超限的显式
        // 症状正是文档口径的 OnRspOrderAction「CTP:下单频率限制」。
        if !self.order_gate(&broker, &investor, GateStream::Cancel) {
            self.journal_cancel_rejected(
                &broker,
                &investor,
                &q,
                ERR_ORDER_FREQ,
                "CTP:下单频率限制",
            );
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
        if let Err((code, msg)) = validate_order_action(&q, &action) {
            self.journal_cancel_rejected(&broker, &investor, &q, code, msg);
            self.send_error(conn_id, frame.req_id, code, msg);
            return self.send_err_rtn_action(conn_id, frame.req_id, code, msg, &frame.payload);
        }

        let exchange = self
            .engine
            .catalog()
            .get(&q.instrument_id)
            .map(|info| info.exchange_id.clone())
            .unwrap_or_else(|| cstr(&action.ExchangeID));
        if !self.exchange_order_gate(&broker, &investor, &exchange, GateStream::Cancel) {
            self.journal_cancel_rejected(
                &broker,
                &investor,
                &q,
                ERR_EXCHANGE_FREQ,
                EXCHANGE_FREQ_MSG,
            );
            self.send_error(conn_id, frame.req_id, ERR_EXCHANGE_FREQ, EXCHANGE_FREQ_MSG);
            return self.send_err_rtn_action(
                conn_id,
                frame.req_id,
                ERR_EXCHANGE_FREQ,
                EXCHANGE_FREQ_MSG,
                &frame.payload,
            );
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
                        final_field = Some(*f);
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
                            (
                                "outcome".into(),
                                json::obj_sorted(vec![("accepted".into(), json::b(true))]),
                            ),
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
    pub(crate) fn journal_cancel_rejected(
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
}

fn decode_order_action(
    payload: &[u8],
    investor_id: String,
) -> Result<CancelCommand, (i32, String)> {
    let action: CThostFtdcInputOrderActionField =
        struct_from_bytes(payload).ok_or((-2, "撤单字段长度错误".to_string()))?;
    let query = CancelQuery {
        front_id: action.FrontID,
        session_id: action.SessionID,
        order_ref: cstr(&action.OrderRef),
        order_sys_id: cstr(&action.OrderSysID),
        investor_id,
        instrument_id: cstr(&action.InstrumentID),
    };
    Ok(CancelCommand { action, query })
}

fn validate_order_action(
    query: &CancelQuery,
    action: &CThostFtdcInputOrderActionField,
) -> Result<(), (i32, &'static str)> {
    if query.instrument_id.is_empty()
        || (!query.order_sys_id.is_empty() && cstr(&action.ExchangeID).is_empty())
    {
        return Err((23, "CTP:错误的报单操作字段"));
    }
    Ok(())
}

fn decode_order_insert(
    payload: &[u8],
    (broker, investor, user_id, front_id, session_id): (String, String, [u8; 16], i32, i32),
    catalog: &Catalog,
    order_local: u32,
) -> Result<OrderInsertCommand, (i32, String)> {
    let input: CThostFtdcInputOrderField =
        struct_from_bytes(payload).ok_or((-2, "报单字段长度错误".to_string()))?;
    if cstr(&input.BrokerID) != broker || cstr(&input.InvestorID) != investor {
        return Err((3, "报单 BrokerID/InvestorID 与登录会话不一致".to_string()));
    }
    let direction = Direction::from_ctp(input.Direction)
        .ok_or((ERR_BAD_FIELD, "CTP:报单字段有误".to_string()))?;
    if !input.LimitPrice.is_finite() || !input.StopPrice.is_finite() {
        return Err((ERR_BAD_FIELD, "CTP:报单字段有误".to_string()));
    }
    let offset = OffsetFlag::from_ctp(input.CombOffsetFlag[0])
        .ok_or((ERR_BAD_FIELD, "CTP:报单字段有误".to_string()))?;
    let instrument = cstr(&input.InstrumentID);
    let mut exchange = cstr(&input.ExchangeID);
    if instrument.is_empty() {
        return Err((ERR_BAD_FIELD, "CTP:报单字段有误".to_string()));
    }
    if let Some(info) = catalog.get(&instrument) {
        if exchange.is_empty() {
            // Backfill from the catalog so exchange-specific rules remain active.
            exchange = info.exchange_id.clone();
        } else if info.exchange_id != exchange {
            return Err((
                ERR_EXCHANGE_ID_INVALID,
                "CTP:无效的ExchangeID字段，请填入正确的ExchangeID".to_string(),
            ));
        }
    }
    let (time_condition, volume_condition) = normalize_conditions(
        input.OrderPriceType,
        input.TimeCondition,
        input.VolumeCondition,
        input.ContingentCondition,
    )?;
    let order_ref = if cstr(&input.OrderRef).is_empty() {
        order_local.to_string()
    } else {
        cstr(&input.OrderRef)
    };
    let order_local_id = order_local.to_string();
    let order_key = format!("{front_id}/{session_id}/{order_ref}");
    let intent = OrderIntent {
        broker_id: to_fixed::<11>(&broker),
        investor_id: to_fixed::<13>(&investor),
        user_id,
        order_ref: to_fixed::<13>(&order_ref),
        order_local_id: to_fixed::<13>(&order_local_id),
        instrument_id: instrument.clone(),
        exchange_id: exchange.clone(),
        direction,
        offset,
        hedge_flag: if input.CombHedgeFlag[0] == 0 {
            b'1'
        } else {
            input.CombHedgeFlag[0]
        },
        price_type: input.OrderPriceType,
        limit_price: input.LimitPrice,
        volume: input.VolumeTotalOriginal,
        time_condition,
        volume_condition,
        min_volume: input.MinVolume,
        contingent_condition: input.ContingentCondition,
        stop_price: input.StopPrice,
        force_close_reason: input.ForceCloseReason,
        request_id: input.RequestID,
        front_id,
        session_id,
    };
    Ok(OrderInsertCommand {
        input,
        intent,
        broker,
        investor,
        order_ref,
        order_key,
        instrument,
        exchange,
        direction,
        offset,
        time_condition,
        volume_condition,
    })
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
        _ => {
            return Err((
                ERR_UNSUPPORTED_FUNCTION,
                format!(
                    "{ERR_UNSUPPORTED_FUNCTION_MSG}：不支持的 VolumeCondition '{}'",
                    volume_condition as char
                ),
            ))
        }
    };
    if contingent_condition != b'1' {
        return Err((
            ERR_UNSUPPORTED_FUNCTION,
            format!(
                "{ERR_UNSUPPORTED_FUNCTION_MSG}：条件单暂不支持（ContingentCondition != 立即）"
            ),
        ));
    }
    let tc = match time_condition {
        b'1' => b'1', // IOC
        b'2' => b'3', // GFS → GFD (section 概念不建模，退化为当日有效)
        b'3' => b'3', // GFD
        b'4' => {
            return Err((
                ERR_UNSUPPORTED_FUNCTION,
                format!("{ERR_UNSUPPORTED_FUNCTION_MSG}：GTD（指定有效期）暂不支持"),
            ))
        }
        b'5' => {
            return Err((
                ERR_UNSUPPORTED_FUNCTION,
                format!("{ERR_UNSUPPORTED_FUNCTION_MSG}：GTC（撤销前有效）暂不支持"),
            ))
        }
        _ => {
            return Err((
                ERR_UNSUPPORTED_FUNCTION,
                format!(
                    "{ERR_UNSUPPORTED_FUNCTION_MSG}：不支持的 TimeCondition '{}'",
                    time_condition as char
                ),
            ))
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_order_insert_is_a_pure_command_seam() {
        let mut input = CThostFtdcInputOrderField::zeroed();
        set_cstr(&mut input.BrokerID, "8888");
        set_cstr(&mut input.InvestorID, "u1");
        set_cstr(&mut input.InstrumentID, "rb2601");
        input.OrderPriceType = b'2';
        input.Direction = b'0';
        input.CombOffsetFlag[0] = b'0';
        input.CombHedgeFlag[0] = b'1';
        input.TimeCondition = b'3';
        input.VolumeCondition = b'1';
        input.ContingentCondition = b'1';
        input.LimitPrice = 3_500.0;
        input.StopPrice = 0.0;
        input.VolumeTotalOriginal = 2;
        input.MinVolume = 1;

        let mut catalog = Catalog::new();
        catalog.insert(ctpbuddy_matching::Instrument::new("rb2601", "SHFE"));
        let command = decode_order_insert(
            &struct_to_bytes(&input),
            ("8888".into(), "u1".into(), [0; 16], 4, 5),
            &catalog,
            7,
        )
        .unwrap();

        assert_eq!(command.order_ref, "7");
        assert_eq!(command.order_key, "4/5/7");
        assert_eq!(command.exchange, "SHFE");
        assert_eq!(
            String::from_utf8_lossy(&command.intent.order_local_id).trim_end_matches('\0'),
            "7"
        );
        assert_eq!(command.intent.time_condition, b'3');
        assert_eq!(command.intent.volume_condition, b'1');
    }

    #[test]
    fn decode_order_insert_rejects_wire_shape_before_world_side_effects() {
        let error = decode_order_insert(
            &[0, 1, 2],
            ("8888".into(), "u1".into(), [0; 16], 1, 1),
            &Catalog::new(),
            1,
        )
        .unwrap_err();
        assert_eq!(error.0, -2);
        assert_eq!(error.1, "报单字段长度错误");
    }

    #[test]
    fn decode_order_action_keeps_wire_validation_separate_from_world_effects() {
        let mut action = CThostFtdcInputOrderActionField::zeroed();
        action.FrontID = 4;
        action.SessionID = 5;
        set_cstr(&mut action.OrderRef, "7");
        set_cstr(&mut action.InstrumentID, "rb2601");
        let command = decode_order_action(&struct_to_bytes(&action), "u1".into()).unwrap();

        assert_eq!(command.query.front_id, 4);
        assert_eq!(command.query.session_id, 5);
        assert_eq!(command.query.order_ref, "7");
        assert!(validate_order_action(&command.query, &command.action).is_ok());
    }

    #[test]
    fn validate_order_action_rejects_missing_instrument() {
        let action = CThostFtdcInputOrderActionField::zeroed();
        let command = decode_order_action(&struct_to_bytes(&action), "u1".into()).unwrap();
        assert_eq!(
            validate_order_action(&command.query, &command.action),
            Err((23, "CTP:错误的报单操作字段"))
        );
    }
}
