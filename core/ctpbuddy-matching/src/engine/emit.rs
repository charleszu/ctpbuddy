//! 回报生成：FAK 三所回报形状、成交/状态事件、通知序号与成交号分配。

use ctpbuddy_wire::generated::{cstr, set_cstr, CThostFtdcTradeField};

use super::*;

impl MatchingEngine {
    /// 官方《报单回调规则》场景 8/9/10：FAK 部成部撤的三所回报形状。
    ///
    /// 场景 8（上期所 / 能源中心 / 中金所）——撤单状态回报**先于**成交回报：
    /// ```text
    /// ReqOrderInsert
    ///   OnRtnOrder（未知单）
    ///   [交易所撤单状态回报] OnRtnOrder（已撤单，VolumeTraded 此时已有值）
    ///   [交易所成交回报]     OnRtnOrder（已撤单） → OnRtnTrade
    /// ```
    /// 没有 '3'（IOC 不进簿）、没有 '1'；每笔成交只推**一行**报单回报——此时
    /// 状态已是终态 '5'，前一状态与新态相同，CTP 不重复推（大商所 §3「不重复
    /// 推送前一状态」规则的一般化）。
    ///
    /// 场景 9（大商所 / 广期所）——进簿确认先到，每笔成交只推**一行**合成的
    /// '1'（同样不重复前态），最后一行 '5'：
    /// ```text
    ///   OnRtnOrder（未知单） → OnRtnOrder（未成交）
    ///   → [OnRtnOrder（部分成交） → OnRtnTrade] × N
    ///   → OnRtnOrder（已撤单）
    /// ```
    ///
    /// 场景 10（郑商所）——进簿确认先到，成交回报走**一般**的前态+新态：
    /// ```text
    ///   OnRtnOrder（未知单） → OnRtnOrder（未成交）
    ///   → [OnRtnOrder（前态） → OnRtnOrder（部分成交） → OnRtnTrade] × N
    ///   → OnRtnOrder（已撤单）
    /// ```
    ///
    /// 三所收尾的都是**交易所主动撤单**（FAK 剩余量被交易所撤掉，不是客户端
    /// ReqOrderAction），所以终态行只推一行、不带前态重复——这与 §2 场景 3/5
    /// 客户端主动撤单的前态+新态形状刻意不同。
    pub(super) fn emit_fak_reports(
        &mut self,
        rec: &mut OrderRecord,
        fills: &[FakFill],
        layout: IocLayout,
        events: &mut Vec<EngineEvent>,
        ctx: &ClockCtx,
    ) {
        // 撤余量是交易所接受后的终态，不是交易所拒单；CancelFirst 在此首次发布。
        rec.order_sys_id = rec.internal_sys_id;
        match layout {
            IocLayout::CancelFirst => {
                // 场景 8: cancel row first, and its VolumeTraded already
                // carries everything that traded.
                rec.status = b'5';
                let seq = self.next_notify_seq();
                rec.notify_seq = seq;
                events.push(EngineEvent::Order(build_order_field(rec, ctx, seq)));
                for (_, after, price, volume, trade_id) in fills {
                    let mut row = after.clone();
                    row.status = b'5';
                    let seq = self.next_notify_seq();
                    row.notify_seq = seq;
                    events.push(EngineEvent::Order(build_order_field(&row, ctx, seq)));
                    let trade = self.trade_event(&row, *price, *volume, trade_id, seq, ctx);
                    events.push(trade);
                }
            }
            IocLayout::TradeDriven | IocLayout::StatusDriven => {
                // 场景 9 / 10: 进簿确认 '3' 已由 submit (2b) 推过，这里逐笔推
                // 成交状态，最后一行 '5'。
                for (before, after, price, volume, trade_id) in fills {
                    let trade = if layout == IocLayout::TradeDriven {
                        // 场景 9: one synthesized 部分成交 row, no 前态 repeat
                        let seq = self.next_notify_seq();
                        let mut row = before.clone();
                        row.status = b'1';
                        row.volume_traded = after.volume_traded;
                        row.volume_total = after.volume_total;
                        row.order_sys_id = after.order_sys_id;
                        row.notify_seq = seq;
                        events.push(EngineEvent::Order(build_order_field(&row, ctx, seq)));
                        self.trade_event(&row, *price, *volume, trade_id, seq, ctx)
                    } else {
                        // 场景 10: the general 前态+新态 pair
                        let seq_prev = self.next_notify_seq();
                        let mut prev = before.clone();
                        prev.notify_seq = seq_prev;
                        events.push(EngineEvent::Order(build_order_field(&prev, ctx, seq_prev)));
                        let seq_new = self.next_notify_seq();
                        let mut row = after.clone();
                        row.status = b'1';
                        row.notify_seq = seq_new;
                        events.push(EngineEvent::Order(build_order_field(&row, ctx, seq_new)));
                        self.trade_event(&row, *price, *volume, trade_id, seq_new, ctx)
                    };
                    events.push(trade);
                }
                rec.status = b'5';
                let seq = self.next_notify_seq();
                rec.notify_seq = seq;
                events.push(EngineEvent::Order(build_order_field(rec, ctx, seq)));
            }
        }
    }

    /// A FAK that filled completely never gets a cancel report. 官方场景
    /// 8/9/10 只描述「部分成交部分撤单」，全成无官方形状可依，退回一般
    /// 前态+新态+Trade 规则（§8.9 场景 2：'a' → 'a' → '0' + Trade），大商所
    /// 沿用 §3「不重复推送前一状态」的例外。
    pub(super) fn emit_fak_full(
        &mut self,
        rec: &mut OrderRecord,
        fills: &[FakFill],
        layout: IocLayout,
        events: &mut Vec<EngineEvent>,
        ctx: &ClockCtx,
    ) {
        for (before, after, price, volume, trade_id) in fills {
            if layout != IocLayout::TradeDriven {
                let seq_prev = self.next_notify_seq();
                let mut prev = before.clone();
                prev.notify_seq = seq_prev;
                events.push(EngineEvent::Order(build_order_field(&prev, ctx, seq_prev)));
            }
            let seq_new = self.next_notify_seq();
            let mut row = after.clone();
            row.notify_seq = seq_new;
            events.push(EngineEvent::Order(build_order_field(&row, ctx, seq_new)));
            let trade = self.trade_event(&row, *price, *volume, trade_id, seq_new, ctx);
            events.push(trade);
        }
        if let Some((_, after, ..)) = fills.last() {
            *rec = after.clone();
        }
    }

    /// Build the OnRtnTrade event for one fill without pushing anything.
    /// `seq` is the notification sequence stamped on BrokerOrderSeq/SequenceNo.
    pub(super) fn trade_event(
        &mut self,
        rec: &OrderRecord,
        price: f64,
        volume: i32,
        trade_id: &str,
        seq: i32,
        ctx: &ClockCtx,
    ) -> EngineEvent {
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
            trade_id: to_fixed(trade_id),
            order_key: format!(
                "{}/{}/{}",
                rec.front_id,
                rec.session_id,
                cstr(&rec.order_ref)
            ),
        };
        EngineEvent::Trade { field: tf, fill }
    }

    /// Apply one fill and emit the full §8.9 event trio for this order:
    /// 前态 OnRtnOrder → 新态 OnRtnOrder → OnRtnTrade.
    ///
    /// `trade_id` is allocated by the caller: a book match passes the same id
    /// to both sides (one exchange trade, two reports), a market fill mints
    /// its own for the single taker report.
    ///
    /// 大商所/广期所特例 (notes/01 B3, landed M2-4): on a full fill DCE
    /// returns only the trade and CTP **self-completes** the all-traded
    /// order report **without repeating the previous state** — so the 前态
    /// push is skipped exactly when this fill completes the order (`'0'`).
    /// GFEX belongs to the DCE group everywhere else (`ioc_layout`, the
    /// 进簿确认 in `submit` 2b), so it follows this rule too. Partial fills
    /// keep the general 前态+新态 rule; the '3' confirmation DCE always
    /// returns (even for an immediately-filled order) is handled in
    /// `submit` (2b).
    pub(super) fn emit_fill(
        &mut self,
        rec: &mut OrderRecord,
        price: f64,
        volume: i32,
        trade_id: &str,
        events: &mut Vec<EngineEvent>,
        ctx: &ClockCtx,
    ) {
        // DCE/GFEX self-completion: this fill takes the order to '0' and the
        // DCE group never repeats the 前态 for its self-completed all-traded
        // report (same group as `ioc_layout`'s TradeDriven).
        let dce_self_complete = crate::exchange_rules::rules_for(&rec.exchange_id)
            .skip_prev_state_on_self_complete
            && rec.volume_total == volume;
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
        rec.order_sys_id = rec.internal_sys_id;
        let seq_new = self.next_notify_seq();
        rec.notify_seq = seq_new;
        events.push(EngineEvent::Order(build_order_field(rec, ctx, seq_new)));
        let trade = self.trade_event(rec, price, volume, trade_id, seq_new, ctx);
        events.push(trade);
    }

    /// Push a single order notification with a new status (no 前态 duplicate).
    pub(super) fn push_status(
        &mut self,
        rec: &mut OrderRecord,
        status: u8,
        events: &mut Vec<EngineEvent>,
        ctx: &ClockCtx,
    ) {
        // 单行确认或 transition 的新态才越过接受边界，前态保持原外部状态。
        rec.order_sys_id = rec.internal_sys_id;
        rec.status = status;
        let seq = self.next_notify_seq();
        rec.notify_seq = seq;
        events.push(EngineEvent::Order(build_order_field(rec, ctx, seq)));
    }

    /// Push a state transition per §8.9: 前态 then 新态.
    pub(super) fn push_transition(
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

    pub(super) fn next_notify_seq(&mut self) -> i32 {
        let s = self.next_notify;
        self.next_notify += 1;
        s
    }

    /// Mint the next exchange trade id. One id per exchange trade: the caller
    /// hands the same id to both sides of a book match.
    pub(super) fn next_trade_id(&mut self) -> String {
        let s = format!("{:010}", self.next_trade);
        self.next_trade += 1;
        s
    }
}
