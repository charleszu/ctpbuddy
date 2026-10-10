//! Account ledger: funds, positions, margin and commission (DESIGN.md §8.6).
//!
//! Accounting model (逐日盯市, notes/04):
//! - `balance` is realized equity; `dynamic_equity()` adds deposits/withdraws
//!   and today's `position_profit` — CTP's `Balance` field;
//! - `available = dynamic_equity - used_margin - frozen_margin - frozen_commission`;
//! - positions keep per-lot details (先开先平); close PnL is priced per lot off
//!   昨结算 (carried lots) or the entry price (today's lots);
//! - margin is the shared per-product aggregate (`product_group_margin`,
//!   品种内大单边 when the instrument enables it), commission is priced per
//!   leg (开仓 / 平昨 / 平今) — CFFEX picks the 平今 lots off a trade-time
//!   opening pool, others off the consumed details' age;
//! - opening orders freeze estimated margin + commission, released pro-rata on
//!   fills; closing orders reserve position volume per age bucket
//!   (`close_buckets`: SHFE/INE 「平仓」= 平昨).
//!
//! The ledger is a single-writer state machine driven by the world loop; it
//! never performs IO.

use std::collections::HashMap;

use ctpbuddy_wire::generated::{cstr, set_cstr, CThostFtdcInvestorProductGroupMarginField};

use ctpbuddy_matching::{Catalog, CommissionKind, Fill, OffsetFlag};

pub const INITIAL_FUNDS: f64 = 2_000_000.0;

// Official CTP error.xml codes (docs/错误码全集.md): ids and prompts verbatim —
// downstream clients key off these numbers.
pub const ERR_FUNDS: i32 = 31; //          INSUFFICIENT_MONEY       CTP:资金不足
pub const ERR_POSITION: i32 = 30; //       OVER_CLOSE_POSITION      CTP:平仓量超过持仓量
pub const ERR_NO_CLOSE_TODAY_LEDGER: i32 = 50; // OVER_CLOSETODAY_POSITION CTP:平今仓位不足
pub const ERR_NO_CLOSE_YD_LEDGER: i32 = 51; // OVER_CLOSEYESTERDAY_POSITION CTP:平昨仓位不足

/// Which age buckets a close may consume: `(today_only, yd_only)`.
///
/// 上期所/能源中心区分今昨仓，报入「平仓」(`Close`) 等同平昨，平今必须报
/// `CloseToday`（知识库 §4.3，notes/02 B10）。其他交易所不区分今昨，`Close`
/// 按先开先平消耗；它们收到的平今/平昨在柜台侧统一转为平仓。
pub fn close_buckets(offset: OffsetFlag, exchange_id: &str) -> (bool, bool) {
    if !ctpbuddy_matching::rules_for(exchange_id).splits_today_yesterday {
        return (false, false);
    }
    match offset {
        OffsetFlag::CloseToday => (true, false),
        OffsetFlag::Close | OffsetFlag::CloseYesterday => (false, true),
        OffsetFlag::Open => (false, false),
    }
}

mod account;
mod money;
mod position;
mod settlement;
mod snapshot;
#[cfg(test)]
mod tests;

pub use account::{Account, AccountKey};
pub use money::Money;
pub use position::{Position, PositionDetail, PositionSide};

/// Frozen funds estimate attached to one order. `margin` / `commission` keep
/// the **original** estimate; each fill releases `original * filled_fraction`
/// and the released part is tracked, so a fully-filled order (no terminal
/// '5', hence no `unfreeze_order`) still ends at exactly zero frozen.
#[derive(Clone, Debug)]
struct FrozenEst {
    key: AccountKey,
    instrument_id: String,
    side: PositionSide,
    margin: Money,
    commission: Money,
    /// Lots of the order filled so far and the order's original size; the
    /// released share is `original * filled / total`, computed from the
    /// original so a fully-filled order ends at exactly zero.
    filled: i64,
    total: i64,
}

impl FrozenEst {
    fn remaining_margin(&self) -> Money {
        (self.margin - self.margin.ratio(self.filled, self.total)).floor_zero()
    }

    fn remaining_commission(&self) -> Money {
        (self.commission - self.commission.ratio(self.filled, self.total)).floor_zero()
    }
}

/// Position volume reserved by an outstanding close order (today/yd split),
/// released on fill or cancel.
#[derive(Clone, Debug)]
struct FrozenPos {
    key: AccountKey,
    instrument_id: String,
    side: PositionSide,
    today: i32,
    yd: i32,
}

type Grid = [[Money; 2]; 4];

#[derive(Default)]
struct GroupAgg {
    used: Grid,
    frozen: Grid,
    position_profit: Money,
    frozen_commission: Money,
    commission: Money,
    close_profit: Money,
}

impl GroupAgg {
    fn use_margin(&self) -> Money {
        let u = &self.used;
        u[0][0] + u[0][1] + u[1][0].max(u[1][1])
    }

    fn frozen_margin(&self) -> Money {
        let (u, f) = (&self.used, &self.frozen);
        let current_max = u[1][0].max(u[1][1]);
        let projected_max = (u[1][0] + f[1][0]).max(u[1][1] + f[1][1]);
        f[0][0] + f[0][1] + (projected_max - current_max).floor_zero()
    }
}

#[derive(Clone)]
pub struct Ledger {
    accounts: HashMap<AccountKey, Account>,
    positions: HashMap<(AccountKey, String, PositionSide), Position>,
    /// order_key -> remaining frozen funds estimate.
    frozen: HashMap<String, FrozenEst>,
    /// order_key -> position volume reservation.
    frozen_pos: HashMap<String, FrozenPos>,
    initial_funds: Money,
    group_activity: HashMap<(AccountKey, String, String), (Money, Money)>,
}

impl Ledger {
    pub fn new(initial_funds: f64) -> Self {
        Ledger {
            accounts: HashMap::new(),
            positions: HashMap::new(),
            frozen: HashMap::new(),
            frozen_pos: HashMap::new(),
            initial_funds: Money::from_f64(initial_funds),
            group_activity: HashMap::new(),
        }
    }

    /// Auto-open an account on first login (SimNow-style convenience): every
    /// investor starts with `initial_funds`.
    pub fn set_initial_funds(&mut self, initial_funds: f64) {
        self.initial_funds = Money::from_f64(initial_funds);
    }

    pub fn ensure_account(&mut self, broker_id: &str, investor_id: &str) -> &mut Account {
        let initial = self.initial_funds;
        self.accounts
            .entry(AccountKey::new(broker_id, investor_id))
            .or_insert_with(|| Account::new(broker_id, investor_id, initial))
    }

    /// Ensure an account exists with scenario-authored initial funds
    /// (DESIGN.md §7.4 `accounts`). Existing accounts keep their state; a
    /// non-positive `funds` falls back to the server default.
    pub fn ensure_account_with(
        &mut self,
        broker_id: &str,
        investor_id: &str,
        funds: f64,
    ) -> &mut Account {
        let initial = self.initial_funds;
        self.accounts
            .entry(AccountKey::new(broker_id, investor_id))
            .or_insert_with(|| {
                Account::new(
                    broker_id,
                    investor_id,
                    if funds > 0.0 {
                        Money::from_f64(funds)
                    } else {
                        initial
                    },
                )
            })
    }

    pub fn account(&self, broker_id: &str, investor_id: &str) -> Option<&Account> {
        self.accounts.get(&AccountKey::new(broker_id, investor_id))
    }

    pub fn account_mut(&mut self, broker_id: &str, investor_id: &str) -> Option<&mut Account> {
        self.accounts
            .get_mut(&AccountKey::new(broker_id, investor_id))
    }

    pub fn accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }

    /// Reset an account to its initial state (admin reset): wipes positions and
    /// any outstanding freezes belonging to the account.
    pub fn reset_account(&mut self, broker_id: &str, investor_id: &str) {
        let key = AccountKey::new(broker_id, investor_id);
        let fresh = Account::new(broker_id, investor_id, self.initial_funds);
        self.accounts.insert(key.clone(), fresh);
        self.positions.retain(|(k, _, _), _| *k != key);
        self.frozen.retain(|_, est| est.key != key);
        self.frozen_pos.retain(|_, est| est.key != key);
        self.group_activity.retain(|(k, _, _), _| *k != key);
    }

    pub fn position_mut_or_create(
        &mut self,
        broker_id: &str,
        investor_id: &str,
        instrument_id: &str,
        side: PositionSide,
    ) -> &mut Position {
        let key = AccountKey::new(broker_id, investor_id);
        self.positions
            .entry((key, instrument_id.to_string(), side))
            .or_insert_with(|| Position::new(instrument_id, side))
    }

    pub fn position(
        &self,
        broker_id: &str,
        investor_id: &str,
        instrument_id: &str,
        side: PositionSide,
    ) -> Option<&Position> {
        let key = AccountKey::new(broker_id, investor_id);
        self.positions.get(&(key, instrument_id.to_string(), side))
    }

    pub fn positions_of(&self, broker_id: &str, investor_id: &str) -> Vec<&Position> {
        let key = AccountKey::new(broker_id, investor_id);
        self.positions
            .iter()
            .filter(|((k, _, _), _)| *k == key)
            .map(|(_, p)| p)
            .collect()
    }

    /// Positions in a **stable order** — by instrument, then by side.
    ///
    /// The query surfaces must be reproducible: the backing store is a
    /// `HashMap`, so iterating it directly hands clients a different row order
    /// on every run. That is not cosmetic — a client diffing two snapshots, or
    /// a journal hash covering a query stream, would see spurious diffs.
    pub fn positions_of_ordered(
        &self,
        broker_id: &str,
        investor_id: &str,
    ) -> Vec<(&Position, &PositionDetail)> {
        let key = AccountKey::new(broker_id, investor_id);
        let mut out: Vec<(&Position, &PositionDetail)> = self
            .positions
            .iter()
            .filter(|((k, _, _), _)| *k == key)
            .flat_map(|(_, p)| {
                p.details
                    .iter()
                    .filter(|d| d.volume > 0)
                    .map(move |d| (p, d))
            })
            .collect();
        out.sort_by(|a, b| {
            a.0.instrument_id
                .cmp(&b.0.instrument_id)
                .then(a.0.side.cmp(&b.0.side))
                // Within a position, 先开先平 order is the meaningful one: it
                // is the order a client will consume them in.
                .then(a.1.open_date.cmp(&b.1.open_date))
                .then(a.1.trade_id.cmp(&b.1.trade_id))
        });
        out
    }

    /// Freeze estimated margin + commission for a new order. Fails with
    /// [`ERR_FUNDS`] when availability is insufficient.
    pub fn freeze(
        &mut self,
        order_key: &str,
        broker_id: &str,
        investor_id: &str,
        instrument_id: &str,
        side: PositionSide,
        est_margin: f64,
        est_commission: f64,
        catalog: &Catalog,
    ) -> Result<(), i32> {
        self.ensure_account(broker_id, investor_id);
        let key = AccountKey::new(broker_id, investor_id);
        self.frozen.insert(
            order_key.to_string(),
            FrozenEst {
                key: key.clone(),
                instrument_id: instrument_id.to_string(),
                side,
                margin: Money::from_f64(est_margin),
                commission: Money::from_f64(est_commission),
                filled: 0,
                total: 1,
            },
        );
        self.refresh_account(&key, catalog);
        if self.accounts[&key].available() < Money::from_f64(-1e-6) {
            self.frozen.remove(order_key);
            self.refresh_account(&key, catalog);
            return Err(ERR_FUNDS);
        }
        Ok(())
    }

    /// Per (exchange, product) aggregates for one account, in fixed point.
    /// Grids are indexed `[bucket][side]`: buckets 0/1 = plain / 大单边-enabled
    /// 占用; 2/3 = the same split at the 交易所 rate. Side 0 = long, 1 = short.
    fn group_aggs(
        &self,
        key: &AccountKey,
        catalog: &Catalog,
    ) -> std::collections::BTreeMap<(String, String), GroupAgg> {
        let mut groups: std::collections::BTreeMap<(String, String), GroupAgg> =
            std::collections::BTreeMap::new();
        for ((k, id, side), pos) in &self.positions {
            if k != key {
                continue;
            }
            let Some(i) = catalog.get(id) else {
                continue;
            };
            let g = groups
                .entry((i.exchange_id.clone(), i.product_id.clone()))
                .or_default();
            let s = if *side == PositionSide::Long { 0 } else { 1 };
            let bucket = if catalog.refdata().product_margin_algorithm(id) {
                1
            } else {
                0
            };
            g.used[bucket][s] += pos.margin;
            let ratio = if s == 0 {
                i.long_margin_ratio
            } else {
                i.short_margin_ratio
            };
            // ExchMargin is query-only: use each detail's booked price
            // baseline, never a later market quote.
            g.used[2 + bucket][s] += pos
                .details
                .iter()
                .map(|d| {
                    Money::from_f64(
                        d.margin_price * d.volume as f64 * i.volume_multiple as f64 * ratio,
                    )
                })
                .sum::<Money>();
            g.position_profit += pos.position_profit;
        }
        for est in self.frozen.values().filter(|e| e.key == *key) {
            let Some(i) = catalog.get(&est.instrument_id) else {
                continue;
            };
            let g = groups
                .entry((i.exchange_id.clone(), i.product_id.clone()))
                .or_default();
            let s = if est.side == PositionSide::Long { 0 } else { 1 };
            let bucket = if catalog
                .refdata()
                .product_margin_algorithm(&est.instrument_id)
            {
                1
            } else {
                0
            };
            g.frozen[bucket][s] += est.remaining_margin();
            g.frozen_commission += est.remaining_commission();
        }
        for ((k, ex, prod), (commission, profit)) in &self.group_activity {
            if k != key {
                continue;
            }
            let g = groups.entry((ex.clone(), prod.clone())).or_default();
            g.commission = *commission;
            g.close_profit = *profit;
        }
        groups
    }

    /// 按账户、交易所、品种聚合；目前账本仅支持投机、空投资单元。
    /// 未启用优惠的合约独立求和，不能被同品种优惠合约抵消。
    pub fn product_group_margin(
        &self,
        broker: &str,
        investor: &str,
        catalog: &Catalog,
        day: &str,
    ) -> Vec<CThostFtdcInvestorProductGroupMarginField> {
        let key = AccountKey::new(broker, investor);
        self.group_aggs(&key, catalog)
            .into_iter()
            .map(|((ex, prod), g)| {
                let (used, frozen) = (&g.used, &g.frozen);
                let mut f = CThostFtdcInvestorProductGroupMarginField::zeroed();
                set_cstr(&mut f.BrokerID, broker);
                set_cstr(&mut f.InvestorID, investor);
                set_cstr(&mut f.ExchangeID, &ex);
                set_cstr(&mut f.ProductGroupID, &prod);
                set_cstr(&mut f.TradingDay, day);
                f.SettlementID = 1;
                f.HedgeFlag = b'1';
                f.PositionProfit = g.position_profit.to_f64();
                f.FrozenCommission = g.frozen_commission.to_f64();
                f.Commission = g.commission.to_f64();
                f.CloseProfit = g.close_profit.to_f64();
                f.LongUseMargin = (used[0][0] + used[1][0]).to_f64();
                f.ShortUseMargin = (used[0][1] + used[1][1]).to_f64();
                f.UseMargin = g.use_margin().to_f64();
                f.LongFrozenMargin = (frozen[0][0] + frozen[1][0]).to_f64();
                f.ShortFrozenMargin = (frozen[0][1] + frozen[1][1]).to_f64();
                f.FrozenMargin = g.frozen_margin().to_f64();
                // 交易所费率独立展示；没有跨品种映射或仓单折抵规则，不推导折抵金额。
                f.LongExchMargin = (used[2][0] + used[3][0]).to_f64();
                f.ShortExchMargin = (used[2][1] + used[3][1]).to_f64();
                f.ExchMargin = (used[2][0] + used[2][1] + used[3][0].max(used[3][1])).to_f64();
                f
            })
            .collect()
    }
    pub fn refresh(&mut self, catalog: &Catalog) {
        self.refresh_margin(catalog);
    }

    fn refresh_margin(&mut self, catalog: &Catalog) {
        let keys: Vec<_> = self.accounts.keys().cloned().collect();
        for key in keys {
            self.refresh_account(&key, catalog);
        }
    }

    /// Recompute one account's margin aggregates. Freezes, fills and cancels
    /// only ever change their own account, so they refresh just that one.
    fn refresh_account(&mut self, key: &AccountKey, catalog: &Catalog) {
        let groups = self.group_aggs(key, catalog);
        if let Some(a) = self.accounts.get_mut(key) {
            a.used_margin = groups.values().map(GroupAgg::use_margin).sum();
            a.frozen_margin = groups.values().map(GroupAgg::frozen_margin).sum();
            a.frozen_commission = groups.values().map(|g| g.frozen_commission).sum();
        }
    }

    /// Reserve closeable volume for a close order.
    ///
    /// Which bucket may be touched comes from [`close_buckets`]: on SHFE/INE
    /// `CloseToday` reserves 今仓 only and `Close`/`CloseYesterday` reserve 昨仓
    /// only (上期所报「平仓」等同平昨). Elsewhere every close flag is a plain
    /// 先开先平 close that reserves **昨仓 first**: notes/04 B3 and the P1
    /// worked example (2 昨 + 2 今, close 3 → 2 昨 + 1 今).
    #[allow(clippy::too_many_arguments)]
    pub fn freeze_close_position(
        &mut self,
        order_key: &str,
        broker_id: &str,
        investor_id: &str,
        instrument_id: &str,
        exchange_id: &str,
        side: PositionSide,
        offset: OffsetFlag,
        volume: i32,
    ) -> Result<(), i32> {
        let key = AccountKey::new(broker_id, investor_id);
        let pos = match self
            .positions
            .get_mut(&(key, instrument_id.to_string(), side))
        {
            Some(p) => p,
            None => return Err(ERR_POSITION),
        };
        let avail_today = pos.today_position - pos.frozen_today;
        let avail_yd = pos.yd_position - pos.frozen_yd;
        let (from_today, from_yd) = match close_buckets(offset, exchange_id) {
            (true, _) => {
                if avail_today < volume {
                    return Err(ERR_NO_CLOSE_TODAY_LEDGER);
                }
                (volume, 0)
            }
            (_, true) => {
                if avail_yd < volume {
                    return Err(ERR_NO_CLOSE_YD_LEDGER);
                }
                (0, volume)
            }
            _ => {
                let y = volume.min(avail_yd);
                let rest = volume - y;
                if rest > avail_today {
                    return Err(ERR_POSITION);
                }
                (rest, y)
            }
        };
        pos.frozen_today += from_today;
        pos.frozen_yd += from_yd;
        self.frozen_pos.insert(
            order_key.to_string(),
            FrozenPos {
                key: AccountKey::new(broker_id, investor_id),
                instrument_id: instrument_id.to_string(),
                side,
                today: from_today,
                yd: from_yd,
            },
        );
        Ok(())
    }

    /// Release whatever remains frozen for an order (cancel path): the
    /// unreleased part of the funds estimate + the position reservation.
    /// Account freezes are recomputed from the remaining entries, so this
    /// only drops the order's entries and refreshes its account.
    pub fn unfreeze_order(&mut self, order_key: &str, catalog: &Catalog) {
        let mut touched = None;
        if let Some(est) = self.frozen.remove(order_key) {
            touched = Some(est.key);
        }
        if let Some(res) = self.frozen_pos.remove(order_key) {
            if let Some(pos) =
                self.positions
                    .get_mut(&(res.key.clone(), res.instrument_id.clone(), res.side))
            {
                pos.frozen_today = (pos.frozen_today - res.today).max(0);
                pos.frozen_yd = (pos.frozen_yd - res.yd).max(0);
            }
            touched.get_or_insert(res.key);
        }
        if let Some(key) = touched {
            self.refresh_account(&key, catalog);
        }
    }

    /// Settle a fill: release the pro-rata freeze, pay commission, move margin
    /// and volume, realize close PnL.
    ///
    /// Commission is priced **per position leg**, not per fill: a `Close` that
    /// eats 2 lots of yesterday and 1 of today pays 平昨 for two and 平今 for
    /// one (notes/04 B3/D1). notes/04 B3 is explicit that even 大商所 — which
    /// does not split positions into 今/昨 — still prices the two with
    /// different rates, so a single blended rate is visibly wrong.
    ///
    /// `pre_settlement` is the instrument's 昨结算价 **as the engine currently
    /// sees it**. It is a parameter rather than a lookup because the first
    /// fill of a session lands before the ledger has ever been marked to
    /// market, and `MarginPriceType == '1'` (the default) charges 今仓 against
    /// 昨结算 — reading the position's own copy there would margin the opening
    /// fill at zero.
    ///
    /// `trading_day` decides each lot's 逐日盯市 basis (notes/04 E2): a detail
    /// opened today is marked against its entry, anything older against 昨结算.
    /// Passing it in rather than storing a day on the ledger keeps a single
    /// source of truth — the world loop's virtual clock, which scenario replay
    /// rewinds.
    pub fn on_fill(
        &mut self,
        fill: &Fill,
        catalog: &Catalog,
        pre_settlement: f64,
        trading_day: &str,
    ) {
        let instr = catalog.get(&fill.instrument_id);
        let before_commission = self
            .account(&cstr(&fill.broker_id), &cstr(&fill.investor_id))
            .unwrap()
            .commission;
        let before_profit = self
            .account(&cstr(&fill.broker_id), &cstr(&fill.investor_id))
            .unwrap()
            .close_profit;
        let mult = instr.map(|i| i.volume_multiple).unwrap_or(1);

        // ---- release pro-rata frozen estimate for this order ----
        // The fraction is of the ORIGINAL estimate (not of what is left), so
        // a multi-tranche fill that ends up fully filled releases exactly
        // 100%: 2/3 + 1/3 of the original, not of the shrinking remainder.
        // The account's frozen totals are recomputed from these entries by
        // `refresh_account` at the end of the fill.
        if let Some(est) = self.frozen.get_mut(&fill.order_key) {
            est.total = fill.volume_total_original.max(1) as i64;
            est.filled = (est.filled + fill.volume as i64).clamp(0, est.total);
        }

        let key = AccountKey::new(&cstr(&fill.broker_id), &cstr(&fill.investor_id));
        assert!(
            self.accounts.contains_key(&key),
            "account must exist: login auto-opens it"
        );

        // An open fill creates/increases a position on the order's side; a
        // close fill reduces the position on the opposite side (sell closes long).
        let side = if fill.offset == OffsetFlag::Open {
            PositionSide::of(fill.direction)
        } else {
            PositionSide::of(fill.direction).opposite()
        };
        let pos = self
            .positions
            .entry((key.clone(), fill.instrument_id.clone(), side))
            .or_insert_with(|| Position::new(&fill.instrument_id, side));

        let turnover = Money::from_f64(fill.price * fill.volume as f64 * mult as f64);
        if fill.offset == OffsetFlag::Open {
            // 昨结算价 comes from the tick stream, not from the position: on
            // the very first fill of a session the position has none yet, and
            // `MarginPriceType == '1'` prices 今仓 against it.
            if pos.pre_settlement_price <= 0.0 && pre_settlement > 0.0 {
                pos.pre_settlement_price = pre_settlement;
            }
            // 今仓保证金 follows `MarginPriceType`; the fill price is the
            // 开仓价 leg of that choice (notes/04 C2).
            let price = catalog.margin_price_for(true, pos.pre_settlement_price, fill.price);
            let margin_actual = Money::from_f64(catalog.margin(
                &fill.instrument_id,
                fill.direction,
                price,
                fill.volume,
            ));
            let commission = Money::from_f64(catalog.commission(
                &fill.instrument_id,
                CommissionKind::Open,
                fill.price,
                fill.volume,
            ));
            pos.commission += commission;
            pos.today_position += fill.volume;
            // 中金所平今费时间序池（知识库 §6.4/§10.4 #10）：当日开仓逐笔
            // 入池，随本 side 的 Position 存放——买开累买池、卖开累卖池，
            // 之后本 side 的平仓消耗的正是这个池（对手向语义的落点）。
            if ctpbuddy_matching::rules_for(&fill.exchange_id).fee_close_pool {
                pos.fee_open_pool += fill.volume;
            }
            pos.open_amount += turnover;
            pos.open_volume += fill.volume;
            pos.position_cost += turnover;
            // 开仓成本 = Σ开仓价×乘数×手数; commission is tracked separately.
            pos.open_cost += turnover;
            pos.margin += margin_actual;
            // notes/04 B2: an opening fill creates its own detail, keyed by
            // (OpenDate, TradeID). The margin travels with the lot so a later
            // close can release exactly what was charged, and so
            // `ReqQryInvestorPositionDetail` has rows to return.
            pos.add_detail(
                trading_day,
                &cstr(&fill.trade_id),
                fill.price,
                fill.volume,
                margin_actual,
                price.value(),
            );
            pos.details.last_mut().unwrap().commission = commission;
            let a = self.accounts.get_mut(&key).expect("account exists");
            a.balance -= commission;
            a.commission += commission;
            // 账户占用由共享聚合重算，不按成交保证金直接相加。
        } else {
            // ---- close: consume details oldest-first (先开先平) ----
            //
            // `close_buckets` says *which age bucket* may be touched (SHFE/INE
            // 平今 / 平昨，「平仓」等同平昨); the walk order is by opening time
            // either way, so the lots and their PnL stay consistent.
            let (want_today_only, want_yd_only) = close_buckets(fill.offset, &fill.exchange_id);

            // 逐日盯市 PnL, priced per detail (notes/04 E2). 昨仓 lots are
            // marked against 昨结算, 今仓 lots against their own entry — so
            // one fill can span two bases, and pricing every lot off the
            // aggregate average cost (what this ledger did before details
            // existed) is wrong by exactly the age mix.
            let mut pnl = Money::ZERO;
            let mut trade_pnl = Money::ZERO;
            let mut closed_amount = Money::ZERO;
            let mut margin_released = Money::ZERO;
            let mut today_take = 0;
            let mut yd_take = 0;
            let taken =
                pos.take_details_filtered(fill.volume, want_today_only, want_yd_only, trading_day);
            for (idx, n) in &taken {
                let (idx, n) = (*idx, *n);
                let d = &mut pos.details[idx];
                let is_today = d.is_today(trading_day);
                if is_today {
                    today_take += n;
                } else {
                    yd_take += n;
                }
                let basis = d.mark_basis(trading_day);
                let leg_pnl = Position::detail_pnl(side, basis, fill.price, n, mult);
                let leg_trade_pnl = Position::detail_pnl(side, d.open_price, fill.price, n, mult);
                pnl += leg_pnl;
                trade_pnl += leg_trade_pnl;
                closed_amount += Money::from_f64(d.open_price * n as f64 * mult as f64);
                d.close_amount += Money::from_f64(fill.price * n as f64 * mult as f64);
                // Margin travels with the lot: release the pro-rata share of
                // what this detail was charged, not a recomputed figure.
                margin_released += d.margin.ratio(n as i64, d.open_volume.max(1) as i64);
                d.close_profit += leg_pnl;
                d.close_profit_trade += leg_trade_pnl;
                // 手续费不在这里计：平今/平昨的判定轴由下方按交易所规则
                // 选择（中金所=时间序池、其他=明细年龄），算出本次实收
                // 总额后再按手数分摊回明细。
            }
            let closed = today_take + yd_take;
            pos.today_position = (pos.today_position - today_take).max(0);
            pos.yd_position = (pos.yd_position - yd_take).max(0);
            // Release only what *this order* still has reserved, and shrink its
            // reservation by the same amount. A later cancel ('5') then frees
            // just the unfilled remainder; and when the '5' arrives first
            // (SHFE FAK `CancelFirst`) the reservation is already gone, so the
            // fill releases nothing a second time.
            let (rel_today, rel_yd) = match self.frozen_pos.get_mut(&fill.order_key) {
                Some(res) => {
                    let t = today_take.min(res.today);
                    let y = yd_take.min(res.yd);
                    res.today -= t;
                    res.yd -= y;
                    (t, y)
                }
                None => (0, 0),
            };
            if self
                .frozen_pos
                .get(&fill.order_key)
                .is_some_and(|res| res.today == 0 && res.yd == 0)
            {
                self.frozen_pos.remove(&fill.order_key);
            }
            pos.frozen_today = (pos.frozen_today - rel_today).max(0);
            pos.frozen_yd = (pos.frozen_yd - rel_yd).max(0);
            // `closed < fill.volume` is unreachable: freeze_close_position
            // reserved the volume at insert time (single-writer ledger).
            debug_assert_eq!(closed, fill.volume);

            pos.open_amount = (pos.open_amount - closed_amount).floor_zero();
            pos.open_volume = (pos.open_volume - closed).max(0);
            // 持仓成本 follows the **持仓价** basis (notes/04 B1), not the open
            // price: a 昨仓 lot is carried at 昨结算 until it is closed. Sum it
            // off the surviving details rather than subtracting, so a carried
            // lot keeps being valued at settlement instead of at its entry.
            pos.position_cost = pos
                .details
                .iter()
                .map(|d| Money::from_f64(d.mark_basis(trading_day) * d.volume as f64 * mult as f64))
                .sum();
            // 开仓成本 stays on the 开仓价 basis of the surviving lots.
            pos.open_cost = pos
                .details
                .iter()
                .map(|d| Money::from_f64(d.open_price * d.volume as f64 * mult as f64))
                .sum();

            // Commission is charged per leg: the 平今 portion at today's rate,
            // the 平昨 portion at yesterday's. Summing two legs is the whole
            // point — a blended single rate is what notes/04 B3 warns about.
            //
            // 平今手数取哪条轴？中金所按**成交时间序开仓池**（知识库
            // §6.4/§10.4 #10，生产数据 2026-10-08 判别 81/81）：先平当日
            // 新开仓、再平历史仓，与上面明细的先开先平消耗互不参考——
            // 昨仓被平可收平今费、今仓被平可收平昨费（IM2410/20240924
            // 双向实锤）。其他交易所跟随被平明细年龄（SHFE 平今指令按
            // 指令桶消耗，today_take 即平今手数）。
            let (fee_today, fee_yd) =
                if ctpbuddy_matching::rules_for(&fill.exchange_id).fee_close_pool {
                    let t = closed.min(pos.fee_open_pool);
                    pos.fee_open_pool -= t;
                    (t, closed - t)
                } else {
                    (today_take, yd_take)
                };
            let comm_today = Money::from_f64(catalog.commission(
                &fill.instrument_id,
                CommissionKind::CloseToday,
                fill.price,
                fee_today,
            ));
            let comm_yd = Money::from_f64(catalog.commission(
                &fill.instrument_id,
                CommissionKind::CloseYesterday,
                fill.price,
                fee_yd,
            ));
            let commission = comm_today + comm_yd;
            pos.commission += commission;
            // 明细手续费按本次实收总额逐笔分摊（最后一笔吃定点残差，明细
            // 之和恒等于实收）。分摊跟随明细消耗序；计费轴已在上面对照
            // 交易所规则选定，两者解耦。
            let mut allocated = Money::ZERO;
            let last = taken.len().saturating_sub(1);
            for (i, (idx, n)) in taken.iter().enumerate() {
                let share = if i == last {
                    commission - allocated
                } else {
                    commission.ratio(*n as i64, closed as i64)
                };
                allocated += share;
                pos.details[*idx].commission += share;
            }

            pos.margin = (pos.margin - margin_released).floor_zero();
            pos.close_profit += pnl;
            pos.close_profit_trade += trade_pnl;

            let a = self.accounts.get_mut(&key).expect("account exists");
            a.balance -= commission;
            a.commission += commission;
            // 平仓后大边可能切换，账户保证金交给共享聚合重算。
            a.balance += pnl;
            a.close_profit += pnl;

            // Exhausted details leave the query surface; CTP does not return
            // a 明细 with zero remaining volume.
            // 当天累计归属依赖已平明细；仅查询入口过滤零余量，不在盘中删除。
            // Keep a static bootstrap row after a full close. CTP can expose the
            // zero Position row until settlement; zero-volume details remain
            // hidden by the detail query projection.
            if pos.volume() == 0
                && pos.frozen_today == 0
                && pos.frozen_yd == 0
                && pos.yd_initial == 0
            {
                let k = (key.clone(), fill.instrument_id.clone(), side);
                self.positions.remove(&k);
            }
        }
        if let Some(i) = instr {
            let a = self.accounts.get(&key).unwrap();
            let activity = self
                .group_activity
                .entry((key.clone(), i.exchange_id.clone(), i.product_id.clone()))
                .or_default();
            activity.0 += a.commission - before_commission;
            activity.1 += a.close_profit - before_profit;
        }
        self.refresh_account(&key, catalog);
    }
}
