//! Market-data subscription and the ReqQry* query handlers.

use super::*;

impl World {
    pub(crate) fn on_sub_md(&mut self, conn_id: u64, frame: &Frame, subscribe: bool) {
        let sz = size_of::<CThostFtdcSpecificInstrumentField>();
        if frame.payload.is_empty() || !frame.payload.len().is_multiple_of(sz) {
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
            let rsp = if subscribe {
                msgs::RSP_SUB_MD
            } else {
                msgs::RSP_UNSUB_MD
            };
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
    pub(crate) fn qry_gate(&mut self, conn_id: u64, req_id: u32) -> bool {
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
    pub(crate) fn mark_to_market_now(&mut self) {
        let prices = self.engine.last_prices();
        let pre_settlements = self.engine.pre_settlements();
        self.ledger.mark_to_market(
            self.engine.catalog(),
            &prices,
            &pre_settlements,
            &self.vt_trading_day,
        );
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
    pub(crate) fn order_gate(&mut self, broker: &str, investor: &str, stream: GateStream) -> bool {
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

    pub(crate) fn on_qry_instrument(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        // lenient: a malformed payload degrades to "no filter"
        let q: CThostFtdcQryInstrumentField =
            struct_from_bytes(&frame.payload).unwrap_or_else(CThostFtdcQryInstrumentField::zeroed);
        let (ex, ins, prod) = (
            cstr(&q.ExchangeID),
            cstr(&q.InstrumentID),
            cstr(&q.ProductID),
        );
        let mut fields: Vec<_> = self
            .engine
            .catalog()
            .iter()
            .filter(|i| ex.is_empty() || i.exchange_id == ex)
            .filter(|i| ins.is_empty() || i.instrument_id == ins)
            .filter(|i| prod.is_empty() || i.product_id == prod)
            .collect();
        fields.sort_by(|a, b| a.instrument_id.cmp(&b.instrument_id));
        let fields: Vec<_> = fields.into_iter().map(|i| i.to_field()).collect();
        for f in &fields {
            self.send_frame(
                conn_id,
                Frame::new(msgs::RSP_QRY_INSTRUMENT, frame.req_id, struct_to_bytes(f)),
            );
        }
        self.send_frame(
            conn_id,
            Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()),
        );
    }

    pub(crate) fn on_qry_trading_account(&mut self, conn_id: u64, frame: &Frame) {
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
        // A session only ever sees its own account: a foreign BrokerID /
        // InvestorID filter answers an empty stream, never another investor.
        let (qb, qi) = (cstr(&q.BrokerID), cstr(&q.InvestorID));
        if (!qb.is_empty() && qb != broker) || (!qi.is_empty() && qi != investor) {
            return self.send_qry_empty(conn_id, frame.req_id);
        }
        let day = self.vt_day();
        if let Some(a) = self.ledger.account(&broker, &investor) {
            let f = a.to_field(&day);
            self.send_frame(
                conn_id,
                Frame::new(
                    msgs::RSP_QRY_TRADING_ACCOUNT,
                    frame.req_id,
                    struct_to_bytes(&f),
                ),
            );
        }
        self.send_frame(
            conn_id,
            Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()),
        );
    }

    pub(crate) fn on_qry_investor_position(&mut self, conn_id: u64, frame: &Frame) {
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
        let mut held: Vec<_> = self
            .ledger
            .positions_of(&broker, &investor)
            .into_iter()
            .filter(|p| filter.is_empty() || p.instrument_id == filter)
            .collect();
        // HashMap order varies per process; clients diff these streams
        held.sort_by(|a, b| {
            a.instrument_id
                .cmp(&b.instrument_id)
                .then(a.side.cmp(&b.side))
        });
        let positions: Vec<_> = held
            .into_iter()
            .flat_map(|p| {
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
            .collect();
        for f in &positions {
            self.send_frame(
                conn_id,
                Frame::new(
                    msgs::RSP_QRY_INVESTOR_POSITION,
                    frame.req_id,
                    struct_to_bytes(f),
                ),
            );
        }
        self.send_frame(
            conn_id,
            Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()),
        );
    }

    /// `ReqQryInvestorPositionDetail` — one row per open lot (notes/04 B2).
    ///
    /// This is the surface a client uses to check the core's 逐日盯市
    /// arithmetic: each row carries its own `OpenPrice`, `LastSettlementPrice`
    /// and `CloseProfitByDate`, so 昨仓 and 今仓 lots are visibly priced on
    /// different bases. Rows come out oldest-first per position (先开先平
    /// order), which is the order a client consumes them in.
    pub(crate) fn on_qry_investor_position_detail(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
        self.mark_to_market_now();
        let (broker, investor) = match self.session(conn_id) {
            Some(v) => (v.0, v.1),
            None => return self.send_error(conn_id, frame.req_id, -3, "用户未登录"),
        };
        let q: CThostFtdcQryInvestorPositionDetailField = struct_from_bytes(&frame.payload)
            .unwrap_or_else(CThostFtdcQryInvestorPositionDetailField::zeroed);
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
                Frame::new(
                    msgs::RSP_QRY_INVESTOR_POSITION_DETAIL,
                    frame.req_id,
                    struct_to_bytes(f),
                ),
            );
        }
        self.send_frame(
            conn_id,
            Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()),
        );
    }

    pub(crate) fn on_qry_order(&mut self, conn_id: u64, frame: &Frame) {
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
        self.send_frame(
            conn_id,
            Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()),
        );
    }

    pub(crate) fn on_qry_trade(&mut self, conn_id: u64, frame: &Frame) {
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
        self.send_frame(
            conn_id,
            Frame::new(msgs::QRY_LAST, frame.req_id, Vec::new()),
        );
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
    pub(crate) fn rate_query_scope(
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
    pub(crate) fn send_qry_empty(&mut self, conn_id: u64, req_id: u32) {
        self.send_frame(conn_id, Frame::new(msgs::QRY_LAST, req_id, Vec::new()));
    }

    /// Common tail of the four reference-data queries: emit one frame per row,
    /// then terminate the stream. Generic because each query answers with its own
    /// CTP response struct — the shared part really is only "rows then QRY_LAST".
    pub(crate) fn send_rate_rows<T: ctpbuddy_wire::WireStruct>(
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

    pub(crate) fn on_qry_instrument_margin_rate(&mut self, conn_id: u64, frame: &Frame) {
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
        self.send_rate_rows(
            conn_id,
            frame.req_id,
            msgs::RSP_QRY_INSTRUMENT_MARGIN_RATE,
            rows,
        );
    }

    pub(crate) fn on_qry_instrument_commission_rate(&mut self, conn_id: u64, frame: &Frame) {
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

    pub(crate) fn on_qry_instrument_order_comm_rate(&mut self, conn_id: u64, frame: &Frame) {
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

    pub(crate) fn on_qry_investor_product_group_margin(&mut self, conn_id: u64, frame: &Frame) {
        if !self.qry_gate(conn_id, frame.req_id) {
            return;
        }
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
        let rows = self
            .ledger
            .product_group_margin(&broker, &investor, self.engine.catalog(), &day)
            .into_iter()
            .filter(|f| filter.is_empty() || cstr(&f.ProductGroupID) == filter)
            .collect();
        self.send_rate_rows(
            conn_id,
            frame.req_id,
            msgs::RSP_QRY_INVESTOR_PRODUCT_GROUP_MARGIN,
            rows,
        );
    }

    pub(crate) fn on_qry_broker_trading_params(&mut self, conn_id: u64, frame: &Frame) {
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
