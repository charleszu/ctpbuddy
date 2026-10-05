use super::*;
use crate::Direction;

fn ctx() -> ClockCtx<'static> {
    ClockCtx {
        trading_day: "20260915",
        now_ms: 34_000.0,
    }
}

/// One GFD sell resting in the book, used as the maker a FAK taker hits.
fn maker(instrument: &str, exchange: &str, price: f64, volume: i32) -> OrderIntent {
    OrderIntent {
        broker_id: *b"SIM\0\0\0\0\0\0\0\0",
        investor_id: *b"INV0001\0\0\0\0\0\0",
        user_id: *b"user\0\0\0\0\0\0\0\0\0\0\0\0",
        order_ref: *b"M1\0\0\0\0\0\0\0\0\0\0\0",
        order_local_id: *b"L1\0\0\0\0\0\0\0\0\0\0\0",
        instrument_id: instrument.to_string(),
        exchange_id: exchange.to_string(),
        direction: Direction::Sell,
        offset: OffsetFlag::Open,
        hedge_flag: b'1',
        price_type: b'2',
        limit_price: price,
        volume,
        time_condition: b'3',
        volume_condition: b'1',
        min_volume: 0,
        contingent_condition: b'1',
        stop_price: 0.0,
        force_close_reason: 0,
        request_id: 1,
        front_id: 1,
        session_id: 1,
    }
}

#[test]
fn nonfinite_prices_never_change_the_book() {
    let mut engine = MatchingEngine::new(Catalog::bundled());
    for price in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for price_type in *b"12" {
            let mut intent = maker("rb2601", "SHFE", price, 1);
            intent.price_type = price_type;
            assert!(matches!(
                engine.submit(&intent, &ctx()),
                SubmitOutcome::Rejected {
                    error_id: ERR_BAD_FIELD,
                    ..
                }
            ));
            assert_eq!(engine.open_order_count(), 0);
            intent.limit_price = 3500.0;
            intent.stop_price = price;
            assert!(engine.check(&intent).is_err());
        }
    }
    assert!(matches!(
        engine.submit(&maker("rb2601", "SHFE", 3500.0, 1), &ctx()),
        SubmitOutcome::Accepted { .. }
    ));
}

/// A FAK buy: IOC + any-volume, crossing the resting ask. `investor_id`
/// differs from the maker's on purpose — the engine's self-trade
/// prevention would otherwise skip the only resting order.
fn fak_taker(order_ref: &str, volume: i32) -> OrderIntent {
    let mut ref13 = [0u8; 13];
    ref13[..order_ref.len()].copy_from_slice(order_ref.as_bytes());
    OrderIntent {
        investor_id: *b"INV0002\0\0\0\0\0\0",
        order_ref: ref13,
        order_local_id: *b"L2\0\0\0\0\0\0\0\0\0\0\0",
        direction: Direction::Buy,
        volume,
        time_condition: b'1',
        volume_condition: b'1',
        request_id: 2,
        ..maker("rb2601", "SHFE", 0.0, 0)
    }
}

/// `(OrderStatus, is_trade)` per event plus each order row's volumes — the
/// shape a downstream client actually counts.
fn shape(events: &[EngineEvent]) -> Vec<(u8, bool, i32, i32)> {
    events
        .iter()
        .map(|e| match e {
            EngineEvent::Order(f) => (f.OrderStatus, false, f.VolumeTraded, f.VolumeTotal),
            EngineEvent::Trade { field, .. } => (0u8, true, field.Volume, 0),
        })
        .collect()
}

fn statuses(events: &[EngineEvent]) -> Vec<(u8, bool)> {
    events
        .iter()
        .map(|e| match e {
            EngineEvent::Order(f) => (f.OrderStatus, false),
            EngineEvent::Trade { .. } => (b'T', true),
        })
        .collect()
}

/// Park a 3-lot maker, then let a 13-lot FAK taker cross it: the taker
/// gets 3 of 13 filled and the rest canceled — the 「部分成交部分撤单」
/// shape scenarios 8/9/10 describe. `submit` returns **both** sides of
/// the match, so keep only the taker's rows (maker uses OrderRef `M1`).
fn fak_scenario(instrument: &str, exchange: &str, price: f64) -> Vec<EngineEvent> {
    let mut e = MatchingEngine::new(Catalog::bundled());
    let rest = e.submit(&maker(instrument, exchange, price, 3), &ctx());
    assert!(matches!(rest, SubmitOutcome::Accepted { .. }));
    let mut taker = fak_taker("T1", 13);
    taker.instrument_id = instrument.to_string();
    taker.exchange_id = exchange.to_string();
    taker.limit_price = price;
    let events = match e.submit(&taker, &ctx()) {
        SubmitOutcome::Accepted { events } => events,
        SubmitOutcome::Rejected { error_id, msg } => panic!("FAK rejected: {error_id} {msg}"),
    };
    let taker_only: Vec<EngineEvent> = events
        .into_iter()
        .filter(|ev| match ev {
            EngineEvent::Order(f) => cstr(&f.OrderRef) == *"T1",
            EngineEvent::Trade { field, .. } => cstr(&field.OrderRef) == *"T1",
        })
        .collect();
    assert!(!taker_only.is_empty(), "taker must report something");
    taker_only
}

// 官方《报单回调规则》场景 8：上期所 FAK 部成部撤 —— 撤单状态回报**先于**
// 成交回报，且每笔成交只有一行 '5'（状态已终态，不重复推前态）。无 '3'、无 '1'。
#[test]
fn fak_shfe_puts_cancel_before_trades() {
    let ev = fak_scenario("rb2601", "SHFE", 3500.0);
    assert_eq!(
        statuses(&ev),
        vec![(b'a', false), (b'5', false), (b'5', false), (b'T', true)],
        "'a' → 撤单行 → 每笔成交一行 '5' + Trade",
    );
    match &ev[1] {
        EngineEvent::Order(f) => {
            assert_eq!(f.OrderStatus, b'5');
            assert_eq!(f.VolumeTraded, 3, "VolumeTraded 此时已有值");
            assert_eq!(f.VolumeTotal, 10);
        }
        _ => unreachable!(),
    }
}

// 官方场景 9：大商所 FAK —— 进簿确认 '3' 先到，每笔成交只推**一行**合成的
// '1'（不重复前态），最后一行 '5'。与郑商所形状不同（场景 10）。
#[test]
fn fak_dce_synthesizes_one_partial_row_per_trade() {
    let ev = fak_scenario("jd2602", "DCE", 900.0);
    assert_eq!(
        statuses(&ev),
        vec![
            (b'a', false),
            (b'3', false),
            (b'1', false),
            (b'T', true),
            (b'5', false)
        ],
        "confirm, one synthesized 部分成交, trade, cancel",
    );
}

// 官方场景 10：郑商所 FAK —— 进簿确认 '3' 先到，成交回报走**一般**的前态+
// 新态（'3' → '1'），最后一行 '5'。
#[test]
fn fak_czce_repeats_previous_state() {
    let ev = fak_scenario("SM602", "CZCE", 4000.0);
    assert_eq!(
        statuses(&ev),
        vec![
            (b'a', false),
            (b'3', false),
            (b'3', false),
            (b'1', false),
            (b'T', true),
            (b'5', false),
        ],
        "confirm, then 前态+新态 per trade, then cancel",
    );
}

// 三个所族必须给出三种不同形状 —— 这正是 #43 存在的理由：按条数计数、
// 或按 '1' 累加成交的下游代码，在其中两组上会静默算错。
#[test]
fn the_three_exchange_groups_do_not_agree() {
    let shfe = statuses(&fak_scenario("rb2601", "SHFE", 3500.0));
    let dce = statuses(&fak_scenario("jd2602", "DCE", 900.0));
    let czce = statuses(&fak_scenario("SM602", "CZCE", 4000.0));
    let gfex = statuses(&fak_scenario("si2602", "GFEX", 5000.0));
    assert_ne!(shfe, dce);
    assert_ne!(dce, czce);
    assert_ne!(shfe, czce);
    assert_eq!(dce, gfex, "GFEX follows the DCE group (场景 9)");
}

// 上期所那条 '5' 的 VolumeTraded/VolumeTotal 语义：撤单回报带走的量与成交
// 回报一致，不能把成交又算一遍。
#[test]
fn fak_shfe_cancel_row_does_not_double_count() {
    let ev = fak_scenario("rb2601", "SHFE", 3500.0);
    let rows = shape(&ev);
    // rows[0] 是初始 'a'，rows[1] 才是那条带成交量的撤单行
    assert_eq!(rows[1].2, 3, "cancel row VolumeTraded");
    assert_eq!(rows[1].3, 10, "cancel row VolumeTotal = 13 - 3");
    let traded: i32 = rows
        .iter()
        .filter(|(_, is_trade, ..)| *is_trade)
        .map(|(_, _, v, _)| *v)
        .sum();
    assert_eq!(traded, 3, "trade volume matches the cancel row");
}

// FAK 一手都没成交：交易所主动撤单，三个所都是**一行**终态，不带前态重复。
#[test]
fn fak_with_no_fill_is_a_single_cancel_row() {
    let mut e = MatchingEngine::new(Catalog::bundled());
    let mut taker = fak_taker("T1", 3);
    taker.instrument_id = "au2602".into();
    taker.exchange_id = "SHFE".into();
    taker.limit_price = 100.0; // 空簿 → 无成交
    let SubmitOutcome::Accepted { events } = e.submit(&taker, &ctx()) else {
        panic!("FAK must be accepted then canceled, not rejected");
    };
    assert_eq!(statuses(&events), vec![(b'a', false), (b'5', false)]);
}

fn order_ref13(s: &str) -> [u8; 13] {
    let mut r = [0u8; 13];
    r[..s.len()].copy_from_slice(s.as_bytes());
    r
}

fn accepted(outcome: SubmitOutcome) -> Vec<EngineEvent> {
    match outcome {
        SubmitOutcome::Accepted { events } => events,
        SubmitOutcome::Rejected { error_id, msg } => panic!("rejected: {error_id} {msg}"),
    }
}

/// rb2601 tick with one bid level and one ask level.
fn rb_tick(bid: f64, bid_v: i32, ask: f64, ask_v: i32) -> Tick {
    let mut t = Tick::default();
    t.instrument_id = "rb2601".into();
    t.exchange_id = "SHFE".into();
    t.trading_day = "20260915".into();
    t.update_time = "09:30:00".into();
    t.last_price = (bid + ask) / 2.0;
    t.bid_prices[0] = bid;
    t.bid_volumes[0] = bid_v;
    t.ask_prices[0] = ask;
    t.ask_volumes[0] = ask_v;
    t
}

// 跨账户撤单：B 的单对 A 来说就是「不存在」，三条定位路线（sysid / 三元组 /
// 仅 ref）都必须校验 investor，且 B 的单仍然挂着。
#[test]
fn cancel_never_crosses_investors() {
    let mut e = MatchingEngine::new(Catalog::bundled());
    let mut b = maker("rb2601", "SHFE", 3500.0, 2);
    b.investor_id = *b"INV0002\0\0\0\0\0\0";
    let events = accepted(e.submit(&b, &ctx()));
    let sys_id = events
        .iter()
        .rev()
        .find_map(|ev| match ev {
            EngineEvent::Order(f) => Some(cstr(&f.OrderSysID)),
            _ => None,
        })
        .expect("order row");
    assert!(!sys_id.is_empty(), "rest confirmation publishes OrderSysID");
    assert_eq!(e.open_order_count(), 1);

    // A tries all three routes against B's order
    let routes = [
        CancelQuery {
            order_sys_id: sys_id.clone(),
            investor_id: "INV0001".into(),
            ..Default::default()
        },
        CancelQuery {
            front_id: 1,
            session_id: 1,
            order_ref: "M1".into(),
            investor_id: "INV0001".into(),
            ..Default::default()
        },
        CancelQuery {
            order_ref: "M1".into(),
            investor_id: "INV0001".into(),
            ..Default::default()
        },
    ];
    for q in &routes {
        match e.cancel(q, &ctx()) {
            Err(err) => assert_eq!(err.0, ERR_ORDER_NOT_FOUND, "{q:?}"),
            Ok(_) => panic!("A must not cancel B's order: {q:?}"),
        }
        assert_eq!(e.open_order_count(), 1, "B's order still resting");
    }
    // B itself can cancel through the sysid route
    let q = CancelQuery {
        order_sys_id: sys_id,
        investor_id: "INV0002".into(),
        ..Default::default()
    };
    let ev = e.cancel(&q, &ctx()).expect("owner cancels");
    assert_eq!(statuses(&ev), vec![(b'3', false), (b'5', false)]);
    assert_eq!(e.open_order_count(), 0);
}

// on_tick：买单吃 ask 档、卖单吃 bid 档，两侧消耗计数必须独立——买侧吃光
// ask1 的 5 手不能让卖侧在 bid1 上少成交一手。
#[test]
fn tick_depth_consumption_is_tracked_per_side() {
    let mut e = MatchingEngine::new(Catalog::bundled());
    // 卖单 3510 与买单 3490：不交叉，都挂着
    let mut sell = maker("rb2601", "SHFE", 3510.0, 5);
    sell.order_ref = order_ref13("S1");
    accepted(e.submit(&sell, &ctx()));
    let mut buy = maker("rb2601", "SHFE", 3490.0, 5);
    buy.order_ref = order_ref13("B1");
    buy.direction = Direction::Buy;
    accepted(e.submit(&buy, &ctx()));
    assert_eq!(e.open_order_count(), 2);

    // tick: bid1 3520×5 (crosses the resting ask), ask1 3480×5 (crosses
    // the resting bid). Each side has exactly enough for its taker.
    let events = e.on_tick(&rb_tick(3520.0, 5, 3480.0, 5));
    let traded: i32 = events
        .iter()
        .filter_map(|ev| match ev {
            EngineEvent::Trade { field, .. } => Some(field.Volume),
            _ => None,
        })
        .sum();
    assert_eq!(traded, 10, "both sides fully filled: 5 + 5");
    assert_eq!(e.open_order_count(), 0, "nothing left resting");
}

// 自成交预防默认关闭：同账户交叉直接成交；显式开启后才跳过同账户挂单。
#[test]
fn self_trade_prevention_is_off_by_default() {
    let mut e = MatchingEngine::new(Catalog::bundled());
    assert!(!e.self_trade_prevention());
    accepted(e.submit(&maker("rb2601", "SHFE", 3500.0, 1), &ctx()));
    let mut buy = maker("rb2601", "SHFE", 3500.0, 1);
    buy.order_ref = order_ref13("B1");
    buy.direction = Direction::Buy;
    let ev = accepted(e.submit(&buy, &ctx()));
    assert!(
        ev.iter().any(|x| matches!(x, EngineEvent::Trade { .. })),
        "same-account orders trade when prevention is off"
    );
    assert_eq!(e.open_order_count(), 0);

    let mut e = MatchingEngine::new(Catalog::bundled());
    e.set_self_trade_prevention(true);
    accepted(e.submit(&maker("rb2601", "SHFE", 3500.0, 1), &ctx()));
    let ev = accepted(e.submit(&buy, &ctx()));
    assert!(
        !ev.iter().any(|x| matches!(x, EngineEvent::Trade { .. })),
        "same-account resting order skipped when prevention is on"
    );
    assert_eq!(e.open_order_count(), 2);
}

// OrderRef 在同一会话内终身唯一：终态（已撤 / 全成）单的 ref 再用也是 22；
// 换一个会话则可以复用。
#[test]
fn terminal_order_ref_is_still_a_duplicate_within_session() {
    let mut e = MatchingEngine::new(Catalog::bundled());
    accepted(e.submit(&maker("rb2601", "SHFE", 3500.0, 1), &ctx()));
    let q = CancelQuery {
        front_id: 1,
        session_id: 1,
        order_ref: "M1".into(),
        investor_id: "INV0001".into(),
        ..Default::default()
    };
    e.cancel(&q, &ctx()).expect("cancel own order");
    assert_eq!(e.open_order_count(), 0);
    assert!(matches!(
        e.submit(&maker("rb2601", "SHFE", 3500.0, 1), &ctx()),
        SubmitOutcome::Rejected {
            error_id: ERR_DUPLICATE_ORDER,
            ..
        }
    ));
    // a fully-filled FAK's ref is just as taken
    let mut t = fak_taker("T1", 1);
    t.limit_price = 3500.0;
    let mut m2 = maker("rb2601", "SHFE", 3500.0, 1);
    m2.order_ref = order_ref13("M2");
    accepted(e.submit(&m2, &ctx()));
    accepted(e.submit(&t, &ctx()));
    assert_eq!(e.open_order_count(), 0);
    assert!(matches!(
        e.submit(&t, &ctx()),
        SubmitOutcome::Rejected {
            error_id: ERR_DUPLICATE_ORDER,
            ..
        }
    ));
    // another session may reuse the same ref
    let mut other = maker("rb2601", "SHFE", 3500.0, 1);
    other.session_id = 2;
    accepted(e.submit(&other, &ctx()));
}

// 终态单的 25/26 区分也只对本账户成立：A 撤 B 已撤的单得到 25（找不到），
// B 自己再撤才是 26（状态不符）。
#[test]
fn terminal_cancel_split_is_per_investor() {
    let mut e = MatchingEngine::new(Catalog::bundled());
    let mut b = maker("rb2601", "SHFE", 3500.0, 1);
    b.investor_id = *b"INV0002\0\0\0\0\0\0";
    accepted(e.submit(&b, &ctx()));
    let own = CancelQuery {
        front_id: 1,
        session_id: 1,
        order_ref: "M1".into(),
        investor_id: "INV0002".into(),
        ..Default::default()
    };
    e.cancel(&own, &ctx()).expect("B cancels its own order");
    assert_eq!(
        cancel_err(&mut e, &own),
        ERR_ORDER_STATUS_UNSUITABLE,
        "owner: already cancelled"
    );
    let foreign = CancelQuery {
        investor_id: "INV0001".into(),
        ..own.clone()
    };
    assert_eq!(
        cancel_err(&mut e, &foreign),
        ERR_ORDER_NOT_FOUND,
        "another account: not found"
    );
}

/// Error id of a cancel that must be rejected.
fn cancel_err(e: &mut MatchingEngine, q: &CancelQuery) -> i32 {
    match e.cancel(q, &ctx()) {
        Err((id, _)) => id,
        Ok(_) => panic!("cancel must be rejected: {q:?}"),
    }
}

// 日结后昨日的报单键空间作废：同会话复用昨日 ref 可以报单（不是 22），
// 撤昨日的终态单是 25（不是 26）——两边口径一致。
#[test]
fn advance_trading_day_resets_the_session_ref_space() {
    let mut e = MatchingEngine::new(Catalog::bundled());
    accepted(e.submit(&maker("rb2601", "SHFE", 3500.0, 1), &ctx()));
    let q = CancelQuery {
        front_id: 1,
        session_id: 1,
        order_ref: "M1".into(),
        investor_id: "INV0001".into(),
        ..Default::default()
    };
    e.cancel(&q, &ctx()).expect("cancel own order");
    assert_eq!(cancel_err(&mut e, &q), ERR_ORDER_STATUS_UNSUITABLE);

    e.advance_trading_day();
    assert_eq!(cancel_err(&mut e, &q), ERR_ORDER_NOT_FOUND);
    accepted(e.submit(&maker("rb2601", "SHFE", 3500.0, 1), &ctx()));
    assert_eq!(e.open_order_count(), 1, "yesterday's ref is free again");
}

// refdata `is_trading = 0` / 非期货 product_class → 17 合约不能交易。
#[test]
fn halted_or_non_future_instruments_are_rejected_with_17() {
    let mut cat = Catalog::new();
    let mut halted = crate::Instrument::new("rb2601", "SHFE");
    halted.is_trading = false;
    cat.insert(halted);
    let mut opt = crate::Instrument::new("rb2601C3500", "SHFE");
    opt.product_class = b'2';
    cat.insert(opt);
    let e = MatchingEngine::new(cat);
    let err = e.check(&maker("rb2601", "SHFE", 3500.0, 1)).unwrap_err();
    assert_eq!(err.0, ERR_INSTRUMENT_NOT_TRADING);
    let err = e
        .check(&maker("rb2601C3500", "SHFE", 100.0, 1))
        .unwrap_err();
    assert_eq!(err.0, ERR_INSTRUMENT_NOT_TRADING);
}

/// FAK 全成：官方无形状可依，退回一般前态+新态+Trade（§8.9 场景 2）。
#[test]
fn fak_full_fill_falls_back_to_general_trio() {
    let mut e = MatchingEngine::new(Catalog::bundled());
    let rest = e.submit(&maker("rb2601", "SHFE", 3500.0, 5), &ctx());
    assert!(matches!(rest, SubmitOutcome::Accepted { .. }));
    let mut taker = fak_taker("T1", 5);
    taker.limit_price = 3500.0;
    let SubmitOutcome::Accepted { events } = e.submit(&taker, &ctx()) else {
        panic!("FAK must be accepted");
    };
    let taker_only: Vec<EngineEvent> = events
        .into_iter()
        .filter(|ev| match ev {
            EngineEvent::Order(f) => cstr(&f.OrderRef) == *"T1",
            EngineEvent::Trade { field, .. } => cstr(&field.OrderRef) == *"T1",
        })
        .collect();
    assert_eq!(
        statuses(&taker_only),
        vec![(b'a', false), (b'a', false), (b'0', false), (b'T', true)],
        "§8.9 场景 2（立即全成）: 'a' → 'a' → '0' + Trade, no cancel row",
    );
}

fn resting_buy(price: f64) -> OrderIntent {
    OrderIntent {
        direction: Direction::Buy,
        ..maker("rb2601", "SHFE", price, 1)
    }
}

fn trade_prices(ev: &[EngineEvent]) -> Vec<f64> {
    ev.iter()
        .filter_map(|e| match e {
            EngineEvent::Trade { field, .. } => Some(field.Price),
            _ => None,
        })
        .collect()
}

fn depth_tick(bid: f64, ask: f64) -> ctpbuddy_market::Tick {
    let mut t = ctpbuddy_market::Tick::default();
    t.instrument_id = "rb2601".into();
    t.exchange_id = "SHFE".into();
    t.trading_day = "20260915".into();
    t.last_price = bid;
    t.bid_prices[0] = bid;
    t.bid_volumes[0] = 10;
    t.ask_prices[0] = ask;
    t.ask_volumes[0] = 10;
    t
}

// 挂单被 tick 吃到：默认按档位价（价格改善），maker 模式按自身限价。
#[test]
fn resting_fill_price_default_vs_maker_at_limit() {
    for (maker_mode, expect) in [(false, 3500.0), (true, 3501.0)] {
        let mut e = MatchingEngine::new(Catalog::bundled());
        e.set_maker_fill_at_limit(maker_mode);
        accepted(e.submit(&resting_buy(3501.0), &ctx()));
        let ev = e.on_tick(&depth_tick(3499.0, 3500.0));
        assert_eq!(trade_prices(&ev), vec![expect], "maker_mode={maker_mode}");
    }
}

// 无深度（只有最新价）的行情源：限价挂单仅在最新价穿越限价时成交。
#[test]
fn depthless_tick_fills_limit_order_only_when_last_price_crosses() {
    let mut e = MatchingEngine::new(Catalog::bundled());
    accepted(e.submit(&resting_buy(3490.0), &ctx()));
    let mut t = ctpbuddy_market::Tick::default();
    t.instrument_id = "rb2601".into();
    t.exchange_id = "SHFE".into();
    t.trading_day = "20260915".into();
    t.last_price = 3500.0;
    assert!(trade_prices(&e.on_tick(&t)).is_empty(), "last above limit");
    assert_eq!(e.open_order_count(), 1);
    t.last_price = 3490.0;
    assert_eq!(trade_prices(&e.on_tick(&t)), vec![3490.0]);
    assert_eq!(e.open_order_count(), 0);
}
