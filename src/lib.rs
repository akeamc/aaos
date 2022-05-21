#![no_std]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[macro_use]
pub mod serial;
#[macro_use]
pub mod vga_buffer;
pub mod gdt;
pub mod interrupts;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("test {} ... ", core::any::type_name::<T>());
        self();
        serial_println!("\x1b[32mok\x1b[0m");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!();
    serial_println!(
        "running {} {}",
        tests.len(),
        if tests.len() == 1 { "test" } else { "tests" }
    );
    for test in tests {
        test.run();
    }
    serial_println!();

    exit_qemu(QemuExitCode::Success)
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

/// Entrypoint for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();

    #[cfg(test)]
    test_main();

    hlt_loop()
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
