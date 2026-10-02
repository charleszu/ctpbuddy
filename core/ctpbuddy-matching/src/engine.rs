//! Immediate-fill matching engine (DESIGN.md §8.4 mode 1).
//!
//! Scope of M1:
//! - 市价 / FAK / FOK: fill against the current counterpart price from the
//!   latest tick (SimNow / LocalCTP semantics);
//! - 限价 GFD: cross the book at arrival, otherwise rest and are re-evaluated
//!   on every subsequent tick of the same instrument;
//! - order-vs-order matching (price/time priority queue estimation) is
//!   deliberately **not** implemented yet — it lands with the limit-order book
//!   in M2 (DESIGN.md §8.4 mode 2). Resting orders in M1 fill against market
//!   ticks only.

use std::collections::{HashMap, HashSet};

use ctpbuddy_market::{format_hhmmss, Tick};
use ctpbuddy_wire::generated::{
    cstr, set_cstr, CThostFtdcOrderField, CThostFtdcTradeField,
};

use crate::{to_fixed, Catalog, Direction, Fill, OffsetFlag, OrderIntent};

// ---- error ids (recognizable against CTP's table; broker rules table TODO) ----
pub const ERR_INSTRUMENT_NOT_FOUND: i32 = 22; // 合约不存在或状态异常
pub const ERR_ORDER_STATUS: i32 = 36; //       合约状态不允许交易
pub const ERR_PRICE_TICK: i32 = 31; //          价格不符合最小变动价位
pub const ERR_PRICE_LIMIT: i32 = 33; //         价格超出涨跌停板
pub const ERR_DIRECTION: i32 = 40; //           买卖方向/开平标志/价格类型非法
pub const ERR_VOLUME_RANGE: i32 = 34; //        报单手数超出合约范围
pub const ERR_NO_POSITION: i32 = 30; //         平仓量超过持仓量
pub const ERR_NO_CLOSE_TODAY: i32 = 38; //      可平今仓不足
pub const ERR_NO_COUNTERPARTY: i32 = 42; //     无对手盘（IOC/市价，当前深度为空）
pub const ERR_ORDER_NOT_FOUND: i32 = 75; //     未找到报单（撤单）
pub const ERR_DUPLICATE_ORDER: i32 = 44; //     重复报单（活动报单引用冲突）

const EPS: f64 = 1e-9;

/// Virtual-clock snapshot handed to the engine on every call. The engine is
/// clock-free: `trading_day` + `now_ms` come from the world loop (playback
/// virtual time, or wall clock when no scenario is loaded).
pub struct ClockCtx<'a> {
    pub trading_day: &'a str,
    pub now_ms: f64,
}

/// A resting order. Mirrors the fields of `CThostFtdcInputOrderField` plus the
/// front/session routing the server needs to cancel it.
#[derive(Clone, Debug)]
pub struct OrderRecord {
    pub broker_id: [u8; 11],
    pub investor_id: [u8; 13],
    pub user_id: [u8; 16],
    pub order_ref: [u8; 13],
    pub order_local_id: [u8; 13],
    pub instrument_id: String,
    pub exchange_id: String,
    pub direction: Direction,
    pub offset: OffsetFlag,
    pub hedge_flag: u8,
    pub price_type: u8,
    pub limit_price: f64,
    pub volume_total_original: i32,
    pub volume_traded: i32,
    /// Remaining volume (original - traded); 0 removes the order.
    pub volume_total: i32,
    pub time_condition: u8,
    pub volume_condition: u8,
    pub min_volume: i32,
    pub contingent_condition: u8,
    pub stop_price: f64,
    pub force_close_reason: u8,
    pub request_id: i32,
    pub order_sys_id: [u8; 21],
    pub front_id: i32,
    pub session_id: i32,
    pub insert_ms: f64,
    /// CTP `THOST_FTDC_OST_*`: '3' queued, '1' partial, '0' all traded, '5' canceled.
    pub status: u8,
    pub notify_seq: i32,
}

/// What the engine emits. The server turns these into wire frames; the ledger
/// consumes [`EngineEvent::Trade::fill`].
pub enum EngineEvent {
    Order(CThostFtdcOrderField),
    Trade {
        field: CThostFtdcTradeField,
        fill: Fill,
    },
}

/// Cancel lookup key. Resolution order: `order_sys_id` when non-empty, else
/// exact (front_id, session_id, order_ref), else order_ref alone within the
/// investor's orders (clients that forget front/session routing).
#[derive(Clone, Debug, Default)]
pub struct CancelQuery {
    pub front_id: i32,
    pub session_id: i32,
    pub order_ref: String,
    pub order_sys_id: String,
    pub investor_id: String,
}

pub enum SubmitOutcome {
    Accepted { events: Vec<EngineEvent> },
    Rejected { error_id: i32, msg: String },
}

pub struct MatchingEngine {
    catalog: Catalog,
    books: HashMap<String, Vec<OrderRecord>>,
    last_md: HashMap<String, Tick>,
    /// (front_id, session_id, order_ref) of orders still on the book.
    active_refs: HashSet<(i32, i32, [u8; 13])>,
    next_sys: u64,
    next_trade: u64,
    next_notify: i32,
}

impl MatchingEngine {
    pub fn new(catalog: Catalog) -> Self {
        MatchingEngine {
            catalog,
            books: HashMap::new(),
            last_md: HashMap::new(),
            active_refs: HashSet::new(),
            next_sys: 1,
            next_trade: 1,
            next_notify: 1,
        }
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }

    pub fn last_price(&self, instrument_id: &str) -> Option<f64> {
        self.last_md.get(instrument_id).map(|t| t.last_price)
    }

    pub fn last_prices(&self) -> HashMap<String, f64> {
        self.last_md.iter().map(|(k, t)| (k.clone(), t.last_price)).collect()
    }

    pub fn open_order_count(&self) -> usize {
        self.books.values().map(|b| b.len()).sum()
    }

    /// Static validation (contract, price, volume). Fund/position sufficiency
    /// is the ledger's job and is checked by the server before `submit`.
    pub fn check(&self, intent: &OrderIntent) -> Result<(), (i32, String)> {
        let instr = match self.catalog.get(&intent.instrument_id) {
            Some(i) => i,
            None => {
                return Err((ERR_INSTRUMENT_NOT_FOUND, format!("合约 {} 不存在", intent.instrument_id)))
            }
        };
        if !instr.is_trading {
            return Err((ERR_ORDER_STATUS, format!("合约 {} 当前不可交易", intent.instrument_id)));
        }
        match intent.price_type {
            b'2' => {
                if intent.limit_price <= 0.0 {
                    return Err((ERR_PRICE_TICK, "限价单价格必须大于 0".into()));
                }
                let aligned = ((intent.limit_price / instr.price_tick).round() * instr.price_tick
                    - intent.limit_price)
                    .abs();
                if aligned > EPS {
                    return Err((
                        ERR_PRICE_TICK,
                        format!(
                            "价格 {} 不符合最小变动价位 {}",
                            intent.limit_price, instr.price_tick
                        ),
                    ));
                }
            }
            b'1' => {} // 任意价（市价）
            _ => {
                return Err((ERR_DIRECTION, format!("不支持的价格类型 '{}'", intent.price_type as char)))
            }
        }
        if intent.volume <= 0 {
            return Err((ERR_VOLUME_RANGE, "报单手数必须为正".into()));
        }
        let min_v = instr.min_volume(intent.price_type);
        let max_v = instr.max_volume(intent.price_type);
        if intent.volume < min_v || intent.volume > max_v {
            return Err((
                ERR_VOLUME_RANGE,
                format!("报单手数 {} 超出范围 [{}, {}]", intent.volume, min_v, max_v),
            ));
        }
        Ok(())
    }

    /// Accept an order. Immediate fills happen against the latest tick;
    /// leftovers rest (GFD) or die (IOC/FAK, FOK) — never queue for future
    /// order-vs-order matching in M1.
    pub fn submit(&mut self, intent: &OrderIntent, ctx: &ClockCtx) -> SubmitOutcome {
        if let Err(e) = self.check(intent) {
            return SubmitOutcome::Rejected { error_id: e.0, msg: e.1 };
        }
        // price band (涨跌停) check against the latest tick, when one exists
        if intent.price_type == b'2' {
            if let Some(md) = self.last_md.get(&intent.instrument_id) {
                let p = intent.limit_price;
                if p < md.lower_limit_price || p > md.upper_limit_price {
                    return SubmitOutcome::Rejected {
                        error_id: ERR_PRICE_LIMIT,
                        msg: format!(
                            "价格 {} 超出涨跌停板 [{}, {}]",
                            p, md.lower_limit_price, md.upper_limit_price
                        ),
                    };
                }
            }
        }
        let key = (intent.front_id, intent.session_id, intent.order_ref);
        if self.active_refs.contains(&key) {
            return SubmitOutcome::Rejected {
                error_id: ERR_DUPLICATE_ORDER,
                msg: "重复的报单引用（同 front/session/OrderRef 仍有活动报单）".into(),
            };
        }
        self.active_refs.insert(key);

        let sys_id = format!("{:010}", self.next_sys);
        self.next_sys += 1;
        let mut rec = OrderRecord {
            broker_id: intent.broker_id,
            investor_id: intent.investor_id,
            user_id: intent.user_id,
            order_ref: intent.order_ref,
            order_local_id: intent.order_local_id,
            instrument_id: intent.instrument_id.clone(),
            exchange_id: intent.exchange_id.clone(),
            direction: intent.direction,
            offset: intent.offset,
            hedge_flag: intent.hedge_flag,
            price_type: intent.price_type,
            limit_price: intent.limit_price,
            volume_total_original: intent.volume,
            volume_traded: 0,
            volume_total: intent.volume,
            time_condition: intent.time_condition,
            volume_condition: intent.volume_condition,
            min_volume: intent.min_volume,
            contingent_condition: intent.contingent_condition,
            stop_price: intent.stop_price,
            force_close_reason: intent.force_close_reason,
            request_id: intent.request_id,
            order_sys_id: to_fixed(&sys_id),
            front_id: intent.front_id,
            session_id: intent.session_id,
            insert_ms: ctx.now_ms,
            status: b'3',
            notify_seq: 0,
        };

        let md = self.last_md.get(&intent.instrument_id).cloned();
        let cp = md.as_ref().and_then(|t| {
            counterpart_price(intent.direction, intent.price_type, intent.limit_price, t)
        });
        let is_ioc = intent.time_condition == b'3';
        let is_fok = is_ioc && intent.volume_condition == b'2';

        let mut events = Vec::new();
        match cp {
            None => {
                // nothing to trade against right now
                if is_ioc {
                    rec.status = b'5';
                    self.active_refs.remove(&key);
                    let seq = self.next_notify_seq();
                    events.push(EngineEvent::Order(build_order_field(&rec, ctx, seq)));
                } else {
                    self.books.entry(rec.instrument_id.clone()).or_default().push(rec.clone());
                    let seq = self.next_notify_seq();
                    events.push(EngineEvent::Order(build_order_field(&rec, ctx, seq)));
                }
            }
            Some((price, avail)) => {
                if is_fok && avail < intent.volume {
                    rec.status = b'5';
                    self.active_refs.remove(&key);
                    let seq = self.next_notify_seq();
                    events.push(EngineEvent::Order(build_order_field(&rec, ctx, seq)));
                } else {
                    let vol = avail.min(intent.volume);
                    let ev = self.do_fill(&mut rec, price, vol, ctx);
                    events.push(ev);
                    if rec.volume_total == 0 {
                        rec.status = b'0';
                        self.active_refs.remove(&key);
                        let seq = self.next_notify_seq();
                        events.push(EngineEvent::Order(build_order_field(&rec, ctx, seq)));
                    } else if is_ioc {
                        rec.status = b'5';
                        self.active_refs.remove(&key);
                        let seq = self.next_notify_seq();
                        events.push(EngineEvent::Order(build_order_field(&rec, ctx, seq)));
                    } else {
                        rec.status = b'1';
                        self.books.entry(rec.instrument_id.clone()).or_default().push(rec.clone());
                        let seq = self.next_notify_seq();
                        events.push(EngineEvent::Order(build_order_field(&rec, ctx, seq)));
                    }
                }
            }
        }
        SubmitOutcome::Accepted { events }
    }

    /// Cancel an active order. Returns the final ('5') order record.
    pub fn cancel(&mut self, q: &CancelQuery, ctx: &ClockCtx) -> Result<CThostFtdcOrderField, (i32, String)> {
        for book in self.books.values_mut() {
            if let Some(pos) = book.iter().position(|r| {
                if !q.order_sys_id.is_empty() {
                    cstr(&r.order_sys_id) == q.order_sys_id
                } else if q.front_id != 0 {
                    r.front_id == q.front_id
                        && r.session_id == q.session_id
                        && cstr(&r.order_ref) == q.order_ref
                } else {
                    // ref-only fallback, restricted to the caller's investor
                    cstr(&r.order_ref) == q.order_ref
                        && cstr(&r.investor_id) == q.investor_id
                }
            }) {
                let mut rec = book.remove(pos);
                rec.status = b'5';
                self.active_refs.remove(&(rec.front_id, rec.session_id, rec.order_ref));
                let seq = self.next_notify_seq();
                return Ok(build_order_field(&rec, ctx, seq));
            }
        }
        Err((ERR_ORDER_NOT_FOUND, "未找到活动报单或报单状态不允许撤单".into()))
    }

    /// Feed a market tick: update the book, fill any crossing resting orders.
    /// Deterministic: resting orders are processed in arrival order.
    pub fn on_tick(&mut self, tick: &Tick) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        let book = match self.books.remove(&tick.instrument_id) {
            Some(b) => b,
            None => {
                self.last_md.insert(tick.instrument_id.clone(), tick.clone());
                return events;
            }
        };
        let ctx = ClockCtx {
            trading_day: &tick.trading_day,
            now_ms: tick.virtual_ms(),
        };
        let mut leftovers = Vec::new();
        for mut rec in book {
            match counterpart_price(rec.direction, rec.price_type, rec.limit_price, tick) {
                Some((price, avail)) => {
                    let vol = avail.min(rec.volume_total);
                    let ev = self.do_fill(&mut rec, price, vol, &ctx);
                    events.push(ev);
                    if rec.volume_total == 0 {
                        rec.status = b'0';
                        self.active_refs.remove(&(rec.front_id, rec.session_id, rec.order_ref));
                        let seq = self.next_notify_seq();
                        events.push(EngineEvent::Order(build_order_field(&rec, &ctx, seq)));
                    } else {
                        rec.status = b'1';
                        let seq = self.next_notify_seq();
                        events.push(EngineEvent::Order(build_order_field(&rec, &ctx, seq)));
                        leftovers.push(rec);
                    }
                }
                None => leftovers.push(rec),
            }
        }
        if !leftovers.is_empty() {
            self.books.insert(tick.instrument_id.clone(), leftovers);
        }
        self.last_md.insert(tick.instrument_id.clone(), tick.clone());
        events
    }

    /// All resting orders as `OrderField`s (QryOrder support).
    pub fn active_order_fields(&self, trading_day: &str) -> Vec<CThostFtdcOrderField> {
        let ctx = ClockCtx {
            trading_day,
            now_ms: self.last_md_time(),
        };
        let mut out: Vec<CThostFtdcOrderField> = Vec::new();
        for book in self.books.values() {
            for rec in book {
                let seq = self.next_notify;
                out.push(build_order_field(rec, &ctx, seq));
            }
        }
        out
    }

    fn last_md_time(&self) -> f64 {
        self.last_md.values().map(|t| t.virtual_ms()).fold(0.0, f64::max)
    }

    /// Apply one fill to `rec` and build the corresponding events.
    fn do_fill(
        &mut self,
        rec: &mut OrderRecord,
        price: f64,
        volume: i32,
        ctx: &ClockCtx,
    ) -> EngineEvent {
        let trade_id = format!("{:010}", self.next_trade);
        self.next_trade += 1;
        rec.volume_traded += volume;
        rec.volume_total -= volume;
        let seq = self.next_notify_seq();
        rec.notify_seq = seq;

        let mut tf = CThostFtdcTradeField::zeroed();
        set_cstr(&mut tf.BrokerID, &cstr(&rec.broker_id));
        set_cstr(&mut tf.InvestorID, &cstr(&rec.investor_id));
        set_cstr(&mut tf.UserID, &cstr(&rec.user_id));
        set_cstr(&mut tf.OrderRef, &cstr(&rec.order_ref));
        set_cstr(&mut tf.OrderLocalID, &cstr(&rec.order_local_id));
        set_cstr(&mut tf.OrderSysID, &cstr(&rec.order_sys_id));
        set_cstr(&mut tf.InstrumentID, &rec.instrument_id);
        set_cstr(&mut tf.ExchangeID, &rec.exchange_id);
        set_cstr(&mut tf.TradeID, &trade_id);
        set_cstr(&mut tf.TradingDay, ctx.trading_day);
        set_cstr(&mut tf.TradeDate, ctx.trading_day);
        set_cstr(&mut tf.TradeTime, &format_hhmmss(ctx.now_ms));
        tf.Direction = rec.direction.as_ctp();
        tf.OffsetFlag = rec.offset.as_ctp();
        tf.HedgeFlag = rec.hedge_flag;
        tf.Price = price;
        tf.Volume = volume;
        tf.TradingRole = b'0';
        tf.TradeType = b'0';
        tf.PriceSource = b'0';
        tf.TradeSource = b'0';
        tf.SettlementID = 1;
        tf.BrokerOrderSeq = seq;
        tf.SequenceNo = seq;

        let fill = Fill {
            broker_id: rec.broker_id,
            investor_id: rec.investor_id,
            user_id: rec.user_id,
            instrument_id: rec.instrument_id.clone(),
            exchange_id: rec.exchange_id.clone(),
            direction: rec.direction,
            offset: rec.offset,
            hedge_flag: rec.hedge_flag,
            price,
            volume,
            volume_total_original: rec.volume_total_original,
            order_sys_id: rec.order_sys_id,
            order_ref: rec.order_ref,
            trade_id: to_fixed(&trade_id),
            order_key: format!(
                "{}/{}/{}",
                rec.front_id,
                rec.session_id,
                cstr(&rec.order_ref)
            ),
        };
        EngineEvent::Trade { field: tf, fill }
    }

    fn next_notify_seq(&mut self) -> i32 {
        let s = self.next_notify;
        self.next_notify += 1;
        s
    }
}

/// Counterpart (price, available volume) for an order against a tick.
/// `None` means "not tradable right now" (no crossing / no depth).
fn counterpart_price(
    direction: Direction,
    price_type: u8,
    limit_price: f64,
    tick: &Tick,
) -> Option<(f64, i32)> {
    match direction {
        Direction::Buy => {
            let (ask, avail) = if tick.ask_prices[0] > 0.0 {
                (tick.ask_prices[0], tick.ask_volumes[0])
            } else if price_type != b'2' && tick.last_price > 0.0 {
                // immediate mode fallback without depth: last price, unbounded
                (tick.last_price, i32::MAX / 2)
            } else {
                return None;
            };
            if price_type == b'2' && limit_price + EPS < ask {
                return None;
            }
            Some((ask, avail))
        }
        Direction::Sell => {
            let (bid, avail) = if tick.bid_prices[0] > 0.0 {
                (tick.bid_prices[0], tick.bid_volumes[0])
            } else if price_type != b'2' && tick.last_price > 0.0 {
                (tick.last_price, i32::MAX / 2)
            } else {
                return None;
            };
            if price_type == b'2' && limit_price > bid + EPS {
                return None;
            }
            Some((bid, avail))
        }
    }
}

fn build_order_field(rec: &OrderRecord, ctx: &ClockCtx, notify_seq: i32) -> CThostFtdcOrderField {
    let mut f = CThostFtdcOrderField::zeroed();
    set_cstr(&mut f.BrokerID, &cstr(&rec.broker_id));
    set_cstr(&mut f.InvestorID, &cstr(&rec.investor_id));
    set_cstr(&mut f.UserID, &cstr(&rec.user_id));
    set_cstr(&mut f.OrderRef, &cstr(&rec.order_ref));
    set_cstr(&mut f.OrderLocalID, &cstr(&rec.order_local_id));
    set_cstr(&mut f.OrderSysID, &cstr(&rec.order_sys_id));
    set_cstr(&mut f.InstrumentID, &rec.instrument_id);
    set_cstr(&mut f.ExchangeID, &rec.exchange_id);
    f.OrderPriceType = rec.price_type;
    f.Direction = rec.direction.as_ctp();
    f.CombOffsetFlag[0] = rec.offset.as_ctp();
    f.CombHedgeFlag[0] = rec.hedge_flag;
    f.LimitPrice = rec.limit_price;
    f.VolumeTotalOriginal = rec.volume_total_original;
    f.TimeCondition = rec.time_condition;
    f.VolumeCondition = rec.volume_condition;
    f.MinVolume = rec.min_volume;
    f.ContingentCondition = rec.contingent_condition;
    f.StopPrice = rec.stop_price;
    f.ForceCloseReason = rec.force_close_reason;
    f.RequestID = rec.request_id;
    f.InstallID = 1;
    f.OrderSubmitStatus = b'0';
    f.NotifySequence = notify_seq;
    set_cstr(&mut f.TradingDay, ctx.trading_day);
    f.SettlementID = 1;
    f.OrderSource = b'0';
    f.OrderStatus = rec.status;
    f.OrderType = b'0';
    f.VolumeTraded = rec.volume_traded;
    f.VolumeTotal = rec.volume_total;
    set_cstr(&mut f.InsertDate, ctx.trading_day);
    set_cstr(&mut f.InsertTime, &format_hhmmss(rec.insert_ms));
    set_cstr(&mut f.UpdateTime, &format_hhmmss(ctx.now_ms));
    if rec.status == b'5' {
        set_cstr(&mut f.CancelTime, &format_hhmmss(ctx.now_ms));
    }
    f.FrontID = rec.front_id;
    f.SessionID = rec.session_id;
    f
}

// OffsetFlag 自成交预防等风控项在 M2 规则表落地（DESIGN.md §8.3）。
#[allow(dead_code)]
fn _offset_label(o: OffsetFlag) -> &'static str {
    o.label()
}
