//! Private-flow bookkeeping and SubscribePrivateTopic replay (RESTART / RESUME / QUICK).

use super::*;

impl World {
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
    }
}
