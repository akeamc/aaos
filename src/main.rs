#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(aaos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use aaos::{log, println};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    aaos::init(boot_info);

    #[cfg(test)]
    test_main();

    log!("it did not crash");

    aaos::hlt_loop()
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    aaos::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    aaos::test_panic_handler(info)
}
