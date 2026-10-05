use super::jsonl::parse_flat_object;
use super::*;

fn rb() -> Instrument {
    let mut i = Instrument::new("rb2601", "SHFE");
    i.volume_multiple = 10;
    i.price_tick = 1.0;
    i
}

fn rate(ratio: f64) -> MarginRate {
    let mut r = MarginRate::default();
    r.instrument_id = "rb2601".into();
    r.long_margin_ratio_by_money = ratio;
    r.short_margin_ratio_by_money = ratio;
    r
}

#[test]
fn margin_sums_by_money_and_by_volume() {
    let mut r = rate(0.16);
    r.long_margin_ratio_by_volume = 3.0;
    // 按手数项与按金额项**相加**，不是取 max（notes/04 C1）：
    // 每手 = 3 + 0.16 × 3500 × 10。
    let m = r.margin(Direction::Buy, 3500.0, 10, 2);
    assert!((m - 2.0 * (3.0 + 0.16 * 3500.0 * 10.0)).abs() < 1e-9, "{m}");
}

#[test]
fn margin_uses_broker_rate_not_exchange_rate() {
    let mut rd = RefData::new();
    rd.insert_instrument(rb());
    // 交易所费率故意设成两倍公司费率：若核心误用它，数字会翻倍。
    rd.insert_margin_rate(rate(0.16));
    let mut inst = rb();
    inst.long_margin_ratio = 0.32;
    rd.insert_instrument(inst);

    let m = rd.margin(
        "rb2601",
        Direction::Buy,
        MarginPrice::PreSettlement(3500.0),
        1,
    );
    assert!((m - 0.16 * 3500.0 * 10.0).abs() < 1e-9, "{m}");
}

#[test]
fn margin_falls_back_to_exchange_rate_when_no_broker_rate() {
    let mut rd = RefData::new();
    let mut inst = rb();
    inst.long_margin_ratio = 0.32;
    rd.insert_instrument(inst);
    // 未配公司费率的desk仍应得到合理数字，而不是 0 保证金。
    let m = rd.margin(
        "rb2601",
        Direction::Buy,
        MarginPrice::PreSettlement(3500.0),
        1,
    );
    assert!((m - 0.32 * 3500.0 * 10.0).abs() < 1e-9, "{m}");
}

#[test]
fn commission_sums_by_money_and_by_volume() {
    let mut c = CommissionRate::default();
    c.instrument_id = "rb2601".into();
    c.open_ratio_by_money = 0.0001;
    c.open_ratio_by_volume = 0.5;
    // ByVolume 是每手固定费，与按成交额部分相加（notes/04 D1）。
    let f = c.commission(CommissionKind::Open, 3500.0, 10, 3);
    assert!(
        (f - 3.0 * (3500.0 * 10.0 * 0.0001 + 0.5)).abs() < 1e-9,
        "{f}"
    );
}

#[test]
fn close_today_and_close_yesterday_are_priced_separately() {
    let mut c = CommissionRate::default();
    c.instrument_id = "rb2601".into();
    c.close_ratio_by_money = 0.0001;
    c.close_today_ratio_by_money = 0.0003;
    let yd = c.commission(CommissionKind::CloseYesterday, 3500.0, 10, 2);
    let td = c.commission(CommissionKind::CloseToday, 3500.0, 10, 1);
    // 吃掉 2 手昨仓 + 1 手今仓的一笔 Close 必须按两腿计价。
    assert!((yd - 2.0 * 3500.0 * 10.0 * 0.0001).abs() < 1e-9, "{yd}");
    assert!((td - 1.0 * 3500.0 * 10.0 * 0.0003).abs() < 1e-9, "{td}");
    // 统一按平昨计会显著偏低 —— 这正是 notes/04 B3 警告的偏差。
    assert!(td / 1.0 > yd / 2.0);
}

#[test]
fn yesterday_positions_always_use_pre_settlement() {
    let mut rd = RefData::new();
    rd.insert_instrument(rb());
    let mut p = TradingParams::default();
    // 让今仓跟随最新价：此时昨仓仍必须钉在昨结算。
    p.margin_price_type = MPT_SETTLEMENT;
    rd.set_params(p);

    let yd = rd.margin_price(false, 3500.0, 3600.0, 3550.0, 3555.0);
    assert!(matches!(yd, MarginPrice::PreSettlement(p) if p == 3500.0));
    let td = rd.margin_price(true, 3500.0, 3600.0, 3550.0, 3555.0);
    assert!(matches!(td, MarginPrice::Last(p) if p == 3600.0));
}

#[test]
fn missing_tables_mean_no_such_rule_not_error() {
    let mut rd = RefData::new();
    rd.insert_instrument(rb());
    // 只配手续费、不配保证金/申报费/交易参数：加载必须成功。
    let mut c = CommissionRate::default();
    c.instrument_id = "rb2601".into();
    c.open_ratio_by_money = 0.0001;
    rd.insert_commission_rate(c);

    assert!(rd.margin_rate("rb2601").is_none());
    assert!(
        rd.margin(
            "rb2601",
            Direction::Buy,
            MarginPrice::PreSettlement(3500.0),
            1
        ) >= 0.0
    );
    assert!(rd.commission("rb2601", CommissionKind::Open, 3500.0, 1) > 0.0);
    assert_eq!(
        rd.commission("rb2601", CommissionKind::Open, 3500.0, 1) / (3500.0 * 10.0 * 0.0001),
        1.0
    );
}

#[test]
fn flat_parser_handles_escapes_and_rejects_bad_literals() {
    let row =
        parse_flat_object(r#"{"instrument_name":"螺纹","is_trading":true,"x":null}"#).unwrap();
    assert_eq!(row.str("instrument_name").as_deref(), Some("螺纹"));
    assert_eq!(row.num("is_trading"), Some(1.0));
    assert!(row.str("x").is_none());
    assert!(parse_flat_object(r#"{"a":tru}"#).is_err());
    assert!(parse_flat_object(r#"{"a":"\q"}"#).is_err());
}

/// Write one `instruments.jsonl` row into a fresh temp dir and load it.
fn load_one_row(tag: &str, row: &str) -> RefData {
    let dir = std::env::temp_dir().join(format!("ctpbuddy-refdata-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("instruments.jsonl"), format!("{row}\n")).unwrap();
    let rd = RefData::load_jsonl_dir(dir.to_str().unwrap()).expect("load");
    let _ = std::fs::remove_dir_all(&dir);
    rd
}

// 文件给了的日期 / 枚举字段必须原样生效，不被代码推导覆盖；只有缺省的才补。
#[test]
fn jsonl_fields_take_precedence_over_code_derivation() {
    let rd = load_one_row(
        "dates",
        r#"{"instrument_id": "rb2610", "exchange_id": "SHFE", "delivery_year": 2026, "delivery_month": 10, "expire_date": "20261014", "create_date": "20251016", "open_date": "20251017", "start_deliv_date": "20261016", "end_deliv_date": "20261020", "position_type": "2", "position_date_type": "1", "is_trading": 1, "inst_life_phase": "1", "product_class": "1", "options_type": "0", "underlying_multiple": 1, "volume_multiple": 10, "price_tick": 1}"#,
    );
    let i = rd.instrument("rb2610").unwrap();
    assert_eq!(i.expire_date, "20261014", "文件值，不是代码推导的 20261015");
    assert_eq!(i.create_date, "20251016");
    assert_eq!(i.open_date, "20251017");
    assert_eq!(i.start_deliv_date, "20261016");
    assert_eq!(i.end_deliv_date, "20261020");
    assert_eq!((i.delivery_year, i.delivery_month), (2026, 10));
    assert_eq!(i.position_type, b'2');
    assert_eq!(i.position_date_type, b'1');
    assert_eq!(i.inst_life_phase, b'1');
    assert_eq!(i.product_class, b'1');
    assert_eq!(i.options_type, b'0');
    assert!(i.is_trading);
    assert_eq!(i.underlying_multiple, 1.0);

    // a date the file does not carry is still derived (缺省才补)
    let rd = load_one_row(
        "partial",
        r#"{"instrument_id": "rb2610", "exchange_id": "SHFE", "expire_date": "20261013"}"#,
    );
    let i = rd.instrument("rb2610").unwrap();
    assert_eq!(
        i.expire_date, "20261013",
        "不被 fill_dates_from_code 改成 15 号"
    );
    assert_eq!(i.create_date, "20261001", "缺省才推导");
    assert_eq!((i.delivery_year, i.delivery_month), (2026, 10));
}

// `is_trading: 0` / 期权 product_class 要从数据读到——引擎的 17 拒单靠这两个字段。
#[test]
fn is_trading_and_product_class_are_read_from_jsonl() {
    let rd = load_one_row(
        "halted",
        r#"{"instrument_id": "rb2610", "exchange_id": "SHFE", "is_trading": 0, "product_class": "2", "options_type": "1"}"#,
    );
    let i = rd.instrument("rb2610").unwrap();
    assert!(!i.is_trading);
    assert_eq!(i.product_class, b'2');
    assert_eq!(i.options_type, b'1');
    assert_eq!(i.to_field().IsTrading, 0);
}

#[test]
fn bundled_refdata_loads_real_contracts() {
    let dir = crate::catalog::bundled_refdata_dir();
    let rd = RefData::load_jsonl_dir(dir).expect("bundled ref data must load");
    // 随包数据是真实快照，不是占位符：rb2601 是上期所螺纹钢。
    let inst = rd.instrument("rb2601").expect("rb2601 in bundled data");
    assert_eq!(inst.exchange_id, "SHFE");
    assert_eq!(inst.volume_multiple, 10);
    // 真实到期日是 2026-01-15（上期所合约月第 15 日），文件值和推导值在这里碰巧
    // 相同；用 CZCE 的 SA506（文件 20250616，推导会是 20250615）证明读的是文件。
    let sa = rd.instrument("SA506").expect("SA506 in bundled data");
    assert_eq!(sa.expire_date, "20250616");
    assert_eq!(sa.position_date_type, b'2');
    assert!(sa.is_trading);
    assert_eq!(sa.product_class, b'1');
    assert!(rd.margin_rate("rb2601").is_some(), "bundled margin rates");
    // 刻意不随包分发手续费表——编造的手续费比没有更糟。
    assert!(rd.commission_rate("rb2601").is_none());
}
