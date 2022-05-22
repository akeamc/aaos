#![no_std]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![cfg_attr(test, no_main)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[macro_use]
pub mod serial;
#[macro_use]
pub mod vga_buffer;
pub mod allocator;
pub mod clock;
pub mod gdt;
pub mod idt;
pub mod memory;
pub mod time;

pub fn init() {
    gdt::init();
    idt::init_idt();
    unsafe { idt::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    time::init();
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

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entrypoint for `cargo test`
#[cfg(test)]
#[no_mangle]
pub fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
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

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
