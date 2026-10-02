//! Limit-order-book matching engine (DESIGN.md §8.4 mode 2).
//!
//! M2 scope:
//! - per-instrument book: bids (price DESC + arrival ASC) / asks (price ASC +
//!   arrival ASC); `books` is a `BTreeMap` keyed by instrument so QryOrder
//!   output order is deterministic;
//! - order-vs-order matching at arrival (price priority, then time
//!   priority): the fill price is the resting (maker) order's limit price;
//! - the incoming order compares the best resting price against the current
//!   tick's five-level depth and takes the better side; on a price tie the
//!   tick depth wins — the snapshot's volume entered the queue before the
//!   just-parked order (time priority). Tick depth is an immutable
//!   snapshot, so consumption is tracked per matching pass with
//!   `used: [i32; DEPTH]`;
//! - FAK/FOK exact semantics with the official CTP encodings
//!   (`ThostFtdcUserApiDataType.h`): FOK = TC_IOC('1')+VC_CV('3'),
//!   FAK = TC_IOC+VC_AV('1') or TC_IOC+VC_MV('2') with MinVolume; GFD('3')
//!   leftovers rest, IOC leftovers are cancelled;
//! - self-trade prevention: resting orders of the same (broker, investor)
//!   are skipped (switchable);
//! - callback sequence per DESIGN §8.9 / docs/notes/01: initial unknown
//!   ('a') push (OrderSubmitStatus '0'), rest confirmation as a single '3'
//!   push, every fill/cancel as 前态+新态 with OnRtnTrade after the new-state
//!   OnRtnOrder — OrderSubmitStatus flips to '3' on every row after the
//!   initial push (the exchange acted);
//! - DCE exception (notes/01 B3, landed M2-4): the '3' confirmation is
//!   pushed before matching (DCE returns it for every book-eligible order,
//!   even an immediate fill), and the self-completed all-traded report
//!   skips the 前态.
//!
//! Matching happens at two moments only: order arrival and tick arrival.
//! Resting orders never match each other directly (a tick is the external
//! counterparty), which keeps the engine deterministic.

use std::collections::{BTreeMap, HashMap, HashSet};

use ctpbuddy_market::{format_hhmmss, Tick, DEPTH};
use ctpbuddy_wire::generated::{
    cstr, set_cstr, CThostFtdcOrderField, CThostFtdcTradeField,
};

use crate::{to_fixed, Catalog, Direction, Fill, OffsetFlag, OrderIntent};

// ---- error ids: the official CTP table (SDK error.xml, docs/错误码全集.md) --
// Every code/prompt below is verbatim from `ctpsdk/6.7.13_20260225/td/win64/
// error.xml` — a downstream client keys off these numbers, so inventing or
// misremembering one defeats the simulation. `git log` before this pass had
// 11 of 12 constants wrong (e.g. freq as 91 instead of 116).
pub const ERR_INSTRUMENT_NOT_FOUND: i32 = 16; // INSTRUMENT_NOT_FOUND      CTP:找不到合约
pub const ERR_INSTRUMENT_NOT_TRADING: i32 = 17; // INSTRUMENT_NOT_TRADING  CTP:合约不能交易
pub const ERR_BAD_FIELD: i32 = 15; //           BAD_FIELD                 CTP:报单字段有误
/// ExchangeID 与合约实际所属交易所不符（合约存在但交易所字段填错）。
pub const ERR_EXCHANGE_ID_INVALID: i32 = 148; // EXCHANGE_ID_IS_INVALID CTP:无效的ExchangeID字段，请填入正确的ExchangeID
pub const ERR_PRICE_TICK: i32 = 165; //         PRICE_WRONG_TICK          CTP:报单价格非最小变动价位整数倍
pub const ERR_PRICE_LIMIT: i32 = 163; //        PRICE_OVER_LIMIT          CTP:报单价格不在涨跌停板价范围内
pub const ERR_VOLUME_RANGE: i32 = 164; //       VOLUME_NOT_VALID          CTP:下单数量不符合交易所规范
pub const ERR_DUPLICATE_ORDER: i32 = 22; //     DUPLICATE_ORDER_REF       CTP:报单错误：不允许重复报单
pub const ERR_NO_POSITION: i32 = 30; //         OVER_CLOSE_POSITION       CTP:平仓量超过持仓量
pub const ERR_NO_CLOSE_TODAY: i32 = 50; //      OVER_CLOSETODAY_POSITION  CTP:平今仓位不足
pub const ERR_INSUFFICIENT_MONEY: i32 = 31; //  INSUFFICIENT_MONEY        CTP:资金不足
/// 无对手盘（IOC/市价，当前深度为空）：交易所侧拒绝，CTP 统一包裹转发为
/// EXCHANGE_RTNERROR「CTP：交易所返回的错误」。
pub const ERR_NO_COUNTERPARTY: i32 = 91; //    EXCHANGE_RTNERROR          CTP：交易所返回的错误
pub const ERR_ORDER_NOT_FOUND: i32 = 25; //     ORDER_NOT_FOUND           CTP:撤单找不到相应报单
pub const ERR_ORDER_STATUS_UNSUITABLE: i32 = 26; // INSUITABLE_ORDER_STATUS CTP:报单已全成交或已撤销，不能再撤
/// 报单流控超限（柜台端【程序化交易频繁报撤单管理】每秒报撤笔数）：
/// 现代柜台口径显式拒绝「CTP:下单频率限制」；2009 FAQ 历史口径为每
/// 会话 6 笔/秒、超限排队不报错——后者可由规则表注入测试旧下游。
pub const ERR_ORDER_FREQ: i32 = 116; //        ORDER_FREQ_LIMIT          CTP:下单频率限制

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
    /// CTP `THOST_FTDC_OST_*`: 'a' unknown, '3' queued, '1' partial,
    /// '0' all traded, '5' canceled.
    pub status: u8,
    /// CTP `THOST_FTDC_OSS_*` — the *instruction's* submit state
    /// (notes/05 §2.2): '0' 报单已提交 on the initial CTP-accept push,
    /// '3' 已经接受 on every row after it (the exchange has confirmed the
    /// order into the book, filled it, or accepted its cancel — a
    /// self-canceled order stays '3', §5.5). '1' 撤单已提交 / '4' 报单
    /// 已被拒绝 / '5' 撤单已被拒绝 / '2','6' 修改相关 belong to
    /// instruction-level surfaces the engine does not emit as order rows;
    /// the server journals rejections with their submit status instead.
    pub submit_status: u8,
    pub notify_seq: i32,
    /// Monotonic arrival sequence (per engine): the time-priority tiebreaker.
    /// Assigned once at accept and never changes, so remove+reinsert keeps a
    /// same-price order's queue position.
    pub arrival_seq: u64,
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
    /// Requested instrument (journaling only; the engine resolves the order
    /// by the key fields above).
    pub instrument_id: String,
}

pub enum SubmitOutcome {
    Accepted { events: Vec<EngineEvent> },
    Rejected { error_id: i32, msg: String },
}

/// One side of an instrument's book, kept in price priority:
/// bids DESC (best bid first), asks ASC (best ask first); ties broken by
/// `arrival_seq` ASC (first in, first filled).
#[derive(Clone, Debug, Default)]
pub struct Book {
    pub bids: Vec<OrderRecord>,
    pub asks: Vec<OrderRecord>,
}

impl Book {
    fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }
}

/// The chosen liquidity source for one fill step.
enum Counterpart {
    /// Resting order at `idx` of the counterpart side of the taker's book.
    Book { idx: usize, price: f64, avail: i32 },
    /// Tick depth level (`None` = the depth-less last-price fallback).
    Market { level: Option<usize>, price: f64, avail: i32 },
}

pub struct MatchingEngine {
    catalog: Catalog,
    books: BTreeMap<String, Book>,
    last_md: HashMap<String, Tick>,
    /// (front_id, session_id, order_ref) of orders still on the book.
    active_refs: HashSet<(i32, i32, [u8; 13])>,
    /// Terminal orders (filled '0' / cancelled '5') kept so a later cancel of
    /// the same key answers the official INSUITABLE_ORDER_STATUS (26) instead
    /// of ORDER_NOT_FOUND (25) — real CTP distinguishes the two (报单回调
    /// 规则 场景 6/7).
    terminal_refs: HashMap<(i32, i32, [u8; 13]), u8>,
    next_sys: u64,
    next_trade: u64,
    next_notify: i32,
    next_arrival: u64,
    /// Self-trade prevention (DESIGN §8.3): resting orders of the same
    /// (broker, investor) are never matched against each other.
    self_trade_prevention: bool,
}

impl MatchingEngine {
    pub fn new(catalog: Catalog) -> Self {
        MatchingEngine {
            catalog,
            books: BTreeMap::new(),
            last_md: HashMap::new(),
            active_refs: HashSet::new(),
            terminal_refs: HashMap::new(),
            next_sys: 1,
            next_trade: 1,
            next_notify: 1,
            next_arrival: 1,
            self_trade_prevention: true,
        }
    }

    /// Retire an order: drop it from the active set and remember its terminal
    /// status for later cancel-answer fidelity.
    fn retire(&mut self, key: (i32, i32, [u8; 13]), status: u8) {
        self.active_refs.remove(&key);
        self.terminal_refs.insert(key, status);
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
        self.books.values().map(|b| b.bids.len() + b.asks.len()).sum()
    }

    /// Static validation (contract, price, volume). Fund/position sufficiency
    /// is the ledger's job and is checked by the server before `submit`.
    pub fn check(&self, intent: &OrderIntent) -> Result<(), (i32, String)> {
        let instr = match self.catalog.get(&intent.instrument_id) {
            Some(i) => i,
            None => return Err((ERR_INSTRUMENT_NOT_FOUND, "CTP:找不到合约".into())),
        };
        if !instr.is_trading {
            return Err((ERR_INSTRUMENT_NOT_TRADING, "CTP:合约不能交易".into()));
        }
        match intent.price_type {
            b'2' => {
                if intent.limit_price <= 0.0 {
                    return Err((ERR_BAD_FIELD, "CTP:报单字段有误".into()));
                }
                let aligned = ((intent.limit_price / instr.price_tick).round() * instr.price_tick
                    - intent.limit_price)
                    .abs();
                if aligned > EPS {
                    return Err((
                        ERR_PRICE_TICK,
                        "CTP:报单价格非最小变动价位整数倍".into(),
                    ));
                }
            }
            b'1' => {} // 任意价（市价）
            _ => {
                return Err((ERR_BAD_FIELD, "CTP:报单字段有误".into()))
            }
        }
        if intent.volume <= 0 {
            return Err((ERR_VOLUME_RANGE, "CTP:下单数量不符合交易所规范".into()));
        }
        let min_v = instr.min_volume(intent.price_type);
        let max_v = instr.max_volume(intent.price_type);
        if intent.volume < min_v || intent.volume > max_v {
            return Err((
                ERR_VOLUME_RANGE,
                "CTP:下单数量不符合交易所规范".into(),
            ));
        }
        Ok(())
    }

    /// Accept an order: match it against the book (order-vs-order) and the
    /// current tick depth, then rest (GFD) or cancel (IOC/FAK/FOK) the
    /// remainder. The emitted event sequence follows DESIGN §8.9.
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
                        msg: "CTP:报单价格不在涨跌停板价范围内".into(),
                    };
                }
            }
        }
        let key = (intent.front_id, intent.session_id, intent.order_ref);
        if self.active_refs.contains(&key) {
            return SubmitOutcome::Rejected {
                error_id: ERR_DUPLICATE_ORDER,
                msg: "CTP:报单错误：不允许重复报单".into(),
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
            status: b'a',
            submit_status: b'0',
            notify_seq: 0,
            arrival_seq: {
                let s = self.next_arrival;
                self.next_arrival += 1;
                s
            },
        };

        // Official CTP encodings (ThostFtdcUserApiDataType.h): TC_IOC='1',
        // TC_GFD='3'; VC_AV='1', VC_MV='2', VC_CV='3'. AnyPrice is IOC by
        // definition (normalized at the server; defended here too).
        let is_ioc = intent.price_type == b'1' || intent.time_condition == b'1';
        let is_fok = is_ioc && intent.volume_condition == b'3';
        let is_mv = is_ioc && intent.volume_condition == b'2';

        let mut events = Vec::new();
        // 1) initial unknown-order push (notes/01: every scenario starts here).
        //    OrderSubmitStatus '0' — 报单已提交 (CTP accepted the insert,
        //    notes/05 §2.2).
        let seq = self.next_notify_seq();
        rec.notify_seq = seq;
        events.push(EngineEvent::Order(build_order_field(&rec, ctx, seq)));
        // From here on the exchange acts on the order (confirm into the
        // book / fill / accept the cancel): every later row reports submit
        // status '3' 已经接受 — a self-canceled order stays '3' (§5.5).
        rec.submit_status = b'3';

        let md = self.last_md.get(&intent.instrument_id).cloned();

        // 2) FOK / FAK-with-min-volume lookahead: decide the whole order's
        //    fate from the immediately tradable depth before filling anything
        //    (docs/notes/01 D2: below MinVolume the entire order is cancelled).
        if is_fok || is_mv {
            let avail = self.available_depth(intent, md.as_ref());
            let threshold = if is_fok { intent.volume } else { intent.min_volume.max(1) };
            if avail < threshold {
                self.push_transition(&mut rec, b'5', &mut events, ctx);
                self.retire(key, b'5');
                return SubmitOutcome::Accepted { events };
            }
        }

        // 2b) 大商所报单确认 (notes/01 B3): DCE returns the 未成交 ('3')
        //     confirmation for EVERY order that enters its book — even one
        //     that fills immediately — so the '3' cannot wait for the rest
        //     branch: an immediate full fill must report 'a' → '3' → '0'.
        //     IOC-class instructions (FAK/FOK/market) never rest and get no
        //     such confirmation. `dce_confirmed` suppresses the duplicate
        //     rest-time push below.
        let dce_confirmed = if rec.exchange_id == "DCE" && !is_ioc {
            self.push_status(&mut rec, b'3', &mut events, ctx);
            true
        } else {
            false
        };

        // 3) matching loop: book counterpart vs tick counterpart, better price
        //    wins (ties to the resting order: it arrived before the snapshot)
        let mut used = [0i32; DEPTH];
        let mut remaining = intent.volume;
        while remaining > 0 {
            let cp = self.best_counterpart(intent, md.as_ref(), &used);
            match cp {
                None => break,
                Some(Counterpart::Book { idx, price, avail }) => {
                    let take = avail.min(remaining);
                    // one exchange trade, two reports: maker and taker share
                    // the TradeID (each keeps its own order key / direction /
                    // offset). take the maker out of the book; reinsert keeps
                    // its queue position via the stable arrival_seq
                    let mut maker = {
                        let book = self.books.get_mut(&rec.instrument_id).expect("book exists");
                        counterpart_side_mut(book, intent.direction).remove(idx)
                    };
                    let trade_id = self.next_trade_id();
                    self.emit_fill(&mut maker, price, take, &trade_id, &mut events, ctx);
                    self.emit_fill(&mut rec, price, take, &trade_id, &mut events, ctx);
                    if maker.volume_total > 0 {
                        let book = self.books.entry(rec.instrument_id.clone()).or_default();
                        insert_resting(book, maker);
                    } else {
                        // maker fully filled by this match: terminal '0'
                        self.retire((maker.front_id, maker.session_id, maker.order_ref), b'0');
                    }
                    remaining -= take;
                }
                Some(Counterpart::Market { level, price, avail }) => {
                    let take = avail.min(remaining);
                    if let Some(l) = level {
                        used[l] += take;
                    }
                    // a market fill has only the taker side to report
                    let trade_id = self.next_trade_id();
                    self.emit_fill(&mut rec, price, take, &trade_id, &mut events, ctx);
                    remaining -= take;
                }
            }
        }

        // 4) leftover: rest (GFD) or cancel (IOC/FAK/FOK)
        if remaining > 0 {
            if is_ioc {
                self.push_transition(&mut rec, b'5', &mut events, ctx);
                self.retire(key, b'5');
            } else {
                if rec.volume_traded == 0 && !dce_confirmed {
                    // exchange 报单确认: a single '3' push, no 前态 duplicate
                    // (notes/01 scenarios 1/3/4). DCE already pushed its '3'
                    // before matching (2b).
                    self.push_status(&mut rec, b'3', &mut events, ctx);
                }
                let book = self.books.entry(rec.instrument_id.clone()).or_default();
                insert_resting(book, rec);
            }
        } else {
            // fully filled at arrival: terminal '0'
            self.retire(key, b'0');
        }
        SubmitOutcome::Accepted { events }
    }

    /// Cancel an active order. Returns the §8.9 event pair (前态 + '5').
    pub fn cancel(
        &mut self,
        q: &CancelQuery,
        ctx: &ClockCtx,
    ) -> Result<Vec<EngineEvent>, (i32, String)> {
        // locate first (instrument, side, position) — immutable pass so the
        // event-emitting mutable pass cannot conflict with the search borrow
        let mut found: Option<(String, bool, usize)> = None;
        'search: for (instr, book) in &self.books {
            if let Some(pos) = book.bids.iter().position(|r| matches_cancel(r, q)) {
                found = Some((instr.clone(), true, pos));
                break 'search;
            }
            if let Some(pos) = book.asks.iter().position(|r| matches_cancel(r, q)) {
                found = Some((instr.clone(), false, pos));
                break 'search;
            }
        }
        let (instr, is_bid, pos) = match found {
            Some(v) => v,
            None => {
                // official split (报单回调规则 场景 6/7): an unknown ref is
                // ORDER_NOT_FOUND (25); a ref that already reached a terminal
                // state is INSUITABLE_ORDER_STATUS (26)
                let key = (q.front_id, q.session_id, {
                    let mut r = [0u8; 13];
                    let b = q.order_ref.as_bytes();
                    let n = b.len().min(13);
                    r[..n].copy_from_slice(&b[..n]);
                    r
                });
                return if self.terminal_refs.contains_key(&key) {
                    Err((ERR_ORDER_STATUS_UNSUITABLE,
                         "CTP:报单已全成交或已撤销，不能再撤".into()))
                } else {
                    Err((ERR_ORDER_NOT_FOUND, "CTP:撤单找不到相应报单".into()))
                };
            }
        };
        let book = self.books.get_mut(&instr).expect("book exists");
        let side = if is_bid { &mut book.bids } else { &mut book.asks };
        let mut rec = side.remove(pos);
        self.retire((rec.front_id, rec.session_id, rec.order_ref), b'5');
        let mut events = Vec::new();
        self.push_transition(&mut rec, b'5', &mut events, ctx);
        Ok(events)
    }

    /// Feed a market tick: update the book, fill any crossing resting orders.
    /// Deterministic: resting orders are processed in book order (price
    /// priority, then arrival), bids before asks; tick depth consumption is
    /// tracked per tick (`used` resets on every new snapshot).
    pub fn on_tick(&mut self, tick: &Tick) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        self.last_md.insert(tick.instrument_id.clone(), tick.clone());
        let book = match self.books.remove(&tick.instrument_id) {
            Some(b) => b,
            None => return events,
        };
        let ctx = ClockCtx {
            trading_day: &tick.trading_day,
            now_ms: tick.virtual_ms(),
        };
        let mut used = [0i32; DEPTH];
        let Book { bids, asks } = book;
        let mut leftover = Book::default();
        for mut rec in bids.into_iter().chain(asks.into_iter()) {
            let mut remaining = rec.volume_total;
            while remaining > 0 {
                match market_counterpart(tick, rec.direction, rec.price_type, rec.limit_price, &used) {
                    Some(m) => {
                        let take = m.avail.min(remaining);
                        if let Some(l) = m.level {
                            used[l] += take;
                        }
                        // the tick is the counterparty: one report, the taker's
                        let trade_id = self.next_trade_id();
                        self.emit_fill(&mut rec, m.price, take, &trade_id, &mut events, &ctx);
                        remaining -= take;
                    }
                    None => break,
                }
            }
            if rec.volume_total > 0 {
                match rec.direction {
                    Direction::Buy => leftover.bids.push(rec),
                    Direction::Sell => leftover.asks.push(rec),
                }
            } else {
                // tick filled the resting order: terminal '0'
                self.retire((rec.front_id, rec.session_id, rec.order_ref), b'0');
            }
        }
        if !leftover.is_empty() {
            self.books.insert(tick.instrument_id.clone(), leftover);
        }
        events
    }

    /// All resting orders as `OrderField`s (QryOrder support). Iteration is
    /// instrument-sorted (BTreeMap), then book order.
    pub fn active_order_fields(&self, trading_day: &str) -> Vec<CThostFtdcOrderField> {
        let ctx = ClockCtx {
            trading_day,
            now_ms: self.last_md_time(),
        };
        let mut out: Vec<CThostFtdcOrderField> = Vec::new();
        for book in self.books.values() {
            for rec in book.bids.iter().chain(book.asks.iter()) {
                let seq = self.next_notify;
                out.push(build_order_field(rec, &ctx, seq));
            }
        }
        out
    }

    fn last_md_time(&self) -> f64 {
        self.last_md.values().map(|t| t.virtual_ms()).fold(0.0, f64::max)
    }

    /// Total immediately tradable volume for `intent` (book + tick depth),
    /// used for the FOK / FAK-min-volume lookahead.
    fn available_depth(&self, intent: &OrderIntent, md: Option<&Tick>) -> i32 {
        let mut total: i32 = 0;
        if let Some(book) = self.books.get(&intent.instrument_id) {
            let side = counterpart_side(book, intent.direction);
            for o in side {
                if self.self_trade_prevention && same_account(o, intent) {
                    continue;
                }
                if !crosses_at(intent.direction, intent.price_type, intent.limit_price, o.limit_price) {
                    break; // side is price-sorted: nothing deeper qualifies
                }
                total = total.saturating_add(o.volume_total);
            }
        }
        if let Some(tick) = md {
            let mut used = [0i32; DEPTH];
            loop {
                match market_counterpart(tick, intent.direction, intent.price_type, intent.limit_price, &used) {
                    Some(m) => {
                        total = total.saturating_add(m.avail);
                        match m.level {
                            Some(l) => used[l] += m.avail,
                            None => break, // unbounded last-price fallback
                        }
                    }
                    None => break,
                }
            }
        }
        total
    }

    /// Best immediately tradable counterpart: the better price of the best
    /// resting order and the best tick depth level; on a price tie the tick
    /// depth wins — its volume entered the queue before the order that just
    /// parked (time priority over the snapshot).
    fn best_counterpart(
        &self,
        intent: &OrderIntent,
        md: Option<&Tick>,
        used: &[i32; DEPTH],
    ) -> Option<Counterpart> {
        let book_cp = self.books.get(&intent.instrument_id).and_then(|book| {
            let side = counterpart_side(book, intent.direction);
            side.iter()
                .position(|o| {
                    (!self.self_trade_prevention || !same_account(o, intent))
                        && crosses_at(intent.direction, intent.price_type, intent.limit_price, o.limit_price)
                })
                .map(|idx| {
                    let o = &side[idx];
                    Counterpart::Book { idx, price: o.limit_price, avail: o.volume_total }
                })
        });
        let mkt_cp = md
            .and_then(|t| market_counterpart(t, intent.direction, intent.price_type, intent.limit_price, used))
            .map(|m| Counterpart::Market { level: m.level, price: m.price, avail: m.avail });
        match (book_cp, mkt_cp) {
            (Some(b), Some(m)) => {
                let (bp, mp) = match (&b, &m) {
                    (Counterpart::Book { price: bp, .. }, Counterpart::Market { price: mp, .. }) => (*bp, *mp),
                    _ => unreachable!(),
                };
                let book_better = match intent.direction {
                    Direction::Buy => bp < mp - EPS,  // lower ask wins
                    Direction::Sell => bp > mp + EPS, // higher bid wins
                };
                Some(if book_better { b } else { m })
            }
            (Some(b), None) => Some(b),
            (None, Some(m)) => Some(m),
            (None, None) => None,
        }
    }

    /// Apply one fill and emit the full §8.9 event trio for this order:
    /// 前态 OnRtnOrder → 新态 OnRtnOrder → OnRtnTrade.
    ///
    /// `trade_id` is allocated by the caller: a book match passes the same id
    /// to both sides (one exchange trade, two reports), a market fill mints
    /// its own for the single taker report.
    ///
    /// 大商所特例 (notes/01 B3, landed M2-4): on a full fill DCE returns
    /// only the trade and CTP **self-completes** the all-traded order report
    /// **without repeating the previous state** — so the 前态 push is
    /// skipped exactly when this fill completes the order (`'0'`). Partial
    /// fills keep the general 前态+新态 rule; the '3' confirmation DCE
    /// always returns (even for an immediately-filled order) is handled in
    /// `submit` (2b).
    fn emit_fill(
        &mut self,
        rec: &mut OrderRecord,
        price: f64,
        volume: i32,
        trade_id: &str,
        events: &mut Vec<EngineEvent>,
        ctx: &ClockCtx,
    ) {
        // DCE self-completion: this fill takes the order to '0' and DCE
        // never repeats the 前态 for its self-completed all-traded report.
        let dce_self_complete = rec.exchange_id == "DCE" && rec.volume_total == volume;
        // 前态 (the state this order was last reported in)
        if !dce_self_complete {
            let mut prev = rec.clone();
            let seq_prev = self.next_notify_seq();
            prev.notify_seq = seq_prev;
            events.push(EngineEvent::Order(build_order_field(&prev, ctx, seq_prev)));
        }
        // 新态
        rec.volume_traded += volume;
        rec.volume_total -= volume;
        rec.status = if rec.volume_total == 0 { b'0' } else { b'1' };
        let seq_new = self.next_notify_seq();
        rec.notify_seq = seq_new;
        events.push(EngineEvent::Order(build_order_field(rec, ctx, seq_new)));

        let mut tf = CThostFtdcTradeField::zeroed();
        set_cstr(&mut tf.BrokerID, &cstr(&rec.broker_id));
        set_cstr(&mut tf.InvestorID, &cstr(&rec.investor_id));
        set_cstr(&mut tf.UserID, &cstr(&rec.user_id));
        set_cstr(&mut tf.OrderRef, &cstr(&rec.order_ref));
        set_cstr(&mut tf.OrderLocalID, &cstr(&rec.order_local_id));
        set_cstr(&mut tf.OrderSysID, &cstr(&rec.order_sys_id));
        set_cstr(&mut tf.InstrumentID, &rec.instrument_id);
        set_cstr(&mut tf.ExchangeID, &rec.exchange_id);
        set_cstr(&mut tf.TradeID, trade_id);
        set_cstr(&mut tf.TradingDay, ctx.trading_day);
        set_cstr(&mut tf.TradeDate, ctx.trading_day);
        set_cstr(&mut tf.TradeTime, &format_hhmmss(ctx.now_ms));
        tf.Direction = rec.direction.as_ctp();
        // 成交开平归一化 (§8.9): only SHFE/INE distinguish 平今/平昨 on the
        // trade report; every other exchange reports Close. The ledger sees
        // the true offset through `Fill.offset`.
        tf.OffsetFlag = trade_offset(rec).as_ctp();
        tf.HedgeFlag = rec.hedge_flag;
        tf.Price = price;
        tf.Volume = volume;
        tf.TradingRole = b'0';
        tf.TradeType = b'0';
        tf.PriceSource = b'0';
        tf.TradeSource = b'0';
        tf.SettlementID = 1;
        tf.BrokerOrderSeq = seq_new;
        tf.SequenceNo = seq_new;

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
            trade_id: to_fixed(trade_id),
            order_key: format!(
                "{}/{}/{}",
                rec.front_id,
                rec.session_id,
                cstr(&rec.order_ref)
            ),
        };
        events.push(EngineEvent::Trade { field: tf, fill });
    }

    /// Push a single order notification with a new status (no 前态 duplicate).
    fn push_status(
        &mut self,
        rec: &mut OrderRecord,
        status: u8,
        events: &mut Vec<EngineEvent>,
        ctx: &ClockCtx,
    ) {
        rec.status = status;
        let seq = self.next_notify_seq();
        rec.notify_seq = seq;
        events.push(EngineEvent::Order(build_order_field(rec, ctx, seq)));
    }

    /// Push a state transition per §8.9: 前态 then 新态.
    fn push_transition(
        &mut self,
        rec: &mut OrderRecord,
        status: u8,
        events: &mut Vec<EngineEvent>,
        ctx: &ClockCtx,
    ) {
        let mut prev = rec.clone();
        let seq_prev = self.next_notify_seq();
        prev.notify_seq = seq_prev;
        events.push(EngineEvent::Order(build_order_field(&prev, ctx, seq_prev)));
        self.push_status(rec, status, events, ctx);
    }

    fn next_notify_seq(&mut self) -> i32 {
        let s = self.next_notify;
        self.next_notify += 1;
        s
    }

    /// Mint the next exchange trade id. One id per exchange trade: the caller
    /// hands the same id to both sides of a book match.
    fn next_trade_id(&mut self) -> String {
        let s = format!("{:010}", self.next_trade);
        self.next_trade += 1;
        s
    }
}

/// The side of `book` an incoming `direction` trades against.
fn counterpart_side(book: &Book, direction: Direction) -> &Vec<OrderRecord> {
    match direction {
        Direction::Buy => &book.asks,
        Direction::Sell => &book.bids,
    }
}

fn counterpart_side_mut(book: &mut Book, direction: Direction) -> &mut Vec<OrderRecord> {
    match direction {
        Direction::Buy => &mut book.asks,
        Direction::Sell => &mut book.bids,
    }
}

/// Insert a resting order keeping price priority + arrival order.
fn insert_resting(book: &mut Book, rec: OrderRecord) {
    match rec.direction {
        Direction::Buy => {
            let pos = book
                .bids
                .iter()
                .position(|o| {
                    o.limit_price < rec.limit_price - EPS
                        || ((o.limit_price - rec.limit_price).abs() <= EPS
                            && o.arrival_seq > rec.arrival_seq)
                })
                .unwrap_or(book.bids.len());
            book.bids.insert(pos, rec);
        }
        Direction::Sell => {
            let pos = book
                .asks
                .iter()
                .position(|o| {
                    o.limit_price > rec.limit_price + EPS
                        || ((o.limit_price - rec.limit_price).abs() <= EPS
                            && o.arrival_seq > rec.arrival_seq)
                })
                .unwrap_or(book.asks.len());
            book.asks.insert(pos, rec);
        }
    }
}

/// Would an order with these terms trade at `price`? AnyPrice crosses at any
/// price; a limit order crosses at-or-through its limit on its own side.
fn crosses_at(direction: Direction, price_type: u8, limit_price: f64, price: f64) -> bool {
    if price_type == b'1' {
        return true;
    }
    match direction {
        Direction::Buy => limit_price + EPS >= price,
        Direction::Sell => limit_price <= price + EPS,
    }
}

fn same_account(o: &OrderRecord, intent: &OrderIntent) -> bool {
    cstr(&o.broker_id) == cstr(&intent.broker_id) && cstr(&o.investor_id) == cstr(&intent.investor_id)
}

/// One tradable slice of tick depth. `level` is `None` for the depth-less
/// last-price fallback (mode-1 degradation, unbounded volume).
struct MktCp {
    level: Option<usize>,
    price: f64,
    avail: i32,
}

/// Best immediately tradable tick-depth counterpart for an order. `used`
/// tracks consumption within the current matching pass (a tick is an
/// immutable snapshot; several fills eat through it).
fn market_counterpart(
    tick: &Tick,
    direction: Direction,
    price_type: u8,
    limit_price: f64,
    used: &[i32; DEPTH],
) -> Option<MktCp> {
    let (prices, volumes) = match direction {
        Direction::Buy => (&tick.ask_prices, &tick.ask_volumes),
        Direction::Sell => (&tick.bid_prices, &tick.bid_volumes),
    };
    if prices[0] <= 0.0 {
        // no depth data at all: degrade to mode 1 (last price, unbounded)
        if price_type != b'2' && tick.last_price > 0.0 {
            return Some(MktCp { level: None, price: tick.last_price, avail: i32::MAX / 2 });
        }
        return None;
    }
    for i in 0..DEPTH {
        let p = prices[i];
        if p <= 0.0 {
            break; // an empty level terminates the book
        }
        let avail = volumes[i] - used[i];
        if avail <= 0 {
            continue;
        }
        if !crosses_at(direction, price_type, limit_price, p) {
            break; // levels are price-sorted: nothing deeper qualifies
        }
        return Some(MktCp { level: Some(i), price: p, avail });
    }
    None
}

/// 成交开平归一化 (DESIGN §8.9): only SHFE/INE distinguish CloseToday/
/// CloseYesterday on the trade report; every other exchange reports Close.
fn trade_offset(rec: &OrderRecord) -> OffsetFlag {
    if rec.offset.is_close() && !matches!(rec.exchange_id.as_str(), "SHFE" | "INE") {
        OffsetFlag::Close
    } else {
        rec.offset
    }
}

fn matches_cancel(r: &OrderRecord, q: &CancelQuery) -> bool {
    if !q.order_sys_id.is_empty() {
        cstr(&r.order_sys_id) == q.order_sys_id
    } else if q.front_id != 0 {
        r.front_id == q.front_id
            && r.session_id == q.session_id
            && cstr(&r.order_ref) == q.order_ref
    } else {
        // ref-only fallback, restricted to the caller's investor
        cstr(&r.order_ref) == q.order_ref && cstr(&r.investor_id) == q.investor_id
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
    // OrderSubmitStatus: '0' on the initial CTP-accept push, '3' on every
    // row after it (the exchange acted: confirmed / filled / accepted the
    // cancel). see OrderRecord.submit_status for the full seven-state map.
    f.OrderSubmitStatus = rec.submit_status;
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
