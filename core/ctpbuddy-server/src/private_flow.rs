//! Private-flow bookkeeping and SubscribePrivateTopic replay (RESTART / RESUME / QUICK).

use serde_json::{json, Value as SerdeValue};

use super::*;

impl World {
    pub(crate) fn private_flow_snapshot(&self) -> SerdeValue {
        let bytes = |data: &[u8]| -> SerdeValue {
            SerdeValue::Array(data.iter().map(|byte| json!(*byte)).collect())
        };
        json!({
            "orders": self.orders_today.iter().map(|field| bytes(&struct_to_bytes(field))).collect::<Vec<_>>(),
            "trades": self.trades_today.iter().map(|field| bytes(&struct_to_bytes(field))).collect::<Vec<_>>(),
            "sequence": self.private_seq.iter().map(|(trade, index)| json!({"trade": trade, "index": index})).collect::<Vec<_>>(),
            "cursors": self.private_cursor.iter().map(|((broker, investor), index)| json!({
                "broker": broker, "investor": investor, "index": index
            })).collect::<Vec<_>>(),
        })
    }

    pub(crate) fn restore_private_flow(&mut self, value: &SerdeValue) -> Result<(), String> {
        let object = value.as_object().ok_or("private_flow 必须是对象")?;
        let decode = |name: &str| -> Result<Vec<Vec<u8>>, String> {
            object
                .get(name)
                .and_then(SerdeValue::as_array)
                .ok_or_else(|| format!("private_flow.{name} 必须是数组"))?
                .iter()
                .map(|item| {
                    item.as_array()
                        .ok_or_else(|| format!("private_flow.{name} 条目必须是字节数组"))?
                        .iter()
                        .map(|byte| {
                            let n = byte
                                .as_u64()
                                .ok_or_else(|| format!("private_flow.{name} 字节必须是整数"))?;
                            u8::try_from(n).map_err(|_| format!("private_flow.{name} 字节超出范围"))
                        })
                        .collect()
                })
                .collect()
        };
        let orders = decode("orders")?;
        let trades = decode("trades")?;
        let mut decoded_orders = Vec::with_capacity(orders.len());
        for bytes in orders {
            decoded_orders.push(
                struct_from_bytes::<CThostFtdcOrderField>(&bytes)
                    .ok_or("private_flow order bytes 长度错误")?,
            );
        }
        let mut decoded_trades = Vec::with_capacity(trades.len());
        for bytes in trades {
            decoded_trades.push(
                struct_from_bytes::<CThostFtdcTradeField>(&bytes)
                    .ok_or("private_flow trade bytes 长度错误")?,
            );
        }
        let sequence = object
            .get("sequence")
            .and_then(SerdeValue::as_array)
            .ok_or("private_flow.sequence 必须是数组")?
            .iter()
            .map(|item| {
                let item = item
                    .as_object()
                    .ok_or("private_flow.sequence 条目必须是对象")?;
                let trade = item
                    .get("trade")
                    .and_then(SerdeValue::as_bool)
                    .ok_or("private_flow.sequence.trade 必须是布尔值")?;
                let index = item
                    .get("index")
                    .and_then(SerdeValue::as_u64)
                    .ok_or("private_flow.sequence.index 必须是整数")?
                    as usize;
                if trade {
                    if index >= decoded_trades.len() {
                        return Err("private_flow.sequence 指向不存在的 trade".into());
                    }
                } else if index >= decoded_orders.len() {
                    return Err("private_flow.sequence 指向不存在的 order".into());
                }
                Ok((trade, index))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut cursors = HashMap::new();
        for item in object
            .get("cursors")
            .and_then(SerdeValue::as_array)
            .ok_or("private_flow.cursors 必须是数组")?
        {
            let item = item
                .as_object()
                .ok_or("private_flow.cursors 条目必须是对象")?;
            let broker = item
                .get("broker")
                .and_then(SerdeValue::as_str)
                .ok_or("private_flow.cursors.broker 必须是字符串")?;
            let investor = item
                .get("investor")
                .and_then(SerdeValue::as_str)
                .ok_or("private_flow.cursors.investor 必须是字符串")?;
            let index = item
                .get("index")
                .and_then(SerdeValue::as_u64)
                .ok_or("private_flow.cursors.index 必须是整数")? as usize;
            if index > sequence.len() {
                return Err("private_flow.cursors.index 超出 sequence".into());
            }
            cursors.insert((broker.to_string(), investor.to_string()), index);
        }
        self.orders_today = decoded_orders;
        self.trades_today = decoded_trades;
        self.private_seq = sequence;
        self.private_cursor = cursors;
        Ok(())
    }

    /// Drop the trading day's order/trade history and the replay indexes
    /// that point into it (reset and explicit settlement).
    pub(crate) fn clear_day_flow(&mut self) {
        self.orders_today.clear();
        self.trades_today.clear();
        self.private_seq.clear();
        self.private_cursor.clear();
    }

    /// Advance the investor's RESUME cursor past the event just emitted when
    /// a live session received it.
    pub(crate) fn mark_delivered(&mut self, broker: &str, investor: &str, delivered: bool) {
        if delivered {
            self.private_cursor.insert(
                (broker.to_string(), investor.to_string()),
                self.private_seq.len(),
            );
        }
    }

    /// Replay the day's private flow to a freshly logged-in session
    /// (SubscribePrivateTopic): RESTART = whole day, RESUME = what this
    /// investor missed while no session was live, QUICK = nothing.
    pub(crate) fn replay_private_flow(&mut self, conn_id: u64, broker: &str, investor: &str) {
        let mode = self.conns.get(&conn_id).map_or(2, |c| c.private_resume);
        let key = (broker.to_string(), investor.to_string());
        let start = match mode {
            0 => 0,
            1 => self.private_cursor.get(&key).copied().unwrap_or(0),
            _ => self.private_seq.len(),
        };
        let start = start.min(self.private_seq.len());
        let mut frames = Vec::new();
        for &(is_trade, idx) in &self.private_seq[start..] {
            if is_trade {
                let t = &self.trades_today[idx];
                if cstr(&t.BrokerID) == broker && cstr(&t.InvestorID) == investor {
                    frames.push(Frame::new(msgs::RTN_TRADE, 0, struct_to_bytes(t)));
                }
            } else {
                let o = &self.orders_today[idx];
                if cstr(&o.BrokerID) == broker && cstr(&o.InvestorID) == investor {
                    frames.push(Frame::new(msgs::RTN_ORDER, 0, struct_to_bytes(o)));
                }
            }
        }
        for f in frames {
            self.send_frame(conn_id, f);
        }
        self.private_cursor.insert(key, self.private_seq.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SubscribePrivateTopic: RESTART replays the whole day, RESUME only what
    /// was emitted while the investor had no live session, QUICK nothing.
    #[test]
    fn private_flow_replay_modes() {
        let mut world = World::new(Config::default()).expect("world");
        let mk_order = |inv: &str, r: &str| {
            let mut o = CThostFtdcOrderField::zeroed();
            set_cstr(&mut o.BrokerID, "9999");
            set_cstr(&mut o.InvestorID, inv);
            set_cstr(&mut o.OrderRef, r);
            o
        };
        let (tx1, rx1) = mpsc::channel::<Frame>();
        world.on_conn_opened(1, tx1, false);
        {
            let c = world.conns.get_mut(&1).unwrap();
            c.broker_id = Some("9999".into());
            c.investor_id = Some("u".into());
        }
        world.dispatch_event(EngineEvent::Order(mk_order("u", "1")));
        world.dispatch_event(EngineEvent::Order(mk_order("other", "9")));
        assert_eq!(rx1.try_iter().count(), 1);
        // session 1 drops; two events happen while nobody is connected
        world.on_conn_closed(1);
        world.dispatch_event(EngineEvent::Order(mk_order("u", "2")));
        world.dispatch_event(EngineEvent::Order(mk_order("u", "3")));
        let replay = |world: &mut World, id: u64, mode: u8| -> Vec<String> {
            let (tx, rx) = mpsc::channel::<Frame>();
            world.on_conn_opened(id, tx, false);
            let c = world.conns.get_mut(&id).unwrap();
            c.broker_id = Some("9999".into());
            c.investor_id = Some("u".into());
            c.private_resume = mode;
            world.replay_private_flow(id, "9999", "u");
            world.on_conn_closed(id);
            rx.try_iter()
                .map(|f| {
                    let o: CThostFtdcOrderField =
                        ctpbuddy_wire::struct_from_bytes(&f.payload).unwrap();
                    cstr(&o.OrderRef)
                })
                .collect()
        };
        assert!(replay(&mut world, 2, 2).is_empty());
        // QUICK still moved the cursor to the end: nothing left to resume
        assert!(replay(&mut world, 3, 1).is_empty());
        assert_eq!(replay(&mut world, 4, 0), vec!["1", "2", "3"]);
        // RESUME from a fresh cursor
        world.private_cursor.clear();
        assert_eq!(replay(&mut world, 5, 1), vec!["1", "2", "3"]);

        let snapshot = world.private_flow_snapshot();
        world.clear_day_flow();
        assert!(world.orders_today.is_empty());
        world.restore_private_flow(&snapshot).unwrap();
        assert_eq!(world.orders_today.len(), 4);
        assert_eq!(world.private_seq.len(), 4);
        assert_eq!(
            world.private_cursor.get(&("9999".into(), "u".into())),
            Some(&4)
        );
    }
}
