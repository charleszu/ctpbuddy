//! Scenario transforms (DESIGN.md §7.4): pure functions over the tick stream.
//!
//! Transforms run in listed order at scenario load, before the virtual clock
//! starts. Each one is a total function `Vec<Tick> -> Vec<Tick>`; the input is
//! never mutated, so the transformed stream is fully determined by the source
//! data plus the scenario file (the §7.6 determinism promise).
//!
//! Every transform carries an optional `instrument` filter: `None` applies
//! to the whole stream, `Some(id)` only to that contract's ticks. A
//! multi-contract scenario can thus gap one product without shifting the
//! others (a gap on `rb2610` must not move `au2612`).

use crate::{Tick, DEPTH};

/// One step of the scenario pipeline.
#[derive(Clone, Debug)]
pub enum Transform {
    /// 停牌: drop every tick in `[at, at + duration)` — no new market data.
    Freeze {
        at_ms: f64,
        duration_ms: f64,
        /// Only this contract (`None` = every contract).
        instrument: Option<String>,
    },
    /// 跳空: from `at` on, shift last/average/five-level prices by `shift`.
    /// Price limits are NOT shifted (the band is a rule-table concept; a jump
    /// inside the band does not move it — a jump through it gets rejected by
    /// the engine's price-limit check, which is the realistic behavior).
    Gap {
        at_ms: f64,
        shift: f64,
        /// Only this contract (`None` = every contract).
        instrument: Option<String>,
    },
    /// 流动性缩放: from `from` on, scale five-level volumes by `scale`
    /// (rounded half away from zero, clamped at 0).
    Liquidity {
        from_ms: f64,
        scale: f64,
        /// Only this contract (`None` = every contract).
        instrument: Option<String>,
    },
}

impl Transform {
    /// Kind tag for journaling / status.
    pub fn kind(&self) -> &'static str {
        match self {
            Transform::Freeze { .. } => "freeze",
            Transform::Gap { .. } => "gap",
            Transform::Liquidity { .. } => "liquidity",
        }
    }

    /// The contract filter (`None` = applies to every contract).
    pub fn instrument(&self) -> Option<&str> {
        match self {
            Transform::Freeze { instrument, .. }
            | Transform::Gap { instrument, .. }
            | Transform::Liquidity { instrument, .. } => instrument.as_deref(),
        }
    }

    /// Does this transform touch `tick`'s contract?
    fn targets(&self, tick: &Tick) -> bool {
        match self.instrument() {
            None => true,
            Some(id) => tick.instrument_id == id,
        }
    }
}

/// Apply every transform in order.
pub fn apply_all(ticks: &[Tick], transforms: &[Transform]) -> Vec<Tick> {
    let mut out: Vec<Tick> = ticks.to_vec();
    for t in transforms {
        out = apply_one(&out, t);
    }
    out
}

fn apply_one(ticks: &[Tick], t: &Transform) -> Vec<Tick> {
    match *t {
        Transform::Freeze {
            at_ms, duration_ms, ..
        } => ticks
            .iter()
            .filter(|tk| {
                let vt = tk.virtual_ms();
                !(t.targets(tk) && vt >= at_ms && vt < at_ms + duration_ms)
            })
            .cloned()
            .collect(),
        Transform::Gap { at_ms, shift, .. } => ticks
            .iter()
            .map(|tk| {
                let mut tk = tk.clone();
                if shift != 0.0 && t.targets(&tk) && tk.virtual_ms() >= at_ms {
                    tk.last_price += shift;
                    tk.average_price += shift;
                    for k in 0..DEPTH {
                        if tk.bid_prices[k] > 0.0 {
                            tk.bid_prices[k] += shift;
                        }
                        if tk.ask_prices[k] > 0.0 {
                            tk.ask_prices[k] += shift;
                        }
                    }
                }
                tk
            })
            .collect(),
        Transform::Liquidity { from_ms, scale, .. } => ticks
            .iter()
            .map(|tk| {
                let mut tk = tk.clone();
                if scale != 1.0 && t.targets(&tk) && tk.virtual_ms() >= from_ms {
                    for k in 0..DEPTH {
                        tk.bid_volumes[k] = scale_vol(tk.bid_volumes[k], scale);
                        tk.ask_volumes[k] = scale_vol(tk.ask_volumes[k], scale);
                    }
                }
                tk
            })
            .collect(),
    }
}

/// Round half away from zero, clamped at 0 (volumes are never negative).
fn scale_vol(v: i32, scale: f64) -> i32 {
    ((v as f64 * scale).round() as i32).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// virtual ms since midnight for HH:MM
    fn hms(h: f64, m: f64) -> f64 {
        (h * 3600.0 + m * 60.0) * 1000.0
    }

    fn tick_at(t: &str, last: f64, bid1v: i32, ask1v: i32) -> Tick {
        let mut tk = Tick::default();
        tk.instrument_id = "rb2610".into();
        tk.update_time = t.into();
        tk.last_price = last;
        tk.bid_prices[0] = last - 2.0;
        tk.ask_prices[0] = last + 2.0;
        tk.bid_volumes[0] = bid1v;
        tk.ask_volumes[0] = ask1v;
        tk
    }

    #[test]
    fn freeze_drops_the_window() {
        let ticks = vec![
            tick_at("09:30:00", 3500.0, 10, 12),
            tick_at("09:32:00", 3503.0, 8, 10),
            tick_at("09:33:00", 3504.0, 8, 10),
        ];
        let out = apply_all(
            &ticks,
            &[Transform::Freeze {
                at_ms: hms(9.0, 32.0),
                duration_ms: 30_000.0,
                instrument: None,
            }],
        );
        assert_eq!(out.len(), 2);
        assert_eq!(out[1].update_time, "09:33:00");
    }

    #[test]
    fn gap_shifts_prices_from_at_on() {
        let ticks = vec![
            tick_at("09:30:00", 3500.0, 10, 12),
            tick_at("09:33:00", 3495.0, 6, 8),
        ];
        let out = apply_all(
            &ticks,
            &[Transform::Gap {
                at_ms: hms(9.0, 33.0),
                shift: -2.0,
                instrument: None,
            }],
        );
        assert_eq!(out[0].last_price, 3500.0); // before: untouched
        assert_eq!(out[1].last_price, 3493.0);
        assert_eq!(out[1].bid_prices[0], 3491.0);
        assert_eq!(out[1].ask_prices[0], 3495.0);
    }

    #[test]
    fn liquidity_scales_volumes() {
        let ticks = vec![tick_at("09:31:00", 3502.0, 8, 10)];
        let out = apply_all(
            &ticks,
            &[Transform::Liquidity {
                from_ms: hms(9.0, 31.0),
                scale: 0.5,
                instrument: None,
            }],
        );
        assert_eq!(out[0].bid_volumes[0], 4);
        assert_eq!(out[0].ask_volumes[0], 5);
        assert_eq!(out[0].last_price, 3502.0); // prices untouched
    }

    #[test]
    fn transforms_compose_in_order() {
        let ticks = vec![
            tick_at("09:30:00", 3500.0, 10, 12),
            tick_at("09:31:00", 3502.0, 8, 10),
            tick_at("09:32:00", 3503.0, 8, 10), // dropped by the freeze
            tick_at("09:33:00", 3495.0, 6, 8),
        ];
        let out = apply_all(
            &ticks,
            &[
                Transform::Freeze {
                    at_ms: hms(9.0, 32.0),
                    duration_ms: 30_000.0,
                    instrument: None,
                },
                Transform::Liquidity {
                    from_ms: hms(9.0, 31.0),
                    scale: 0.5,
                    instrument: None,
                },
                Transform::Gap {
                    at_ms: hms(9.0, 33.0),
                    shift: -2.0,
                    instrument: None,
                },
            ],
        );
        assert_eq!(out.len(), 3); // 09:30 / 09:31 / 09:33 (09:32 frozen away)
        assert_eq!(out[0].ask_volumes[0], 12); // before liquidity: untouched
        assert_eq!(out[1].ask_volumes[0], 5); // liquidity only
        assert_eq!(out[2].last_price, 3493.0); // gap (and liquidity)
        assert_eq!(out[2].ask_volumes[0], 4);
    }

    // 多合约：带 instrument 的 transform 只动该合约，其他品种的 tick 原封不动；
    // 不带 instrument 仍对全部生效。
    #[test]
    fn instrument_filter_scopes_each_transform() {
        let mut au = tick_at("09:33:00", 800.0, 20, 30);
        au.instrument_id = "au2612".into();
        let ticks = vec![
            tick_at("09:32:00", 3503.0, 8, 10),
            tick_at("09:33:00", 3495.0, 6, 8),
            au,
        ];
        let out = apply_all(
            &ticks,
            &[
                Transform::Gap {
                    at_ms: hms(9.0, 33.0),
                    shift: -2.0,
                    instrument: Some("rb2610".into()),
                },
                Transform::Liquidity {
                    from_ms: hms(9.0, 30.0),
                    scale: 0.5,
                    instrument: Some("rb2610".into()),
                },
                Transform::Freeze {
                    at_ms: hms(9.0, 32.0),
                    duration_ms: 30_000.0,
                    instrument: Some("au2612".into()),
                },
            ],
        );
        // rb 09:32 survives the au-only freeze; rb 09:33 gapped + halved
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].last_price, 3503.0);
        assert_eq!(out[0].bid_volumes[0], 4);
        assert_eq!(out[1].last_price, 3493.0);
        assert_eq!(out[1].ask_volumes[0], 4);
        // au: untouched by the rb-scoped gap/liquidity, outside the freeze window
        assert_eq!(out[2].instrument_id, "au2612");
        assert_eq!(out[2].last_price, 800.0);
        assert_eq!(out[2].bid_volumes[0], 20);

        // an au-scoped freeze covering 09:33 drops only the au tick
        let out = apply_all(
            &ticks,
            &[Transform::Freeze {
                at_ms: hms(9.0, 33.0),
                duration_ms: 30_000.0,
                instrument: Some("au2612".into()),
            }],
        );
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|t| t.instrument_id == "rb2610"));

        // no filter: everything shifts
        let out = apply_all(
            &ticks,
            &[Transform::Gap {
                at_ms: hms(9.0, 33.0),
                shift: -2.0,
                instrument: None,
            }],
        );
        assert_eq!(out[1].last_price, 3493.0);
        assert_eq!(out[2].last_price, 798.0);
    }
}
