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

            $crate::sys::vga::print_fmt(format_args!("[{:.6}] {}\n", uptime, format_args!($($arg)*)));
        }
    });
}
