//! Atomic, bounded live read model. Order rows show the latest lifecycle state.
use crate::{
    json::{self, Value},
    World,
};
use ctpbuddy_wire::generated::cstr;
use std::collections::BTreeMap;

pub(crate) struct LiveQuery {
    broker: Option<String>,
    investor: Option<String>,
    pub limit: usize,
    pub offset: usize,
}

impl LiveQuery {
    pub fn parse(value: &Value) -> Result<Self, String> {
        let fields = value.as_obj().ok_or("live_status 请求必须为对象")?;
        if json::unknown_or_duplicate(fields, &["cmd", "broker", "investor", "limit", "offset"])
            .is_some()
        {
            return Err("live_status 含未知或重复字段".into());
        }
        let identity = |key: &str| -> Result<Option<String>, String> {
            match value.get(key) {
                None => Ok(None),
                Some(Value::Str(s)) if !s.is_empty() && s.len() <= 64 => Ok(Some(s.clone())),
                _ => Err(format!("live_status {key} 无效")),
            }
        };
        let number = |key: &str, default: usize, max: usize| -> Result<usize, String> {
            match value.get(key) {
                None => Ok(default),
                Some(Value::Num(n))
                    if n.is_finite() && n.fract() == 0.0 && *n >= 0.0 && *n <= max as f64 =>
                {
                    Ok(*n as usize)
                }
                _ => Err(format!("live_status {key} 无效")),
            }
        };
        let limit = number("limit", 100, 200)?;
        if limit == 0 {
            return Err("live_status limit 必须在 1..200".into());
        }
        Ok(Self {
            broker: identity("broker")?,
            investor: identity("investor")?,
            limit,
            offset: number("offset", 0, 1_000_000_000)?,
        })
    }

    pub fn matches(&self, broker: &str, investor: &str) -> bool {
        self.broker.as_deref().map(|v| v == broker).unwrap_or(true)
            && self
                .investor
                .as_deref()
                .map(|v| v == investor)
                .unwrap_or(true)
    }

    pub fn page(&self, rows: Vec<Value>) -> Value {
        Value::Arr(
            rows.into_iter()
                .skip(self.offset)
                .take(self.limit)
                .collect(),
        )
    }
}

impl World {
    pub(crate) fn live_orders_trades(&self, query: &LiveQuery) -> Vec<(String, Value)> {
        let mut latest = BTreeMap::new();
        for order in &self.orders_today {
            let broker = cstr(&order.BrokerID);
            let investor = cstr(&order.InvestorID);
            if query.matches(&broker, &investor) {
                latest.insert(
                    (
                        broker,
                        investor,
                        order.FrontID,
                        order.SessionID,
                        cstr(&order.OrderRef),
                    ),
                    order,
                );
            }
        }
        let orders: Vec<_> = latest
            .values()
            .skip(query.offset)
            .take(query.limit)
            .map(|o| {
                json::obj_sorted(vec![
                    ("broker".into(), json::s(&cstr(&o.BrokerID))),
                    ("investor".into(), json::s(&cstr(&o.InvestorID))),
                    ("instrument".into(), json::s(&cstr(&o.InstrumentID))),
                    ("exchange".into(), json::s(&cstr(&o.ExchangeID))),
                    ("order_ref".into(), json::s(&cstr(&o.OrderRef))),
                    ("order_sys_id".into(), json::s(&cstr(&o.OrderSysID))),
                    ("front_id".into(), json::n(o.FrontID as f64)),
                    ("session_id".into(), json::n(o.SessionID as f64)),
                    (
                        "status".into(),
                        json::s(&(o.OrderStatus as char).to_string()),
                    ),
                    (
                        "submit_status".into(),
                        json::s(&(o.OrderSubmitStatus as char).to_string()),
                    ),
                    (
                        "direction".into(),
                        json::s(&(o.Direction as char).to_string()),
                    ),
                    (
                        "offset".into(),
                        json::s(&(o.CombOffsetFlag[0] as char).to_string()),
                    ),
                    ("volume".into(), json::n(o.VolumeTotalOriginal as f64)),
                    ("volume_traded".into(), json::n(o.VolumeTraded as f64)),
                    ("volume_total".into(), json::n(o.VolumeTotal as f64)),
                    ("status_message".into(), json::s(&cstr(&o.StatusMsg))),
                    ("price".into(), json::n(o.LimitPrice)),
                    (
                        "force_close_reason".into(),
                        json::s(&(o.ForceCloseReason as char).to_string()),
                    ),
                ])
            })
            .collect();
        let trades: Vec<_> = self
            .trades_today
            .iter()
            .filter(|t| query.matches(&cstr(&t.BrokerID), &cstr(&t.InvestorID)))
            .collect();
        let trade_count = trades.len();
        let trades: Vec<_> = trades
            .into_iter()
            .skip(query.offset)
            .take(query.limit)
            .map(|t| {
                json::obj_sorted(vec![
                    ("broker".into(), json::s(&cstr(&t.BrokerID))),
                    ("investor".into(), json::s(&cstr(&t.InvestorID))),
                    ("instrument".into(), json::s(&cstr(&t.InstrumentID))),
                    ("exchange".into(), json::s(&cstr(&t.ExchangeID))),
                    ("trade_id".into(), json::s(&cstr(&t.TradeID))),
                    ("order_ref".into(), json::s(&cstr(&t.OrderRef))),
                    ("order_sys_id".into(), json::s(&cstr(&t.OrderSysID))),
                    (
                        "direction".into(),
                        json::s(&(t.Direction as char).to_string()),
                    ),
                    (
                        "offset".into(),
                        json::s(&(t.OffsetFlag as char).to_string()),
                    ),
                    ("price".into(), json::n(t.Price)),
                    ("volume".into(), json::n(t.Volume as f64)),
                    ("trading_day".into(), json::s(&cstr(&t.TradingDay))),
                    ("time".into(), json::s(&cstr(&t.TradeTime))),
                ])
            })
            .collect();
        vec![
            ("orders".into(), Value::Arr(orders)),
            ("trades".into(), Value::Arr(trades)),
            ("order_count".into(), json::n(latest.len() as f64)),
            ("trade_count".into(), json::n(trade_count as f64)),
        ]
    }
}
