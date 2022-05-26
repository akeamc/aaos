use core::sync::atomic::{AtomicUsize, Ordering};

use crate::sys;

use time::{Duration, PrimitiveDateTime};
use x86_64::structures::idt::InterruptStackFrame;

use sys::time::{ticks, PIT_INTERVAL_SECS};

use self::cmos::Cmos;

mod cmos;

static LAST_RTC_UPDATE: AtomicUsize = AtomicUsize::new(0);

/// System uptime (seconds).
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn uptime() -> f64 {
    PIT_INTERVAL_SECS * ticks() as f64
}

/// # Panics
///
/// If the date returned by CMOS isn't a valid date,
/// this function panics.
#[must_use]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
pub fn realtime() -> PrimitiveDateTime {
    let datetime: PrimitiveDateTime = Cmos::new()
        .rtc_checked()
        .try_into()
        .expect("invalid date returned by CMOS");
    let fract = PIT_INTERVAL_SECS * (ticks() - LAST_RTC_UPDATE.load(Ordering::Relaxed)) as f64;
    let nanos = (fract * 1_000_000_000.0) as _;
    datetime + Duration::nanoseconds(nanos)
}

/// Handle an RTC interrupt.
pub(crate) extern "x86-interrupt" fn handle_rtc_interrupt(_stack_frame: InterruptStackFrame) {
    Cmos::new().notify_end_of_interrupt();
    LAST_RTC_UPDATE.store(ticks(), Ordering::Relaxed);
}

pub fn init() {
    Cmos::new().enable_update_interrupt();
}
