//! Per-exchange behaviour table (DESIGN §8.3 「规则表化」).
//!
//! Every place that used to branch on a literal exchange id (`"SHFE" | "INE"`,
//! `"DCE" | "GFEX"`, ...) asks this table instead, so supporting a new
//! exchange or a rule change is a one-row edit. Unknown exchange codes get
//! [`ExchangeRules::DEFAULT`] (historical behaviour: SHFE-style FAK layout,
//! no today/yesterday split).

use crate::engine::IocLayout;

/// Exchange-specific behaviours the simulator reproduces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExchangeRules {
    pub id: &'static str,
    /// FAK report shape (官方《报单回调规则》场景 8/9/10).
    pub ioc_layout: IocLayout,
    /// 平今/平昨 are distinct offsets: close bucket selection, position rows
    /// split by 今/昨 and the trade report keeps CloseToday/CloseYesterday.
    /// Everywhere else the counter folds them into plain 平仓.
    pub splits_today_yesterday: bool,
    /// A fill that completes the order in one go is reported without the
    /// 前态 repeat (DCE group).
    pub skip_prev_state_on_self_complete: bool,
}

impl ExchangeRules {
    /// Fallback for codes not in the table.
    pub const DEFAULT: ExchangeRules = ExchangeRules {
        id: "",
        ioc_layout: IocLayout::CancelFirst,
        splits_today_yesterday: false,
        skip_prev_state_on_self_complete: false,
    };
}

/// The six CTP futures exchanges.
pub const TABLE: [ExchangeRules; 6] = [
    ExchangeRules {
        id: "SHFE",
        ioc_layout: IocLayout::CancelFirst,
        splits_today_yesterday: true,
        skip_prev_state_on_self_complete: false,
    },
    ExchangeRules {
        id: "INE",
        ioc_layout: IocLayout::CancelFirst,
        splits_today_yesterday: true,
        skip_prev_state_on_self_complete: false,
    },
    ExchangeRules {
        id: "CFFEX",
        ioc_layout: IocLayout::CancelFirst,
        splits_today_yesterday: false,
        skip_prev_state_on_self_complete: false,
    },
    ExchangeRules {
        id: "DCE",
        ioc_layout: IocLayout::TradeDriven,
        splits_today_yesterday: false,
        skip_prev_state_on_self_complete: true,
    },
    ExchangeRules {
        id: "GFEX",
        ioc_layout: IocLayout::TradeDriven,
        splits_today_yesterday: false,
        skip_prev_state_on_self_complete: true,
    },
    ExchangeRules {
        id: "CZCE",
        ioc_layout: IocLayout::StatusDriven,
        splits_today_yesterday: false,
        skip_prev_state_on_self_complete: false,
    },
];

/// Rules for `exchange_id` (exact, case-sensitive match like CTP).
pub fn rules_for(exchange_id: &str) -> &'static ExchangeRules {
    TABLE
        .iter()
        .find(|r| r.id == exchange_id)
        .unwrap_or(&ExchangeRules::DEFAULT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_matches_documented_groups() {
        for ex in ["SHFE", "INE"] {
            let r = rules_for(ex);
            assert!(r.splits_today_yesterday);
            assert_eq!(r.ioc_layout, IocLayout::CancelFirst);
        }
        assert_eq!(rules_for("CFFEX").ioc_layout, IocLayout::CancelFirst);
        for ex in ["DCE", "GFEX"] {
            let r = rules_for(ex);
            assert_eq!(r.ioc_layout, IocLayout::TradeDriven);
            assert!(r.skip_prev_state_on_self_complete);
            assert!(!r.splits_today_yesterday);
        }
        assert_eq!(rules_for("CZCE").ioc_layout, IocLayout::StatusDriven);
    }

    #[test]
    fn unknown_exchange_gets_default_and_ids_are_unique() {
        assert_eq!(rules_for("TEST"), &ExchangeRules::DEFAULT);
        assert_eq!(rules_for(""), &ExchangeRules::DEFAULT);
        for (i, a) in TABLE.iter().enumerate() {
            assert!(TABLE[i + 1..].iter().all(|b| b.id != a.id));
        }
    }
}
