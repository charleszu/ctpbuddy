use super::*;
use ctpbuddy_matching::{
    to_fixed, CommissionRate, Direction, Instrument, MarginRate, OffsetFlag, TradingParams,
};

fn fixture(enabled: bool) -> Catalog {
    // 明确虚构的测试合约和费率，不代表生产默认值。
    let mut catalog = Catalog::new();
    for (id, product) in [
        ("TESTA01", "TESTA"),
        ("TESTA02", "TESTA"),
        ("TESTB01", "TESTB"),
    ] {
        let mut i = Instrument::new(id, "TEST");
        i.product_id = product.into();
        i.max_margin_side_algorithm = if enabled { b'1' } else { b'0' };
        catalog.insert(i);
        catalog.insert_margin_rate(MarginRate {
            instrument_id: id.into(),
            long_margin_ratio_by_volume: 100.0,
            short_margin_ratio_by_volume: 150.0,
            ..Default::default()
        });
    }
    let mut params = TradingParams::default();
    params.margin_price_type = b'4';
    catalog.set_trading_params(params);
    catalog
}

fn fill(id: &str, direction: Direction, offset: OffsetFlag, volume: i32) -> Fill {
    Fill {
        broker_id: to_fixed("TEST"),
        investor_id: to_fixed("alice"),
        user_id: to_fixed("alice"),
        instrument_id: id.into(),
        exchange_id: "TEST".into(),
        direction,
        offset,
        hedge_flag: b'1',
        price: 10.0,
        volume,
        volume_total_original: volume,
        order_sys_id: to_fixed("1"),
        order_ref: to_fixed("1"),
        trade_id: to_fixed("1"),
        order_key: "1".into(),
    }
}

#[test]
fn explicit_settlement_carries_today_to_yesterday_and_rolls_account() {
    let catalog = fixture(true);
    let mut ledger = Ledger::new(10_000.0);
    ledger.ensure_account("TEST", "alice");
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 1),
        &catalog,
        10.0,
        "20261003",
    );
    ledger.mark_to_market(
        &catalog,
        &HashMap::from([("TESTA01".into(), 12.0)]),
        &HashMap::new(),
        "20261003",
    );
    ledger.account_mut("TEST", "alice").unwrap().deposit = Money::from_f64(3.0);
    assert!(ledger
        .settle_trading_day(
            &catalog,
            &HashMap::from([("TESTA01".into(), 12.0)]),
            "20261003",
            "20261004"
        )
        .is_ok());
    let p = ledger
        .position("TEST", "alice", "TESTA01", PositionSide::Long)
        .unwrap();
    assert_eq!(p.today_position, 0);
    assert_eq!(p.yd_position, 1);
    assert_eq!(p.yd_initial, 1);
    assert_eq!(p.details[0].open_date, "20261003");
    let a = ledger.account("TEST", "alice").unwrap();
    assert_eq!(a.pre_balance, a.balance);
    assert_eq!(a.deposit.to_f64(), 0.0);
    assert_eq!(a.withdraw.to_f64(), 0.0);
    assert_eq!(a.close_profit.to_f64(), 0.0);
    assert_eq!(a.position_profit.to_f64(), 0.0);
    assert_eq!(a.commission.to_f64(), 0.0);
    assert_eq!(a.frozen_margin.to_f64(), 0.0);
    assert_eq!(a.frozen_commission.to_f64(), 0.0);
}

#[test]
fn settlement_is_atomic_across_accounts_and_uses_supplied_final_price() {
    let catalog = fixture(true);
    let mut ledger = Ledger::new(10_000.0);
    for name in ["alice", "bob"] {
        ledger.ensure_account("TEST", name);
    }
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 2),
        &catalog,
        10.0,
        "20261003",
    );
    let mut other = fill("TESTB01", Direction::Sell, OffsetFlag::Open, 1);
    other.investor_id = to_fixed("bob");
    ledger.on_fill(&other, &catalog, 10.0, "20261003");
    let before = ledger.account("TEST", "alice").unwrap().balance;
    assert!(ledger
        .settle_trading_day(
            &catalog,
            &HashMap::from([("TESTA01".into(), 12.0)]),
            "20261003",
            "20261004"
        )
        .is_err());
    assert_eq!(ledger.account("TEST", "alice").unwrap().balance, before);
    assert_eq!(
        ledger
            .position("TEST", "alice", "TESTA01", PositionSide::Long)
            .unwrap()
            .today_position,
        2
    );
    let prices = HashMap::from([("TESTA01".into(), 12.0), ("TESTB01".into(), 11.0)]);
    ledger
        .settle_trading_day(&catalog, &prices, "20261003", "20261004")
        .unwrap();
    let mult = catalog.get("TESTA01").unwrap().volume_multiple as f64;
    assert_eq!(
        ledger.account("TEST", "alice").unwrap().pre_balance,
        before + Money::from_f64(4.0 * mult)
    );
    for day in ["20261004", "20260230", "2026-10-05", "20261003"] {
        assert!(ledger
            .settle_trading_day(&catalog, &prices, "20261004", day)
            .is_err());
    }
    for price in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(ledger
            .settle_trading_day(
                &catalog,
                &HashMap::from([("TESTA01".into(), price)]),
                "20261004",
                "20261005"
            )
            .is_err());
    }
}

#[test]
fn invalid_settlement_is_atomic_and_requires_all_prices() {
    let catalog = fixture(true);
    let mut ledger = Ledger::new(10_000.0);
    ledger.ensure_account("TEST", "alice");
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 1),
        &catalog,
        10.0,
        "20261003",
    );
    let before = ledger.account("TEST", "alice").unwrap().clone();
    assert!(ledger
        .settle_trading_day(&catalog, &HashMap::new(), "20261003", "20261004")
        .is_err());
    assert_eq!(
        ledger.account("TEST", "alice").unwrap().pre_balance,
        before.pre_balance
    );
    assert_eq!(
        ledger
            .position("TEST", "alice", "TESTA01", PositionSide::Long)
            .unwrap()
            .today_position,
        1
    );
    assert!(ledger
        .settle_trading_day(
            &catalog,
            &HashMap::from([("TESTA01".into(), 10.0)]),
            "20261004",
            "20261004"
        )
        .is_err());
}

#[test]
fn shfe_query_projection_splits_age_buckets_without_double_counting() {
    let mut p = Position::new("rb", PositionSide::Long);
    p.add_bootstrap_detail("20261002", "YD1", 10.0, 2, Money::from_f64(40.0), 9.0, 1);
    p.details.push(PositionDetail::new(
        "20261003",
        "TD1",
        11.0,
        3,
        Money::from_f64(60.0),
        10.0,
    ));
    p.today_position = 3;
    p.open_volume = 5;
    p.open_amount = Money::from_f64(53.0);
    p.position_cost = Money::from_f64(49.0);
    p.margin = Money::from_f64(100.0);
    p.commission = Money::from_f64(10.0);
    p.frozen_today = 1;
    let rows = p.to_query_fields("B", "I", "SHFE", "20261003", 1);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].PositionDate, b'2');
    assert_eq!(rows[0].Position, 2);
    assert_eq!(rows[1].PositionDate, b'1');
    assert_eq!(rows[1].Position, 3);
    assert_eq!(rows.iter().map(|r| r.Position).sum::<i32>(), p.volume());
    assert_eq!(rows.iter().map(|r| r.OpenVolume).sum::<i32>(), 3);
    assert!((rows.iter().map(|r| r.OpenAmount).sum::<f64>() - 33.0).abs() < 1e-9);
    assert!((rows.iter().map(|r| r.UseMargin).sum::<f64>() - p.margin.to_f64()).abs() < 1e-6);
    assert_eq!(
        rows.iter().map(|r| r.ShortFrozen).sum::<i32>(),
        p.frozen_today
    );
    assert_eq!(rows[0].YdPosition, 2);
    assert_eq!(rows[1].YdPosition, 0);
}

#[test]
fn non_shfe_query_projection_keeps_one_row() {
    let mut p = Position::new("a", PositionSide::Short);
    p.yd_position = 2;
    p.today_position = 3;
    let rows = p.to_query_fields("B", "I", "DCE", "20261003", 1);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].Position, 5);
    assert_eq!(rows[0].TodayPosition, 3);
    assert_eq!(rows[0].YdPosition, 0);
}

#[test]
fn same_product_cross_contract_and_different_product_isolation() {
    let catalog = fixture(true);
    let mut ledger = Ledger::new(10000.0);
    ledger.ensure_account("TEST", "alice");
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3),
        &catalog,
        10.0,
        "20261003",
    );
    ledger.on_fill(
        &fill("TESTA02", Direction::Sell, OffsetFlag::Open, 1),
        &catalog,
        10.0,
        "20261003",
    );
    assert_eq!(
        ledger
            .account("TEST", "alice")
            .unwrap()
            .used_margin
            .to_f64(),
        300.0
    );
    ledger.on_fill(
        &fill("TESTB01", Direction::Sell, OffsetFlag::Open, 1),
        &catalog,
        10.0,
        "20261003",
    );
    assert_eq!(
        ledger
            .account("TEST", "alice")
            .unwrap()
            .used_margin
            .to_f64(),
        450.0
    );
    let rows = ledger.product_group_margin("TEST", "alice", &catalog, "20261003");
    assert_eq!(rows.iter().map(|r| r.UseMargin).sum::<f64>(), 450.0);
}

#[test]
fn closing_larger_side_switches_margin_side() {
    let catalog = fixture(true);
    let mut ledger = Ledger::new(10000.0);
    ledger.ensure_account("TEST", "alice");
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3),
        &catalog,
        10.0,
        "20261003",
    );
    ledger.on_fill(
        &fill("TESTA02", Direction::Sell, OffsetFlag::Open, 1),
        &catalog,
        10.0,
        "20261003",
    );
    ledger.on_fill(
        &fill("TESTA01", Direction::Sell, OffsetFlag::Close, 2),
        &catalog,
        10.0,
        "20261003",
    );
    assert_eq!(
        ledger
            .account("TEST", "alice")
            .unwrap()
            .used_margin
            .to_f64(),
        150.0
    );
}

#[test]
fn pending_orders_partial_fill_and_cancel_share_incremental_risk() {
    let catalog = fixture(true);
    let mut ledger = Ledger::new(350.0);
    ledger.ensure_account("TEST", "alice");
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3),
        &catalog,
        10.0,
        "20261003",
    );
    ledger
        .freeze(
            "pending",
            "TEST",
            "alice",
            "TESTA02",
            PositionSide::Short,
            300.0,
            0.0,
            &catalog,
        )
        .unwrap();
    assert_eq!(
        ledger
            .account("TEST", "alice")
            .unwrap()
            .frozen_margin
            .to_f64(),
        0.0
    );
    assert_eq!(
        ledger.freeze(
            "extra",
            "TEST",
            "alice",
            "TESTA02",
            PositionSide::Short,
            150.0,
            0.0,
            &catalog
        ),
        Err(ERR_FUNDS)
    );
    let mut f = fill("TESTA02", Direction::Sell, OffsetFlag::Open, 1);
    f.order_key = "pending".into();
    f.volume_total_original = 2;
    ledger.on_fill(&f, &catalog, 10.0, "20261003");
    assert_eq!(
        ledger
            .account("TEST", "alice")
            .unwrap()
            .used_margin
            .to_f64(),
        300.0
    );
    assert_eq!(
        ledger
            .account("TEST", "alice")
            .unwrap()
            .frozen_margin
            .to_f64(),
        0.0
    );
    ledger.unfreeze_order("pending", &catalog);
    assert_eq!(
        ledger
            .account("TEST", "alice")
            .unwrap()
            .frozen_margin
            .to_f64(),
        0.0
    );
    ledger.on_fill(
        &fill("TESTA01", Direction::Sell, OffsetFlag::Close, 2),
        &catalog,
        10.0,
        "20261003",
    );
    assert_eq!(
        ledger
            .account("TEST", "alice")
            .unwrap()
            .used_margin
            .to_f64(),
        150.0
    );
}

#[test]
fn broker_investor_exchange_and_mixed_rules_do_not_offset() {
    let mut catalog = fixture(true);
    let mut mixed = catalog.get("TESTA02").unwrap().clone();
    mixed.max_margin_side_algorithm = b'0';
    catalog.insert(mixed);
    let mut other = catalog.get("TESTA01").unwrap().clone();
    other.instrument_id = "OTHER".into();
    other.exchange_id = "OTHER".into();
    catalog.insert(other);
    catalog.insert_margin_rate(MarginRate {
        instrument_id: "OTHER".into(),
        short_margin_ratio_by_volume: 150.0,
        ..Default::default()
    });
    let mut ledger = Ledger::new(10000.0);
    ledger.ensure_account("TEST", "alice");
    ledger.ensure_account("OTHER", "alice");
    ledger.ensure_account("TEST", "bob");
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3),
        &catalog,
        10.0,
        "20261003",
    );
    ledger.on_fill(
        &fill("TESTA02", Direction::Sell, OffsetFlag::Open, 1),
        &catalog,
        10.0,
        "20261003",
    );
    ledger.on_fill(
        &fill("OTHER", Direction::Sell, OffsetFlag::Open, 1),
        &catalog,
        10.0,
        "20261003",
    );
    let mut f = fill("TESTA01", Direction::Sell, OffsetFlag::Open, 1);
    f.broker_id = to_fixed("OTHER");
    ledger.on_fill(&f, &catalog, 10.0, "20261003");
    f.broker_id = to_fixed("TEST");
    f.investor_id = to_fixed("bob");
    ledger.on_fill(&f, &catalog, 10.0, "20261003");
    assert_eq!(
        ledger
            .account("TEST", "alice")
            .unwrap()
            .used_margin
            .to_f64(),
        600.0
    );
    assert_eq!(
        ledger
            .account("OTHER", "alice")
            .unwrap()
            .used_margin
            .to_f64(),
        150.0
    );
    assert_eq!(
        ledger.account("TEST", "bob").unwrap().used_margin.to_f64(),
        150.0
    );
}

#[test]
fn mark_to_market_does_not_reprice_booked_margin_after_market_move() {
    let mut catalog = fixture(true);
    catalog.insert_margin_rate(MarginRate {
        instrument_id: "TESTA01".into(),
        long_margin_ratio_by_money: 0.1,
        ..Default::default()
    });
    let mut ledger = Ledger::new(10000.0);
    ledger.ensure_account("TEST", "alice");
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3),
        &catalog,
        10.0,
        "20261003",
    );
    let before = ledger.account("TEST", "alice").unwrap().used_margin;
    let prices = HashMap::from([("TESTA01".into(), 20.0)]);
    let previous = HashMap::from([("TESTA01".into(), 10.0)]);
    ledger.mark_to_market(&catalog, &prices, &previous, "20261003");
    assert_eq!(ledger.account("TEST", "alice").unwrap().used_margin, before);
    ledger.on_fill(
        &fill("TESTA01", Direction::Sell, OffsetFlag::Close, 1),
        &catalog,
        10.0,
        "20261003",
    );
    let after_partial_close = ledger.account("TEST", "alice").unwrap().used_margin;
    assert!((after_partial_close.to_f64() - before.to_f64() * 2.0 / 3.0).abs() < 1e-6);
    ledger.mark_to_market(
        &catalog,
        &HashMap::from([("TESTA01".into(), 30.0)]),
        &previous,
        "20261003",
    );
    assert_eq!(
        ledger.account("TEST", "alice").unwrap().used_margin,
        after_partial_close
    );
    ledger.on_fill(
        &fill("TESTA01", Direction::Sell, OffsetFlag::Close, 1),
        &catalog,
        10.0,
        "20261003",
    );
    let rows = ledger.product_group_margin("TEST", "alice", &catalog, "20261003");
    assert!((rows.iter().map(|r| r.UseMargin).sum::<f64>() - before.to_f64() / 3.0).abs() < 1e-6);
    ledger.on_fill(
        &fill("TESTA01", Direction::Sell, OffsetFlag::Close, 1),
        &catalog,
        10.0,
        "20261003",
    );
    let rows = ledger.product_group_margin("TEST", "alice", &catalog, "20261003");
    assert_eq!(rows.iter().map(|r| r.UseMargin).sum::<f64>(), 0.0);
}

// MarginPriceType == '2'：今仓保证金随最新价变化，昨仓明细不动，平仓按比例释放。
#[test]
fn last_price_margin_type_reprices_only_today_details() {
    let mut catalog = fixture(false);
    let mut params = TradingParams::default();
    params.margin_price_type = b'2';
    catalog.set_trading_params(params);
    catalog.insert_margin_rate(MarginRate {
        instrument_id: "TESTA01".into(),
        long_margin_ratio_by_money: 0.1,
        ..Default::default()
    });
    let mut ledger = Ledger::new(10_000.0);
    ledger.ensure_account("TEST", "alice");
    ledger
        .position_mut_or_create("TEST", "alice", "TESTA01", PositionSide::Long)
        .add_bootstrap_detail("20261002", "YD1", 10.0, 1, Money::from_f64(1.0), 10.0, 1);
    ledger.refresh(&catalog);
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 2),
        &catalog,
        10.0,
        "20261003",
    );
    let used = |l: &Ledger| l.account("TEST", "alice").unwrap().used_margin.to_f64();
    // 昨仓 1 手 1.0 + 今仓 2 手按开仓时的最新价 10 → 2.0
    assert!((used(&ledger) - 3.0).abs() < 1e-6);
    ledger.mark_to_market(
        &catalog,
        &HashMap::from([("TESTA01".into(), 20.0)]),
        &HashMap::new(),
        "20261003",
    );
    // 今仓 2 手按 20 → 4.0；昨仓仍为 1.0
    assert!((used(&ledger) - 5.0).abs() < 1e-6);
    // 重复盯市幂等
    ledger.mark_to_market(
        &catalog,
        &HashMap::from([("TESTA01".into(), 20.0)]),
        &HashMap::new(),
        "20261003",
    );
    assert!((used(&ledger) - 5.0).abs() < 1e-6);
    // 平 1 手今仓（成交价 10 不影响已计保证金，按比例释放 2.0）
    let mut close = fill("TESTA01", Direction::Sell, OffsetFlag::Close, 1);
    close.order_key = "c".into();
    ledger.on_fill(&close, &catalog, 10.0, "20261003");
    ledger.mark_to_market(
        &catalog,
        &HashMap::from([("TESTA01".into(), 20.0)]),
        &HashMap::new(),
        "20261003",
    );
    // 先开先平：平的是昨仓还是今仓取决于账本规则，总量与重估口径一致即可
    let p = ledger
        .position("TEST", "alice", "TESTA01", PositionSide::Long)
        .unwrap();
    let expect: f64 = p
        .details
        .iter()
        .map(|d| {
            if d.is_today("20261003") {
                0.1 * 20.0 * d.volume as f64
            } else {
                d.margin
                    .ratio(d.volume as i64, d.open_volume as i64)
                    .to_f64()
            }
        })
        .sum();
    assert!((used(&ledger) - expect).abs() < 1e-6);
    // 行情回落后再次重估
    ledger.mark_to_market(
        &catalog,
        &HashMap::from([("TESTA01".into(), 5.0)]),
        &HashMap::new(),
        "20261003",
    );
    let today_vol: i32 = ledger
        .position("TEST", "alice", "TESTA01", PositionSide::Long)
        .unwrap()
        .details
        .iter()
        .filter(|d| d.is_today("20261003"))
        .map(|d| d.volume)
        .sum();
    assert!(used(&ledger) <= expect + 1e-6);
    assert!(today_vol >= 1);
}

#[test]
fn disabled_algorithm_keeps_sum() {
    let catalog = fixture(false);
    let mut ledger = Ledger::new(10000.0);
    ledger.ensure_account("TEST", "alice");
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3),
        &catalog,
        10.0,
        "20261003",
    );
    ledger.on_fill(
        &fill("TESTA02", Direction::Sell, OffsetFlag::Open, 1),
        &catalog,
        10.0,
        "20261003",
    );
    assert_eq!(
        ledger
            .account("TEST", "alice")
            .unwrap()
            .used_margin
            .to_f64(),
        450.0
    );
}

/// 1 手昨仓（启动初仓）+ 2 手今仓的上期所多头持仓。
fn shfe_long(exchange: &str) -> (Catalog, Ledger) {
    let mut catalog = fixture(false);
    let mut inst = Instrument::new("rb2601", exchange);
    inst.product_id = "rb".into();
    catalog.insert(inst);
    let mut ledger = Ledger::new(1_000_000.0);
    ledger.ensure_account("TEST", "alice");
    ledger
        .position_mut_or_create("TEST", "alice", "rb2601", PositionSide::Long)
        .add_bootstrap_detail("20261002", "YD1", 10.0, 1, Money::from_f64(0.0), 10.0, 1);
    let mut open = fill("rb2601", Direction::Buy, OffsetFlag::Open, 2);
    open.exchange_id = exchange.into();
    ledger.on_fill(&open, &catalog, 10.0, "20261003");
    (catalog, ledger)
}

fn close_fill(exchange: &str, key: &str, offset: OffsetFlag, volume: i32, original: i32) -> Fill {
    let mut f = fill("rb2601", Direction::Sell, offset, volume);
    f.exchange_id = exchange.into();
    f.order_key = key.into();
    f.volume_total_original = original;
    f
}

#[test]
fn shfe_close_means_close_yesterday() {
    let (_, mut ledger) = shfe_long("SHFE");
    // 上期所「平仓」= 平昨：只有 1 手昨仓可平，今仓不能被 Close 动用
    assert_eq!(
        ledger.freeze_close_position(
            "c2",
            "TEST",
            "alice",
            "rb2601",
            "SHFE",
            PositionSide::Long,
            OffsetFlag::Close,
            2
        ),
        Err(ERR_NO_CLOSE_YD_LEDGER)
    );
    assert!(ledger
        .freeze_close_position(
            "c1",
            "TEST",
            "alice",
            "rb2601",
            "SHFE",
            PositionSide::Long,
            OffsetFlag::Close,
            1
        )
        .is_ok());
    assert_eq!(
        ledger.freeze_close_position(
            "c3",
            "TEST",
            "alice",
            "rb2601",
            "SHFE",
            PositionSide::Long,
            OffsetFlag::Close,
            1
        ),
        Err(ERR_NO_CLOSE_YD_LEDGER)
    );
    // 其他交易所的 Close 仍按先开先平跨越今昨
    let (_, mut dce) = shfe_long("DCE");
    assert!(dce
        .freeze_close_position(
            "c2",
            "TEST",
            "alice",
            "rb2601",
            "DCE",
            PositionSide::Long,
            OffsetFlag::Close,
            3
        )
        .is_ok());
}

#[test]
fn partial_fill_then_cancel_keeps_other_reservations() {
    let (catalog, mut ledger) = shfe_long("SHFE");
    // A、B 各预留 1 手今仓（今仓共 2 手，已全部占满）
    for key in ["A", "B"] {
        ledger
            .freeze_close_position(
                key,
                "TEST",
                "alice",
                "rb2601",
                "SHFE",
                PositionSide::Long,
                OffsetFlag::CloseToday,
                1,
            )
            .unwrap();
    }
    // A 成交后再收到 '5'（普通 GFD 撤单路径）
    ledger.on_fill(
        &close_fill("SHFE", "A", OffsetFlag::CloseToday, 1, 1),
        &catalog,
        10.0,
        "20261003",
    );
    ledger.unfreeze_order("A", &catalog);
    let p = ledger
        .position("TEST", "alice", "rb2601", PositionSide::Long)
        .unwrap();
    assert_eq!(p.today_position, 1);
    assert_eq!(p.frozen_today, 1, "B 的预留不能被 A 的撤单冲掉");
    assert_eq!(
        ledger.freeze_close_position(
            "C",
            "TEST",
            "alice",
            "rb2601",
            "SHFE",
            PositionSide::Long,
            OffsetFlag::CloseToday,
            1
        ),
        Err(ERR_NO_CLOSE_TODAY_LEDGER),
        "仍在途的 B 已占住最后 1 手今仓"
    );
}

#[test]
fn cancel_first_fak_does_not_release_twice() {
    let (catalog, mut ledger) = shfe_long("SHFE");
    ledger
        .freeze_close_position(
            "FAK",
            "TEST",
            "alice",
            "rb2601",
            "SHFE",
            PositionSide::Long,
            OffsetFlag::CloseToday,
            2,
        )
        .unwrap();
    // 场景 8：'5' 先到（全额释放剩余预留），随后才是 1 手成交
    ledger.unfreeze_order("FAK", &catalog);
    ledger
        .freeze_close_position(
            "OTHER",
            "TEST",
            "alice",
            "rb2601",
            "SHFE",
            PositionSide::Long,
            OffsetFlag::CloseToday,
            1,
        )
        .unwrap();
    ledger.on_fill(
        &close_fill("SHFE", "FAK", OffsetFlag::CloseToday, 1, 2),
        &catalog,
        10.0,
        "20261003",
    );
    let p = ledger
        .position("TEST", "alice", "rb2601", PositionSide::Long)
        .unwrap();
    assert_eq!(p.today_position, 1);
    assert_eq!(p.frozen_today, 1, "OTHER 的预留不能被迟到的成交再扣一次");
}

#[test]
fn open_cost_excludes_commission_and_close_profit_by_trade_is_per_lot() {
    let mut catalog = fixture(false);
    let mut inst = Instrument::new("rb2601", "DCE");
    inst.product_id = "rb".into();
    inst.volume_multiple = 10;
    catalog.insert(inst);
    catalog.insert_commission_rate(CommissionRate {
        instrument_id: "rb2601".into(),
        open_ratio_by_volume: 2.0,
        close_ratio_by_volume: 3.0,
        close_today_ratio_by_volume: 3.0,
        ..Default::default()
    });
    let mut ledger = Ledger::new(1_000_000.0);
    ledger.ensure_account("TEST", "alice");
    // 1 手昨仓：开仓价 8，昨结算 10
    ledger
        .position_mut_or_create("TEST", "alice", "rb2601", PositionSide::Long)
        .add_bootstrap_detail("20261002", "YD1", 8.0, 1, Money::from_f64(0.0), 10.0, 10);
    let mut open = fill("rb2601", Direction::Buy, OffsetFlag::Open, 2);
    open.exchange_id = "DCE".into();
    open.price = 12.0;
    ledger.on_fill(&open, &catalog, 10.0, "20261003");
    let p = ledger
        .position("TEST", "alice", "rb2601", PositionSide::Long)
        .unwrap();
    assert_eq!(p.commission.to_f64(), 4.0);
    // 开仓成本 = Σ开仓价×乘数×手数，不含手续费
    assert_eq!(p.open_cost.to_f64(), 8.0 * 10.0 + 12.0 * 2.0 * 10.0);
    assert_eq!(p.position_cost.to_f64(), 10.0 * 10.0 + 12.0 * 2.0 * 10.0);
    let f = p.to_field("TEST", "alice", "DCE", "20261003", 10);
    assert_eq!(f.OpenCost, 320.0);
    assert_eq!(f.PositionCost, 340.0);

    // 平 2 手（先开先平：1 手昨仓 + 1 手今仓）@ 14
    ledger
        .freeze_close_position(
            "c1",
            "TEST",
            "alice",
            "rb2601",
            "DCE",
            PositionSide::Long,
            OffsetFlag::Close,
            2,
        )
        .unwrap();
    let mut close = fill("rb2601", Direction::Sell, OffsetFlag::Close, 2);
    close.exchange_id = "DCE".into();
    close.order_key = "c1".into();
    close.price = 14.0;
    ledger.on_fill(&close, &catalog, 10.0, "20261003");
    let p = ledger
        .position("TEST", "alice", "rb2601", PositionSide::Long)
        .unwrap();
    assert_eq!(p.commission.to_f64(), 10.0);
    assert_eq!(p.open_cost.to_f64(), 12.0 * 10.0, "剩余 1 手今仓的开仓成本");
    // 盯市：昨仓 (14-10)*10 + 今仓 (14-12)*10 = 60；逐笔：(14-8)*10 + (14-12)*10 = 80
    assert_eq!(p.close_profit.to_f64(), 60.0);
    assert_eq!(p.close_profit_trade.to_f64(), 80.0);
    let f = p.to_field("TEST", "alice", "DCE", "20261003", 10);
    assert_eq!(f.OpenCost, 120.0);
    assert_eq!(f.CloseProfitByDate, 60.0);
    assert_eq!(f.CloseProfitByTrade, 80.0);
    let detail_trade: Money = p.details.iter().map(|d| d.close_profit_trade).sum();
    assert_eq!(f.CloseProfitByTrade, detail_trade.to_f64());
}

#[test]
fn snapshot_roundtrip_is_exact_and_drops_working_order_reservations() {
    let catalog = fixture(true);
    let mut ledger = Ledger::new(10_000.0);
    ledger.ensure_account("TEST", "alice");
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 3),
        &catalog,
        10.0,
        "20261003",
    );
    ledger.on_fill(
        &fill("TESTA01", Direction::Sell, OffsetFlag::Close, 1),
        &catalog,
        10.0,
        "20261003",
    );
    ledger.mark_to_market(
        &catalog,
        &HashMap::from([("TESTA01".into(), 12.3)]),
        &HashMap::new(),
        "20261003",
    );
    let snap = ledger.to_snapshot();
    // survives a text round trip, as it does on disk
    let text = snap.to_string();
    let back = Ledger::from_snapshot(&serde_json::from_str(&text).unwrap()).unwrap();
    assert_eq!(back.to_snapshot(), snap);
    let (a, b) = (
        ledger.account("TEST", "alice").unwrap(),
        back.account("TEST", "alice").unwrap(),
    );
    assert_eq!(a.balance, b.balance);
    assert_eq!(a.used_margin, b.used_margin);
    assert_eq!(a.position_profit, b.position_profit);
    assert_eq!(
        ledger.positions_of("TEST", "alice").len(),
        back.positions_of("TEST", "alice").len()
    );

    let mut bad = snap.clone();
    bad["version"] = serde_json::json!(99);
    assert!(Ledger::from_snapshot(&bad).is_err());
    let mut bad = snap;
    bad["accounts"] = serde_json::json!([]);
    assert!(
        Ledger::from_snapshot(&bad).is_err(),
        "positions need an account"
    );
}

#[test]
fn restore_after_pending_order_frees_funds_and_leaves_no_residue() {
    let catalog = fixture(true);
    let mut ledger = Ledger::new(10_000.0);
    ledger.ensure_account("TEST", "alice");
    let before = ledger.account("TEST", "alice").unwrap().available();
    ledger
        .freeze(
            "pending",
            "TEST",
            "alice",
            "TESTA01",
            PositionSide::Long,
            1_000.0,
            5.0,
            &catalog,
        )
        .unwrap();
    let a = ledger.account("TEST", "alice").unwrap();
    assert_eq!(a.available(), before - Money::from_f64(1_005.0));
    // crash-time snapshot while the order is still resting
    let text = ledger.to_snapshot().to_string();
    let mut back = Ledger::from_snapshot(&serde_json::from_str(&text).unwrap()).unwrap();
    let b = back.account("TEST", "alice").unwrap();
    assert_eq!(b.frozen_margin, Money::ZERO);
    assert_eq!(b.frozen_commission, Money::ZERO);
    assert_eq!(b.available(), before, "available must not stay reduced");
    // a late unfreeze of the lost order must be a harmless no-op
    back.unfreeze_order("pending", &catalog);
    assert_eq!(back.account("TEST", "alice").unwrap().available(), before);
}

#[test]
fn settlement_reprices_margin_at_settlement_price() {
    let mut catalog = fixture(true);
    catalog.insert_margin_rate(MarginRate {
        instrument_id: "TESTA01".into(),
        long_margin_ratio_by_money: 0.1,
        ..Default::default()
    });
    let mut ledger = Ledger::new(100_000.0);
    ledger.ensure_account("TEST", "alice");
    ledger.on_fill(
        &fill("TESTA01", Direction::Buy, OffsetFlag::Open, 2),
        &catalog,
        10.0,
        "20261003",
    );
    let mult = catalog.get("TESTA01").unwrap().volume_multiple as f64;
    let booked = ledger
        .account("TEST", "alice")
        .unwrap()
        .used_margin
        .to_f64();
    assert!((booked - 10.0 * 2.0 * mult * 0.1).abs() < 1e-6);
    ledger
        .settle_trading_day(
            &catalog,
            &HashMap::from([("TESTA01".into(), 12.0)]),
            "20261003",
            "20261004",
        )
        .unwrap();
    let want = 12.0 * 2.0 * mult * 0.1;
    let p = ledger
        .position("TEST", "alice", "TESTA01", PositionSide::Long)
        .unwrap();
    assert!((p.margin.to_f64() - want).abs() < 1e-6);
    assert_eq!(p.details[0].margin_price, 12.0);
    let a = ledger.account("TEST", "alice").unwrap();
    assert!((a.used_margin.to_f64() - want).abs() < 1e-6);
}

/// 中金所平今费时间序池（知识库 §6.4/§10.4 #10；IM2410/20240924 生产实锤
/// 81/81）：持仓消耗走明细先开先平，手续费判定走「先平当日新开仓」的开仓
/// 池，两轴互不参考。本测试复刻实锤序列的三段分叉——昨仓被平收平今费、
/// 池内平今仓收平今费、池尽后平今仓收平昨费——并与非池交易所（DCE，费用
/// 跟随被平明细年龄）同序列逐段对照。当日开仓全部当日平光时两轴总量恰好
/// 相同（4200），分叉只在逐段增量里可见，故必须按段断言。
#[test]
fn close_fee_axis_is_trade_time_pool_on_cffex_and_detail_age_elsewhere() {
    let legs = [
        ("CFFEX", [2000.0, 2000.0, 200.0]),
        ("DCE", [200.0, 2000.0, 2000.0]),
    ];
    for (exchange, wants) in legs {
        let mut catalog = fixture(false);
        let mut inst = Instrument::new("IF2606", exchange);
        inst.product_id = "if".into();
        inst.volume_multiple = 200;
        catalog.insert(inst);
        // 明确虚构的测试费率：平今 1% / 平昨 0.1%（按金额），只为断言可读。
        catalog.insert_commission_rate(CommissionRate {
            exchange_id: exchange.into(),
            instrument_id: "IF2606".into(),
            close_ratio_by_money: 0.001,
            close_today_ratio_by_money: 0.01,
            ..Default::default()
        });
        let mut ledger = Ledger::new(1_000_000.0);
        ledger.ensure_account("TEST", "alice");
        ledger
            .position_mut_or_create("TEST", "alice", "IF2606", PositionSide::Long)
            .add_bootstrap_detail("20261008", "YD1", 1000.0, 1, Money::ZERO, 1000.0, 200);
        let comm = |l: &Ledger| l.account("TEST", "alice").unwrap().commission.to_f64();
        // 手数 1：turnover = 1000 × 200，平今 = ×1% = 2000，平昨 = ×0.1% = 200。
        let (open_ref, close_ref) = (["T1", "T3"], ["T2", "T4", "T5"]);
        let mut refs = open_ref.iter().chain(close_ref.iter());

        // ① 昨仓 1 手 + 当日买开 1 手（CFFEX 池 = 1）。
        let mut f = fill("IF2606", Direction::Buy, OffsetFlag::Open, 1);
        f.exchange_id = exchange.into();
        f.price = 1000.0;
        f.trade_id = to_fixed(refs.next().unwrap());
        f.order_key = "o1".into();
        ledger.on_fill(&f, &catalog, 1000.0, "20261009");
        if exchange == "CFFEX" {
            assert_eq!(
                ledger
                    .position("TEST", "alice", "IF2606", PositionSide::Long)
                    .unwrap()
                    .fee_open_pool,
                1
            );
        }

        // ② 平 1 手：明细轴先开先平平掉**昨仓**；费用轴 CFFEX 池有余 →
        //    平今费（昨仓被平收平今费，IM2410 第 3 笔实锤），DCE 跟年龄收
        //    平昨费。
        let c0 = comm(&ledger);
        let mut f = fill("IF2606", Direction::Sell, OffsetFlag::Close, 1);
        f.exchange_id = exchange.into();
        f.price = 1000.0;
        f.trade_id = to_fixed(refs.next().unwrap());
        f.order_key = "o2".into();
        ledger.on_fill(&f, &catalog, 1000.0, "20261009");
        assert!(
            (comm(&ledger) - c0 - wants[0]).abs() < 1e-6,
            "{exchange} leg ②"
        );

        // ③ 再买开 1 手（CFFEX 池回到 1）。
        let mut f = fill("IF2606", Direction::Buy, OffsetFlag::Open, 1);
        f.exchange_id = exchange.into();
        f.price = 1000.0;
        f.trade_id = to_fixed(refs.next().unwrap());
        f.order_key = "o3".into();
        ledger.on_fill(&f, &catalog, 1000.0, "20261009");

        // ④ 平 1 手：明细轴吃先开的今仓；CFFEX 池有余仍收平今费，DCE 按年龄
        //    收平今费——此段两轴一致。
        let c0 = comm(&ledger);
        let mut f = fill("IF2606", Direction::Sell, OffsetFlag::Close, 1);
        f.exchange_id = exchange.into();
        f.price = 1000.0;
        f.trade_id = to_fixed(refs.next().unwrap());
        f.order_key = "o4".into();
        ledger.on_fill(&f, &catalog, 1000.0, "20261009");
        assert!(
            (comm(&ledger) - c0 - wants[1]).abs() < 1e-6,
            "{exchange} leg ④"
        );

        // ⑤ 平最后 1 手（今仓）：CFFEX 池已耗尽 → **平今仓收平昨费**
        //    （IM2410 序号 1095839 实锤），DCE 按年龄收平今费。
        let c0 = comm(&ledger);
        let mut f = fill("IF2606", Direction::Sell, OffsetFlag::Close, 1);
        f.exchange_id = exchange.into();
        f.price = 1000.0;
        f.trade_id = to_fixed(refs.next().unwrap());
        f.order_key = "o5".into();
        ledger.on_fill(&f, &catalog, 1000.0, "20261009");
        assert!(
            (comm(&ledger) - c0 - wants[2]).abs() < 1e-6,
            "{exchange} leg ⑤"
        );
    }
}

/// 平今费时间序池随交易日清零：「当日新开仓」的当日按交易日界定，隔日
/// 开仓池出局、与 today_position 归零同点。
#[test]
fn settlement_clears_the_fee_pool_with_the_trading_day() {
    let mut catalog = fixture(false);
    let mut inst = Instrument::new("IF2606", "CFFEX");
    inst.product_id = "if".into();
    inst.volume_multiple = 200;
    catalog.insert(inst);
    let mut ledger = Ledger::new(1_000_000.0);
    ledger.ensure_account("TEST", "alice");
    let mut f = fill("IF2606", Direction::Buy, OffsetFlag::Open, 2);
    f.exchange_id = "CFFEX".into();
    ledger.on_fill(&f, &catalog, 1000.0, "20261009");
    assert_eq!(
        ledger
            .position("TEST", "alice", "IF2606", PositionSide::Long)
            .unwrap()
            .fee_open_pool,
        2
    );
    ledger
        .settle_trading_day(
            &catalog,
            &HashMap::from([("IF2606".into(), 1000.0)]),
            "20261009",
            "20261010",
        )
        .unwrap();
    let p = ledger
        .position("TEST", "alice", "IF2606", PositionSide::Long)
        .unwrap();
    assert_eq!(p.today_position, 0);
    assert_eq!(p.yd_position, 2);
    assert_eq!(p.fee_open_pool, 0);
}
