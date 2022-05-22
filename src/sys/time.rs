use x86_64::{
    instructions::{interrupts, port::Port},
    structures::idt::InterruptStackFrame,
};

use crate::sys::{clock::uptime, idt::Irq, pic::PICS};
use core::sync::atomic::{AtomicUsize, Ordering};

pub const PIT_FREQUENCY: f64 = 3_579_545.0 / 3.0; // 1_193_181.666 Hz
const PIT_DIVIDER: u32 = 1193;
pub const PIT_INTERVAL_SECS: f64 = (PIT_DIVIDER as f64) / PIT_FREQUENCY;

static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);

pub fn ticks() -> usize {
    PIT_TICKS.load(Ordering::Relaxed)
}

pub fn halt() {
    let disabled = !interrupts::are_enabled();
    interrupts::enable_and_hlt();
    if disabled {
        interrupts::disable();
    }
}

pub fn sleep(seconds: f64) {
    let start = uptime();
    while uptime() - start < seconds {
        halt();
    }
}

/// Set the divisor of the PIT. `divisor == 0` actually means
/// 65536.
fn set_pit_frequency_divider(divider: u16, channel: u8) {
    interrupts::without_interrupts(|| {
        let bytes = divider.to_le_bytes();
        let mut cmd: Port<u8> = Port::new(0x43);
        let mut data: Port<u8> = Port::new(0x40 + channel as u16);
        let operating_mode = 6; // Square wave generator
        let access_mode = 3; // Lobyte + Hibyte

        unsafe {
            cmd.write((channel << 6) | (access_mode << 4) | operating_mode);
            data.write(bytes[0]);
            data.write(bytes[1]);
        }
    });
}

pub(crate) extern "x86-interrupt" fn handle_timer_interrupt(_stack_frame: InterruptStackFrame) {
    PIT_TICKS.fetch_add(1, Ordering::Relaxed);

    unsafe { PICS.lock().notify_end_of_interrupt(Irq::Timer.as_u8()) }
}

pub fn init() {
    let divider = if PIT_DIVIDER < u16::MAX.into() {
        PIT_DIVIDER as _
    } else {
        0
    };
    let channel = 0;
    set_pit_frequency_divider(divider, channel);
}
