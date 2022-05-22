#[macro_use]
pub mod serial;
#[macro_use]
pub mod vga;
pub mod allocator;
pub mod clock;
pub mod gdt;
pub mod idt;
pub mod keyboard;
pub mod memory;
pub mod pic;
pub mod time;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        if !cfg!(test) {
            let uptime = $crate::sys::clock::uptime();

            $crate::sys::vga::print_fmt(format_args!("\x1b[92m[{:>12.6}]\x1b[0m {}\n", uptime, format_args!($($arg)*)));
        }
    });
}
