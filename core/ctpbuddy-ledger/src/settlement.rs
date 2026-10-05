//! Trading-day settlement and mark-to-market (split out of `lib.rs`).

use std::collections::HashMap;

use ctpbuddy_matching::Catalog;

use crate::*;

impl Ledger {
    /// 使用调用方供给的结算价原子结算全部账户。
    /// 调用方负责暂停播放及检查活动订单；所有价格和日期校验通过后才暂存滚存。
    pub fn settle_trading_day(
        &mut self,
        catalog: &Catalog,
        prices: &HashMap<String, f64>,
        trading_day: &str,
        next_trading_day: &str,
    ) -> Result<(), String> {
        fn valid_day(s: &str) -> bool {
            if s.len() != 8 || !s.bytes().all(|b| b.is_ascii_digit()) {
                return false;
            }
            let y: u32 = s[..4].parse().unwrap();
            let m: u32 = s[4..6].parse().unwrap();
            let d: u32 = s[6..].parse().unwrap();
            let leap = y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400));
            let max = match m {
                2 if leap => 29,
                2 => 28,
                4 | 6 | 9 | 11 => 30,
                1..=12 => 31,
                _ => 0,
            };
            y >= 1900 && d > 0 && d <= max
        }
        if !valid_day(trading_day)
            || !valid_day(next_trading_day)
            || next_trading_day <= trading_day
        {
            return Err("next_trading_day 必须为严格递增的合法 YYYYMMDD".into());
        }
        for (instrument, price) in prices {
            if catalog.get(instrument).is_none() || !price.is_finite() || *price <= 0.0 {
                return Err(format!("未知合约或非法结算价: {instrument}"));
            }
        }
        if self
            .frozen
            .values()
            .any(|e| e.remaining_margin().is_positive() || e.remaining_commission().is_positive())
            || self
                .positions
                .values()
                .any(|p| p.frozen_today > 0 || p.frozen_yd > 0)
        {
            return Err("存在未释放的订单冻结".into());
        }
        for ((_, instrument, _), position) in &self.positions {
            if position.volume() <= 0 {
                continue;
            }
            let price = prices
                .get(instrument)
                .copied()
                .ok_or_else(|| format!("缺少持仓合约 {instrument} 的结算价"))?;
            if !price.is_finite() || price <= 0.0 {
                return Err(format!("合约 {instrument} 的结算价必须为正有限数"));
            }
        }
        let mut staged = self.clone();
        staged.mark_to_market(catalog, prices, &HashMap::new(), trading_day);
        let final_equity: HashMap<AccountKey, Money> = staged
            .accounts
            .values()
            .map(|a| {
                (
                    AccountKey::new(&a.broker_id, &a.investor_id),
                    a.dynamic_equity(),
                )
            })
            .collect();
        for position in staged.positions.values_mut() {
            let volume = position.volume();
            let price = prices
                .get(&position.instrument_id)
                .copied()
                .unwrap_or(position.settlement_price);
            position.yd_position = volume;
            position.yd_initial = volume;
            position.today_position = 0;
            position.frozen_today = 0;
            position.frozen_yd = 0;
            position.pre_settlement_price = price;
            position.settlement_price = price;
            let mult = catalog
                .get(&position.instrument_id)
                .map(|i| i.volume_multiple)
                .unwrap_or(1) as f64;
            position.position_cost = Money::from_f64(price * volume as f64 * mult);
            position.position_profit = Money::ZERO;
            position.close_profit = Money::ZERO;
            position.close_profit_trade = Money::ZERO;
            position.commission = Money::ZERO;
            position.details.retain(|d| d.volume > 0);
            // 日结后保证金按结算价重估（昨仓恒用昨结算价，notes/04 C2）：
            // 每个明细按剩余手数重算，持仓保证金为明细之和。
            let direction = match position.side {
                PositionSide::Long => ctpbuddy_matching::Direction::Buy,
                PositionSide::Short => ctpbuddy_matching::Direction::Sell,
            };
            for detail in &mut position.details {
                detail.margin = Money::from_f64(catalog.margin(
                    &position.instrument_id,
                    direction,
                    ctpbuddy_matching::MarginPrice::PreSettlement(price),
                    detail.volume,
                ));
                detail.margin_price = price;
                detail.open_volume = detail.volume;
            }
            position.margin = position.details.iter().map(|d| d.margin).sum();
            position.open_volume = position.details.iter().map(|d| d.volume).sum();
            position.open_amount = position
                .details
                .iter()
                .map(|d| Money::from_f64(d.open_price * d.volume as f64 * mult))
                .sum();
            position.open_cost = position.open_amount;
            for detail in &mut position.details {
                if detail.volume > 0 {
                    detail.last_settlement_price = price;
                }
                detail.close_profit = Money::ZERO;
                detail.close_profit_trade = Money::ZERO;
                detail.commission = Money::ZERO;
                detail.close_volume = 0;
                detail.close_amount = Money::ZERO;
            }
        }
        for account in staged.accounts.values_mut() {
            let final_equity = *final_equity
                .get(&AccountKey::new(&account.broker_id, &account.investor_id))
                .expect("account equity precomputed");
            account.pre_balance = final_equity;
            account.balance = final_equity;
            account.deposit = Money::ZERO;
            account.withdraw = Money::ZERO;
            account.close_profit = Money::ZERO;
            account.position_profit = Money::ZERO;
            account.commission = Money::ZERO;
            account.frozen_margin = Money::ZERO;
            account.frozen_commission = Money::ZERO;
        }
        staged.frozen.clear();
        staged.frozen_pos.clear();
        staged.group_activity.clear();
        staged.refresh_margin(catalog);
        if staged
            .accounts
            .values()
            .any(|a| a.frozen_margin != Money::ZERO || a.frozen_commission != Money::ZERO)
        {
            return Err("日结后仍存在冻结资金".into());
        }
        *self = staged;
        Ok(())
    }

    pub fn mark_to_market(
        &mut self,
        catalog: &Catalog,
        prices: &HashMap<String, f64>,
        pre_settlements: &HashMap<String, f64>,
        trading_day: &str,
    ) {
        for pos in self.positions.values_mut() {
            let mult = catalog
                .get(&pos.instrument_id)
                .map(|i| i.volume_multiple)
                .unwrap_or(1);
            let p = prices
                .get(&pos.instrument_id)
                .copied()
                .unwrap_or(pos.settlement_price);
            if let Some(ps) = pre_settlements.get(&pos.instrument_id) {
                if *ps > 0.0 && pos.yd_initial == 0 {
                    pos.pre_settlement_price = *ps;
                    for d in pos.details.iter_mut().filter(|d| !d.bootstrap) {
                        d.last_settlement_price = *ps;
                    }
                }
            }
            pos.position_profit = pos
                .details
                .iter()
                .map(|d| {
                    let basis = d.mark_basis(trading_day);
                    Position::detail_pnl(pos.side, basis, p, d.volume, mult)
                })
                .sum();
            pos.settlement_price = p;
        }
        let mut sums: HashMap<AccountKey, Money> = HashMap::new();
        for ((key, _, _), pos) in &self.positions {
            *sums.entry(key.clone()).or_insert(Money::ZERO) += pos.position_profit;
        }
        for a in self.accounts.values_mut() {
            let key = AccountKey::new(&a.broker_id, &a.investor_id);
            a.position_profit = sums.get(&key).copied().unwrap_or(Money::ZERO);
        }
    }
}
