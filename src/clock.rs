use crate::time::{ticks, PIT_INTERVAL_SECS};

/// System uptime (seconds).
pub fn uptime() -> f64 {
    PIT_INTERVAL_SECS * ticks() as f64
}
