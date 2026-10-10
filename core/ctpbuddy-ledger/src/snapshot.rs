//! Ledger snapshot / restore (crash recovery).
//!
//! A snapshot carries what survives a restart: cash accounts, positions with
//! their per-lot details, the per-product activity counters and the initial
//! funds setting. Working orders do not survive (the matching engine is not
//! snapshotted), so reservations are dropped on restore: `frozen_*` fields are
//! zeroed and no `FrozenEst`/`FrozenPos` entries are recreated. Money is stored
//! as exact integer units; prices as shortest round-trip `f64` text, so
//! `from_snapshot(to_snapshot(x))` reproduces the ledger bit-for-bit.

use std::collections::HashMap;

use serde_json::{json, Map, Value};

use crate::{Account, AccountKey, Ledger, Money, Position, PositionDetail, PositionSide};

/// v2: positions carry `fee_open_pool` (CFFEX 平今费时间序池, 知识库 §6.4) —
/// it decides later commissions, so the roundtrip must be exact. Version 1 is
/// still accepted by the reader; the recovery wrapper supplies the trading day
/// so the pool can be reconstructed from today's non-bootstrap details.
pub const SNAPSHOT_VERSION: u64 = 2;
const LEGACY_SNAPSHOT_VERSION: u64 = 1;

fn m(x: Money) -> Value {
    json!(x.units())
}

fn side_str(s: PositionSide) -> &'static str {
    match s {
        PositionSide::Long => "long",
        PositionSide::Short => "short",
    }
}

fn detail_json(d: &PositionDetail) -> Value {
    json!({
        "open_date": d.open_date, "trade_id": d.trade_id, "open_price": d.open_price,
        "bootstrap": d.bootstrap, "volume": d.volume, "margin": m(d.margin),
        "margin_price": d.margin_price, "open_volume": d.open_volume,
        "last_settlement_price": d.last_settlement_price,
        "close_profit": m(d.close_profit), "close_profit_trade": m(d.close_profit_trade),
        "commission": m(d.commission), "close_volume": d.close_volume,
        "close_amount": m(d.close_amount),
    })
}

fn get<'a>(o: &'a Value, k: &str) -> Result<&'a Value, String> {
    o.get(k)
        .ok_or_else(|| format!("snapshot: missing field '{k}'"))
}
fn s(o: &Value, k: &str) -> Result<String, String> {
    get(o, k)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| format!("snapshot: '{k}' not a string"))
}
fn f(o: &Value, k: &str) -> Result<f64, String> {
    get(o, k)?
        .as_f64()
        .ok_or_else(|| format!("snapshot: '{k}' not a number"))
}
fn i(o: &Value, k: &str) -> Result<i32, String> {
    get(o, k)?
        .as_i64()
        .and_then(|v| i32::try_from(v).ok())
        .ok_or_else(|| format!("snapshot: '{k}' not an i32"))
}
fn mo(o: &Value, k: &str) -> Result<Money, String> {
    get(o, k)?
        .as_i64()
        .map(Money::from_units)
        .ok_or_else(|| format!("snapshot: '{k}' not an integer"))
}
fn arr<'a>(o: &'a Value, k: &str) -> Result<&'a Vec<Value>, String> {
    get(o, k)?
        .as_array()
        .ok_or_else(|| format!("snapshot: '{k}' not an array"))
}

impl Ledger {
    /// Deterministic snapshot (entries sorted by key).
    pub fn to_snapshot(&self) -> Value {
        let mut accounts: Vec<&Account> = self.accounts.values().collect();
        accounts
            .sort_by(|a, b| (&a.broker_id, &a.investor_id).cmp(&(&b.broker_id, &b.investor_id)));
        let accounts: Vec<Value> = accounts
            .into_iter()
            .map(|a| {
                json!({
                    "broker_id": a.broker_id, "investor_id": a.investor_id,
                    "pre_balance": m(a.pre_balance), "deposit": m(a.deposit),
                    "withdraw": m(a.withdraw), "balance": m(a.balance),
                    "position_profit": m(a.position_profit), "close_profit": m(a.close_profit),
                    "commission": m(a.commission), "used_margin": m(a.used_margin),
                    "currency_id": a.currency_id,
                })
            })
            .collect();

        let mut keys: Vec<_> = self.positions.keys().collect();
        keys.sort_by(|a, b| {
            (&a.0.broker_id, &a.0.investor_id, &a.1, a.2).cmp(&(
                &b.0.broker_id,
                &b.0.investor_id,
                &b.1,
                b.2,
            ))
        });
        let positions: Vec<Value> = keys
            .into_iter()
            .map(|k| {
                let p = &self.positions[k];
                json!({
                    "broker_id": k.0.broker_id, "investor_id": k.0.investor_id,
                    "instrument_id": p.instrument_id, "side": side_str(p.side),
                    "today_position": p.today_position, "yd_position": p.yd_position,
                    "yd_initial": p.yd_initial, "fee_open_pool": p.fee_open_pool,
                    "open_amount": m(p.open_amount),
                    "open_volume": p.open_volume, "position_cost": m(p.position_cost),
                    "open_cost": m(p.open_cost), "margin": m(p.margin),
                    "commission": m(p.commission), "close_profit": m(p.close_profit),
                    "close_profit_trade": m(p.close_profit_trade),
                    "position_profit": m(p.position_profit),
                    "pre_settlement_price": p.pre_settlement_price,
                    "settlement_price": p.settlement_price,
                    "details": p.details.iter().map(detail_json).collect::<Vec<_>>(),
                })
            })
            .collect();

        let mut act: Vec<_> = self.group_activity.iter().collect();
        act.sort_by(|a, b| {
            (&a.0 .0.broker_id, &a.0 .0.investor_id, &a.0 .1, &a.0 .2).cmp(&(
                &b.0 .0.broker_id,
                &b.0 .0.investor_id,
                &b.0 .1,
                &b.0 .2,
            ))
        });
        let activity: Vec<Value> = act
            .into_iter()
            .map(|(k, v)| {
                json!({
                    "broker_id": k.0.broker_id, "investor_id": k.0.investor_id,
                    "exchange_id": k.1, "product_id": k.2,
                    "commission": m(v.0), "close_profit": m(v.1),
                })
            })
            .collect();

        let mut o = Map::new();
        o.insert("version".into(), json!(SNAPSHOT_VERSION));
        o.insert("initial_funds".into(), m(self.initial_funds));
        o.insert("accounts".into(), Value::Array(accounts));
        o.insert("positions".into(), Value::Array(positions));
        o.insert("group_activity".into(), Value::Array(activity));
        Value::Object(o)
    }

    /// Rebuild a ledger from [`Ledger::to_snapshot`] output. Working-order
    /// reservations are intentionally not restored (see module docs).
    pub fn from_snapshot(v: &Value) -> Result<Ledger, String> {
        Self::from_snapshot_with_trading_day(v, None)
    }

    /// Rebuild a ledger while migrating a v1 snapshot when the current trading
    /// day is known. v1 had no CFFEX fee pool; its remaining pool is exactly the
    /// volume of non-bootstrap details opened on that trading day. v2 stores the
    /// pool directly and does not need this reconstruction.
    pub fn from_snapshot_with_trading_day(
        v: &Value,
        trading_day: Option<&str>,
    ) -> Result<Ledger, String> {
        let version = get(v, "version")?.as_u64().ok_or("snapshot: bad version")?;
        if version != SNAPSHOT_VERSION && version != LEGACY_SNAPSHOT_VERSION {
            return Err("snapshot: unsupported version".into());
        }
        let mut ledger = Ledger::new(0.0);
        ledger.initial_funds = mo(v, "initial_funds")?;
        for a in arr(v, "accounts")? {
            let acct = Account {
                broker_id: s(a, "broker_id")?,
                investor_id: s(a, "investor_id")?,
                pre_balance: mo(a, "pre_balance")?,
                deposit: mo(a, "deposit")?,
                withdraw: mo(a, "withdraw")?,
                balance: mo(a, "balance")?,
                position_profit: mo(a, "position_profit")?,
                close_profit: mo(a, "close_profit")?,
                commission: mo(a, "commission")?,
                used_margin: mo(a, "used_margin")?,
                frozen_margin: Money::ZERO,
                frozen_commission: Money::ZERO,
                currency_id: s(a, "currency_id")?,
            };
            let key = AccountKey::new(&acct.broker_id, &acct.investor_id);
            if ledger.accounts.insert(key, acct).is_some() {
                return Err("snapshot: duplicate account".into());
            }
        }
        for p in arr(v, "positions")? {
            let key = AccountKey::new(&s(p, "broker_id")?, &s(p, "investor_id")?);
            if !ledger.accounts.contains_key(&key) {
                return Err("snapshot: position without account".into());
            }
            let side = match s(p, "side")?.as_str() {
                "long" => PositionSide::Long,
                "short" => PositionSide::Short,
                other => return Err(format!("snapshot: bad side '{other}'")),
            };
            let mut details = Vec::new();
            for d in arr(p, "details")? {
                details.push(PositionDetail {
                    open_date: s(d, "open_date")?,
                    trade_id: s(d, "trade_id")?,
                    open_price: f(d, "open_price")?,
                    bootstrap: get(d, "bootstrap")?
                        .as_bool()
                        .ok_or("snapshot: bad bootstrap")?,
                    volume: i(d, "volume")?,
                    margin: mo(d, "margin")?,
                    margin_price: f(d, "margin_price")?,
                    open_volume: i(d, "open_volume")?,
                    last_settlement_price: f(d, "last_settlement_price")?,
                    close_profit: mo(d, "close_profit")?,
                    close_profit_trade: mo(d, "close_profit_trade")?,
                    commission: mo(d, "commission")?,
                    close_volume: i(d, "close_volume")?,
                    close_amount: mo(d, "close_amount")?,
                });
            }
            let fee_open_pool = if version == SNAPSHOT_VERSION {
                i(p, "fee_open_pool")?
            } else if let Some(day) = trading_day {
                details
                    .iter()
                    .filter(|d| !d.bootstrap && d.open_date == day)
                    .try_fold(0i32, |pool, d| {
                        pool.checked_add(d.volume)
                            .ok_or_else(|| "snapshot: fee pool overflow".to_string())
                    })?
            } else {
                0
            };
            let instrument_id = s(p, "instrument_id")?;
            let pos = Position {
                instrument_id: instrument_id.clone(),
                side,
                today_position: i(p, "today_position")?,
                yd_position: i(p, "yd_position")?,
                yd_initial: i(p, "yd_initial")?,
                fee_open_pool,
                open_amount: mo(p, "open_amount")?,
                open_volume: i(p, "open_volume")?,
                position_cost: mo(p, "position_cost")?,
                open_cost: mo(p, "open_cost")?,
                margin: mo(p, "margin")?,
                frozen_today: 0,
                frozen_yd: 0,
                commission: mo(p, "commission")?,
                close_profit: mo(p, "close_profit")?,
                close_profit_trade: mo(p, "close_profit_trade")?,
                position_profit: mo(p, "position_profit")?,
                pre_settlement_price: f(p, "pre_settlement_price")?,
                settlement_price: f(p, "settlement_price")?,
                details,
            };
            if ledger
                .positions
                .insert((key, instrument_id, side), pos)
                .is_some()
            {
                return Err("snapshot: duplicate position".into());
            }
        }
        let mut activity: HashMap<(AccountKey, String, String), (Money, Money)> = HashMap::new();
        for g in arr(v, "group_activity")? {
            let key = (
                AccountKey::new(&s(g, "broker_id")?, &s(g, "investor_id")?),
                s(g, "exchange_id")?,
                s(g, "product_id")?,
            );
            activity.insert(key, (mo(g, "commission")?, mo(g, "close_profit")?));
        }
        ledger.group_activity = activity;
        Ok(ledger)
    }
}
