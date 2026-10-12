//! Wall-clock front budgets; zero always means disabled.
use std::time::{Duration, Instant};

pub(crate) fn origin() -> Instant {
    static ORIGIN: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    *ORIGIN.get_or_init(Instant::now)
}

/// Journal the actual transport gate invocation, including denied connects.
pub(crate) fn connect_gate(
    budget: &mut WindowBudget,
    now: Instant,
    limit: u32,
    tx: &std::sync::mpsc::Sender<crate::WorldMsg>,
    scope: &'static str,
) -> bool {
    origin();
    let allowed = budget.take(now, limit);
    if limit != 0 {
        let _ = tx.send(crate::WorldMsg::FlowConnect {
            scope,
            clock: now,
            allowed,
        });
    }
    allowed
}

#[derive(Default)]
pub(crate) struct WindowBudget {
    start: Option<Instant>,
    used: u32,
}

impl WindowBudget {
    pub fn take(&mut self, now: Instant, limit: u32) -> bool {
        if limit == 0 {
            return true;
        }
        if self
            .start
            .map(|s| now.saturating_duration_since(s) >= Duration::from_secs(1))
            .unwrap_or(true)
        {
            self.start = Some(now);
            self.used = 0;
        }
        if self.used >= limit {
            return false;
        }
        self.used += 1;
        true
    }
}
