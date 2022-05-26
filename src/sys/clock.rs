use crate::sys::{
    self,
    pic::{Irq, PICS},
};

use x86_64::{
    instructions::{interrupts, port::Port},
    structures::idt::InterruptStackFrame,
};

use crate::log;
use sys::time::{ticks, PIT_INTERVAL_SECS};

use self::cmos::Cmos;

mod cmos;

/// System uptime (seconds).
pub fn uptime() -> f64 {
    PIT_INTERVAL_SECS * ticks() as f64
}

/// Handle an RTC interrupt.
pub(crate) extern "x86-interrupt" fn handle_rtc_interrupt(_stack_frame: InterruptStackFrame) {
    let mut cmos = Cmos::new();
    let rtc = cmos.rtc_unchecked();

    cmos.notify_end_of_interrupt();

    let dt = rtc.datetime();
}

pub fn init() {
    Cmos::new().enable_update_interrupt();
}
