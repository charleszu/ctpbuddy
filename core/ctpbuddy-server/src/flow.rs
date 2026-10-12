//! Wall-clock front budgets; zero always means disabled.
use std::time::{Duration, Instant};

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
