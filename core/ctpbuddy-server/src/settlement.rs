//! 结算报告输入、持久化与查询；正文为原始 GBK 字节。
use crate::json::{self, Value};
use crate::{PrivateNotification, World};
use ctpbuddy_ledger::Ledger;
use ctpbuddy_matching::{Direction, Fill, OffsetFlag};
use ctpbuddy_wire::generated::{cstr, set_cstr, CThostFtdcOrderField, CThostFtdcTradeField};
use ctpbuddy_wire::{msgs, struct_to_bytes, Frame};

#[derive(Clone)]
pub(crate) struct Report {
    pub broker: String,
    pub investor: String,
    pub day: String,
    pub settlement_id: i32,
    pub account: String,
    pub currency: String,
    pub content: Vec<u8>,
    pub source: String,
}

impl Report {
    pub fn parse(v: &Value, broker: &str) -> Result<Self, String> {
        let fields = v.as_obj().ok_or("报告必须为对象")?;
        if json::unknown_or_duplicate(
            fields,
            &[
                "broker",
                "investor",
                "trading_day",
                "settlement_id",
                "account_id",
                "currency_id",
                "content_bytes",
                "source",
            ],
        )
        .is_some()
        {
            return Err("报告含未知或重复字段".into());
        }
        let b = v.get_str("broker").ok_or("报告缺少 broker")?;
        let investor = v.get_str("investor").ok_or("报告缺少 investor")?;
        let day = v.get_str("trading_day").ok_or("报告缺少 trading_day")?;
        let id = v.get_num("settlement_id").ok_or("报告缺少 settlement_id")?;
        let account = v.get_str("account_id").unwrap_or_default();
        let currency = v.get_str("currency_id").unwrap_or_default();
        if b != broker
            || investor.is_empty()
            || investor.len() > 12
            || !investor.is_ascii()
            || investor.contains('\0')
            || b.len() > 10
            || !b.is_ascii()
            || b.contains('\0')
        {
            return Err("报告身份无效".into());
        }
        if !((day.len() == 8 && valid_day(&day))
            || (day.len() == 6
                && day.bytes().all(|c| c.is_ascii_digit())
                && day[4..]
                    .parse::<u32>()
                    .map(|m| (1..=12).contains(&m))
                    .unwrap_or(false)))
        {
            return Err("报告 trading_day 必须为有效 YYYYMMDD 或 YYYYMM".into());
        }
        if id < 1.0
            || id > i32::MAX as f64
            || id.fract() != 0.0
            || !id.is_finite()
            || account.len() > 12
            || currency.len() > 3
            || !account.is_ascii()
            || !currency.is_ascii()
            || account.contains('\0')
            || currency.contains('\0')
        {
            return Err("报告元数据无效".into());
        }
        let Some(Value::Arr(bytes)) = v.get("content_bytes") else {
            return Err("报告需要 content_bytes 字节数组".into());
        };
        if bytes.is_empty() || bytes.len() > 512 * 1024 {
            return Err("报告正文长度必须为 1..524288 字节".into());
        }
        let mut content = Vec::with_capacity(bytes.len());
        for byte in bytes {
            let n = byte.as_num().ok_or("正文必须为字节数组")?;
            if !(1.0..=255.0).contains(&n) || n.fract() != 0.0 {
                return Err("正文字节必须为 1..255，不能含 NUL".into());
            }
            content.push(n as u8);
        }
        Ok(Self {
            broker: b,
            investor,
            day,
            settlement_id: id as i32,
            account,
            currency,
            content,
            source: v
                .get_str("source")
                .unwrap_or_else(|| "user_supplied".into()),
        })
    }

    pub fn value(&self) -> Value {
        json::obj_sorted(vec![
            ("broker".into(), json::s(&self.broker)),
            ("investor".into(), json::s(&self.investor)),
            ("trading_day".into(), json::s(&self.day)),
            ("settlement_id".into(), json::n(self.settlement_id as f64)),
            ("account_id".into(), json::s(&self.account)),
            ("currency_id".into(), json::s(&self.currency)),
            ("source".into(), json::s(&self.source)),
            (
                "content_bytes".into(),
                Value::Arr(self.content.iter().map(|b| json::n(*b as f64)).collect()),
            ),
        ])
    }
}

impl World {
    pub(crate) fn validate_reports(&self, reports: &[Report]) -> Result<(), String> {
        let mut keys = std::collections::HashSet::new();
        for r in reports {
            if !keys.insert((&r.broker, &r.investor, &r.day))
                || self.settlement_reports.iter().any(|old| {
                    old.broker == r.broker && old.investor == r.investor && old.day == r.day
                })
            {
                return Err("结算报告已存在或输入重复，不允许覆盖".into());
            }
        }
        Ok(())
    }

    pub(crate) fn save_reports(&mut self, reports: Vec<Report>) -> Result<(), String> {
        self.validate_reports(&reports)?;
        if !self.cfg.data_dir.is_empty() && !reports.is_empty() {
            use std::io::Write;
            let dir = std::path::Path::new(&self.cfg.data_dir).join("settlement_reports");
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos();
            let path = dir.join(format!("{stamp}.json"));
            let pending = dir.join(format!("{stamp}.pending"));
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&pending)
                .map_err(|e| e.to_string())?;
            file.write_all(
                Value::Arr(reports.iter().map(Report::value).collect())
                    .to_json()
                    .as_bytes(),
            )
            .and_then(|_| file.sync_all())
            .map_err(|e| e.to_string())?;
            drop(file);
            std::fs::rename(pending, path).map_err(|e| e.to_string())?;
        }
        for r in reports {
            self.journal_record_json("settlement_report", &r.broker, &r.investor, r.value());
            self.settlement_reports.push(r);
        }
        Ok(())
    }

    pub(crate) fn minimal_reports(ledger: &Ledger, day: &str) -> Vec<Report> {
        let accounts = accounts_sorted_by_broker(ledger);
        accounts.into_iter().map(|a| {
            let content = format!("CTPBuddy simulated ledger minimal settlement report\nSource=modeled_ledger_only; not an official broker statement\nTradingDay={day}\nBrokerID={}\nInvestorID={}\nCurrencyID={}\nSettledEquity={}\nCarriedMargin={}\nUnmodeled fields omitted; amounts from explicit settle_day\n", a.broker_id, a.investor_id, a.currency_id, a.pre_balance.to_f64(), a.used_margin.to_f64()).into_bytes();
            Report { broker: a.broker_id.clone(), investor: a.investor_id.clone(), day: day.into(), settlement_id: 1, account: a.investor_id.clone(), currency: a.currency_id.clone(), content, source: "modeled_ledger_minimal".into() }
        }).collect()
    }

    // ---- settlement day orchestration (moved from admin.rs; pure move) ----

    pub(crate) fn settle_day(&mut self, v: &Value) -> Result<Value, String> {
        let next_day = match v.get_str("next_trading_day") {
            Some(day) => day.to_string(),
            None => match self
                .cfg
                .calendar_snapshot
                .as_ref()
                .and_then(|calendar| calendar.next_trading_day(&self.vt_trading_day).ok())
            {
                Some(day) => day,
                None => {
                    return Err(
                        ("settle_day 需要 next_trading_day（或配置完整离线日历）").to_string()
                    )
                }
            },
        };
        let Some(prices) = v.get("settlement_prices").and_then(Value::as_obj) else {
            return Err(("settle_day 需要 settlement_prices 对象").to_string());
        };
        if self.playback.as_ref().map(|p| !p.paused()).unwrap_or(false) {
            return Err(("日结前必须暂停 playback").to_string());
        }
        if self.engine.open_order_count() != 0 {
            return Err(("日结前不能存在活动订单").to_string());
        }
        let Some(fields) = v.as_obj() else {
            return Err(("settle_day 请求必须为对象").to_string());
        };
        if json::unknown_or_duplicate(fields, &["cmd", "settlement_prices", "next_trading_day"])
            .is_some()
        {
            return Err(("settle_day 含未知或重复字段").to_string());
        }
        let mut map = std::collections::HashMap::new();
        for (instrument, value) in prices {
            let Some(price) = value.as_num() else {
                return Err(format!("结算价 {instrument} 必须为数字"));
            };
            if !price.is_finite() || price <= 0.0 {
                return Err(format!("结算价 {instrument} 必须为有限正数"));
            }
            if map.insert(instrument.clone(), price).is_some() {
                return Err(("结算价合约重复").to_string());
            }
        }
        let current_day = self.vt_trading_day.clone();
        let mut brokers: Vec<String> = self
            .cfg
            .broker_ids()
            .into_iter()
            .map(str::to_string)
            .collect();
        brokers.sort_unstable();
        let mut staged_ledger = self.ledger.clone();
        let forced = self.stage_expired_liquidations(&map, &current_day, &mut staged_ledger)?;
        if let Err(e) =
            staged_ledger.settle_trading_day(self.engine.catalog(), &map, &current_day, &next_day)
        {
            return Err(e.to_string());
        }
        let mut staged_engine = self.engine.clone();
        staged_engine.advance_trading_day();
        let mut staged_playback = self.playback.clone();
        if let Some(pb) = staged_playback.as_mut() {
            pb.advance_trading_day(&next_day);
        }
        let accounts = crate::settlement::accounts_sorted_by_broker(&staged_ledger);
        let account_results = Value::Arr(
            accounts
                .iter()
                .map(|a| {
                    json::obj_sorted(vec![
                        ("broker".into(), json::s(&a.broker_id)),
                        ("investor".into(), json::s(&a.investor_id)),
                        ("pre_balance".into(), json::n(a.pre_balance.to_f64())),
                        ("used_margin".into(), json::n(a.used_margin.to_f64())),
                        (
                            "positions".into(),
                            Value::Arr(
                                staged_ledger
                                    .positions_of_ordered(&a.broker_id, &a.investor_id)
                                    .iter()
                                    .map(|(p, d)| {
                                        json::obj_sorted(vec![
                                            ("instrument".into(), json::s(&p.instrument_id)),
                                            (
                                                "side".into(),
                                                json::s(
                                                    if p.side == ctpbuddy_ledger::PositionSide::Long
                                                    {
                                                        "long"
                                                    } else {
                                                        "short"
                                                    },
                                                ),
                                            ),
                                            ("open_date".into(), json::s(&d.open_date)),
                                            ("trade_id".into(), json::s(&d.trade_id)),
                                            ("open_price".into(), json::n(d.open_price)),
                                            ("volume".into(), json::n(d.volume as f64)),
                                            (
                                                "last_settlement_price".into(),
                                                json::n(d.last_settlement_price),
                                            ),
                                        ])
                                    })
                                    .collect(),
                            ),
                        ),
                    ])
                })
                .collect(),
        );
        let event = json::obj_sorted(vec![
            ("from_trading_day".into(), json::s(&current_day)),
            ("next_trading_day".into(), json::s(&next_day)),
            (
                "settlement_prices".into(),
                json::obj_sorted(prices.to_vec()),
            ),
            ("accounts".into(), account_results),
            (
                "cleared_orders".into(),
                json::n(self.orders_today.len() as f64),
            ),
            (
                "cleared_trades".into(),
                json::n(self.trades_today.len() as f64),
            ),
            ("forced_liquidations".into(), json::n(forced.len() as f64)),
            (
                "cleared_confirmations".into(),
                json::n(self.settlement_confirmed.len() as f64),
            ),
            (
                "brokers".into(),
                Value::Arr(brokers.iter().map(|broker| json::s(broker)).collect()),
            ),
        ]);
        let generated = World::minimal_reports(&staged_ledger, &current_day)
            .into_iter()
            .filter(|r| {
                !self.settlement_reports.iter().any(|old| {
                    old.broker == r.broker && old.investor == r.investor && old.day == r.day
                })
            })
            .collect();
        if let Err(e) = self.save_reports(generated) {
            return Err(format!("保存结算报告失败: {e}"));
        }
        self.ledger = staged_ledger;
        self.record_forced_liquidations(&forced);
        self.engine = staged_engine;
        self.playback = staged_playback;
        let broker = self.cfg.primary_broker().to_string();
        self.journal_record_json("settlement", &broker, "", event);
        let sys_event = json::obj_sorted(vec![
            ("type".into(), json::s("settlement_complete")),
            ("from_trading_day".into(), json::s(&current_day)),
            ("trading_day".into(), json::s(&next_day)),
            ("forced_liquidations".into(), json::n(forced.len() as f64)),
            (
                "brokers".into(),
                Value::Arr(brokers.iter().map(|broker| json::s(broker)).collect()),
            ),
        ]);
        self.publish(crate::Broadcast {
            topic: "sys/settlement".into(),
            frame: Frame::new(msgs::ADMIN_RSP, 0, sys_event.to_json().into_bytes()),
        });
        self.clear_day_flow();
        self.settlement_confirmed.clear();
        self.vt_trading_day = next_day.clone();
        self.persist_ledger();
        Ok(json::obj_sorted(vec![
            ("ok".into(), json::b(true)),
            ("cmd".into(), json::s("settle_day")),
            ("trading_day".into(), json::s(&next_day)),
            ("forced_liquidations".into(), json::n(forced.len() as f64)),
            (
                "brokers".into(),
                Value::Arr(brokers.iter().map(|broker| json::s(broker)).collect()),
            ),
        ]))
    }

    /// Stage expiry closes against a cloned ledger. Missing or invalid prices
    /// are rejected before the live world is changed.
    fn stage_expired_liquidations(
        &self,
        prices: &std::collections::HashMap<String, f64>,
        trading_day: &str,
        staged: &mut ctpbuddy_ledger::Ledger,
    ) -> Result<Vec<(CThostFtdcOrderField, Fill, CThostFtdcTradeField, f64)>, String> {
        struct Candidate {
            broker: String,
            investor: String,
            instrument: String,
            exchange: String,
            direction: Direction,
            offset: OffsetFlag,
            volume: i32,
            price: f64,
            pre_settlement: f64,
        }
        let accounts: Vec<(String, String)> = self
            .ledger
            .accounts()
            .map(|a| (a.broker_id.clone(), a.investor_id.clone()))
            .collect();
        let mut candidates: Vec<Candidate> = Vec::new();
        for (broker, investor) in accounts {
            for position in self.ledger.positions_of(&broker, &investor) {
                let Some(instrument) = self.engine.catalog().get(&position.instrument_id) else {
                    continue;
                };
                if position.volume() <= 0
                    || instrument.expire_date.is_empty()
                    || instrument.expire_date.as_str() > trading_day
                {
                    continue;
                }
                let price = prices
                    .get(&position.instrument_id)
                    .copied()
                    .ok_or_else(|| {
                        format!(
                            "到期合约 {} 缺少结算价，自动强平已拒绝",
                            position.instrument_id
                        )
                    })?;
                // Positive-finiteness is enforced once over the whole price
                // map before staging starts (see settle_day).
                let direction = match position.side {
                    ctpbuddy_ledger::PositionSide::Long => Direction::Sell,
                    ctpbuddy_ledger::PositionSide::Short => Direction::Buy,
                };
                let rules = ctpbuddy_matching::rules_for(&instrument.exchange_id);
                if rules.splits_today_yesterday {
                    if position.today_position > 0 {
                        candidates.push(Candidate {
                            broker: broker.clone(),
                            investor: investor.clone(),
                            instrument: position.instrument_id.clone(),
                            exchange: instrument.exchange_id.clone(),
                            direction,
                            offset: OffsetFlag::CloseToday,
                            volume: position.today_position,
                            price,
                            pre_settlement: position.pre_settlement_price,
                        });
                    }
                    if position.yd_position > 0 {
                        candidates.push(Candidate {
                            broker: broker.clone(),
                            investor: investor.clone(),
                            instrument: position.instrument_id.clone(),
                            exchange: instrument.exchange_id.clone(),
                            direction,
                            offset: OffsetFlag::CloseYesterday,
                            volume: position.yd_position,
                            price,
                            pre_settlement: position.pre_settlement_price,
                        });
                    }
                } else {
                    candidates.push(Candidate {
                        broker: broker.clone(),
                        investor: investor.clone(),
                        instrument: position.instrument_id.clone(),
                        exchange: instrument.exchange_id.clone(),
                        direction,
                        offset: OffsetFlag::Close,
                        volume: position.volume(),
                        price,
                        pre_settlement: position.pre_settlement_price,
                    });
                }
            }
        }
        candidates.sort_by(|a, b| {
            a.broker
                .cmp(&b.broker)
                .then(a.investor.cmp(&b.investor))
                .then(a.instrument.cmp(&b.instrument))
                .then(a.offset.as_ctp().cmp(&b.offset.as_ctp()))
        });
        let mut out = Vec::with_capacity(candidates.len());
        for (index, candidate) in candidates.into_iter().enumerate() {
            let Candidate {
                broker,
                investor,
                instrument,
                exchange,
                direction,
                offset,
                volume,
                price,
                pre_settlement,
            } = candidate;
            let order_ref = format!("AUTOFC-{trading_day}-{}", index + 1);
            let order_sys_id = format!("AUTOFC-SYS-{}", index + 1);
            let trade_id = format!("AUTOFC-TRD-{}", index + 1);
            let mut fill = Fill {
                broker_id: [0; 11],
                investor_id: [0; 13],
                user_id: [0; 16],
                instrument_id: instrument.clone(),
                exchange_id: exchange.clone(),
                direction,
                offset,
                hedge_flag: b'1',
                price,
                volume,
                volume_total_original: volume,
                order_sys_id: [0; 21],
                order_ref: [0; 13],
                trade_id: [0; 21],
                order_key: format!("auto-force-close/{trading_day}/{index}"),
            };
            set_cstr(&mut fill.broker_id, &broker);
            set_cstr(&mut fill.investor_id, &investor);
            set_cstr(&mut fill.order_sys_id, &order_sys_id);
            set_cstr(&mut fill.order_ref, &order_ref);
            set_cstr(&mut fill.trade_id, &trade_id);
            staged.on_fill(&fill, self.engine.catalog(), pre_settlement, trading_day);

            let mut order = CThostFtdcOrderField::zeroed();
            set_cstr(&mut order.BrokerID, &broker);
            set_cstr(&mut order.InvestorID, &investor);
            set_cstr(&mut order.OrderRef, &order_ref);
            set_cstr(&mut order.OrderSysID, &order_sys_id);
            set_cstr(&mut order.InstrumentID, &instrument);
            set_cstr(&mut order.ExchangeID, &exchange);
            set_cstr(&mut order.TradingDay, trading_day);
            order.OrderPriceType = b'2';
            order.Direction = direction.as_ctp();
            order.CombOffsetFlag[0] = offset.as_ctp();
            order.CombHedgeFlag[0] = b'1';
            order.LimitPrice = price;
            order.VolumeTotalOriginal = volume;
            order.TimeCondition = b'3';
            order.VolumeCondition = b'1';
            // CTP's PersonDeliv reason ('7') identifies a delivery/expiry
            // close; it is distinct from a user order with reason '0'.
            order.ForceCloseReason = b'7';
            order.OrderSubmitStatus = b'3';
            order.OrderStatus = b'0';
            order.OrderType = b'0';
            order.VolumeTraded = volume;
            set_cstr(&mut order.StatusMsg, "到期合约自动强平");

            let mut trade = CThostFtdcTradeField::zeroed();
            set_cstr(&mut trade.BrokerID, &broker);
            set_cstr(&mut trade.InvestorID, &investor);
            set_cstr(&mut trade.OrderRef, &order_ref);
            set_cstr(&mut trade.OrderSysID, &order_sys_id);
            set_cstr(&mut trade.InstrumentID, &instrument);
            set_cstr(&mut trade.ExchangeID, &exchange);
            set_cstr(&mut trade.TradeID, &trade_id);
            set_cstr(&mut trade.TradingDay, trading_day);
            trade.Direction = direction.as_ctp();
            trade.OffsetFlag = offset.as_ctp();
            trade.HedgeFlag = b'1';
            trade.Price = price;
            trade.Volume = volume;
            out.push((order, fill, trade, pre_settlement));
        }
        Ok(out)
    }

    fn record_forced_liquidations(
        &mut self,
        events: &[(CThostFtdcOrderField, Fill, CThostFtdcTradeField, f64)],
    ) {
        let day = self.vt_trading_day.clone();
        for (order, fill, trade, pre_settlement) in events {
            self.private_seq.push((false, self.orders_today.len()));
            self.orders_today.push(*order);
            self.journal_record_order_update(&day, self.vt_now_ms, order);
            self.emit_private_notification(PrivateNotification {
                broker: cstr(&order.BrokerID),
                investor: cstr(&order.InvestorID),
                frame: Frame::new(msgs::RTN_ORDER, 0, struct_to_bytes(order)),
            });
            self.private_seq.push((true, self.trades_today.len()));
            self.trades_today.push(*trade);
            self.journal_record_fill(&day, self.vt_now_ms, fill, *pre_settlement);
            self.emit_private_notification(PrivateNotification {
                broker: cstr(&trade.BrokerID),
                investor: cstr(&trade.InvestorID),
                frame: Frame::new(msgs::RTN_TRADE, 0, struct_to_bytes(trade)),
            });
        }
    }
}

fn valid_day(day: &str) -> bool {
    if day.len() != 8 || !day.bytes().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let y = day[0..4].parse::<i64>().ok();
    let m = day[4..6].parse::<u32>().ok();
    let d = day[6..8].parse::<u32>().ok();
    match (y, m, d) {
        (Some(y), Some(m), Some(d)) if (1..=12).contains(&m) && (1..=31).contains(&d) => {
            let next = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
            crate::dtime::days_from_civil(next.0, next.1, 1)
                - crate::dtime::days_from_civil(y, m, 1)
                >= d as i64
        }
        _ => false,
    }
}

/// Accounts ordered by (broker, investor) — the stable admin/report order.
pub(crate) fn accounts_sorted_by_broker(ledger: &Ledger) -> Vec<&ctpbuddy_ledger::Account> {
    let mut accounts: Vec<_> = ledger.accounts().collect();
    accounts.sort_by(|a, b| {
        a.broker_id
            .cmp(&b.broker_id)
            .then(a.investor_id.cmp(&b.investor_id))
    });
    accounts
}

pub(crate) fn load_reports(dir: &str, brokers: &[&str]) -> Result<Vec<Report>, String> {
    let path = std::path::Path::new(dir).join("settlement_reports");
    if dir.is_empty() || !path.exists() {
        return Ok(Vec::new());
    }
    let mut paths = std::fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    paths.sort_by_key(|e| e.file_name());
    let mut out = Vec::new();
    let mut keys = std::collections::HashSet::new();
    for entry in paths {
        if entry.path().extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(entry.path()).map_err(|e| e.to_string())?;
        let Value::Arr(rows) = json::parse(&text)? else {
            return Err("报告存储格式错误".into());
        };
        for row in rows {
            let broker = row
                .get_str("broker")
                .ok_or_else(|| "报告存储缺少 broker".to_string())?;
            if !brokers.iter().any(|allowed| *allowed == broker) {
                return Err(format!("报告存储含不支持的 BrokerID: {broker}"));
            }
            let report = Report::parse(&row, &broker)?;
            if !keys.insert((
                report.broker.clone(),
                report.investor.clone(),
                report.day.clone(),
            )) {
                return Err("存储中报告重复".into());
            }
            out.push(report);
        }
    }
    Ok(out)
}
