// in src/main.rs

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(aaos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use aaos::{
    println,
    vga_buffer::{Color, ColorCode, ScreenChar, BUFFER_HEIGHT, BUFFER_WIDTH},
};
use core::panic::PanicInfo;
use rand::{prelude::SmallRng, RngCore, SeedableRng};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();

    let mut small_rng = SmallRng::seed_from_u64(69420);

    let buf = &mut aaos::vga_buffer::WRITER.lock().buffer.chars;

    let mut color = ColorCode::new(Color::Black, Color::Black);

    let mut x = 10;
    let mut y = 10;
    let mut dx = 1;
    let mut dy = 1;

    let text = b"\x86keOS";

    loop {
        for i in 0..text.len() {
            buf[y as usize][x as usize + i].write(ScreenChar::new(
                b' ',
                ColorCode::new(Color::Black, Color::Black),
            ));
        }

        x += dx;
        y += dy;
        let mut bounce = false;

        if x == 0 {
            dx = 1;
            bounce = true;
        }

        if (x as usize + text.len()) == BUFFER_WIDTH {
            dx = -1;
            bounce = true;
        }

        if y == 0 {
            dy = 1;
            bounce = true;
        }

        if (y as usize) + 1 == BUFFER_HEIGHT {
            dy = -1;
            bounce = true;
        }

        if bounce {
            let mut buf = [0u8];
            small_rng.fill_bytes(&mut buf);

            color = ColorCode(buf[0])
        }

        /// i haven't implemented interrupts yet....
        for _ in 0..100000 {
            for (i, char) in text.iter().enumerate() {
                buf[y as usize][x as usize + i].write(ScreenChar::new(*char, color));
            }
        }

        for (i, char) in text.iter().enumerate() {
            buf[10][10 + i].write(ScreenChar::new(
                *char,
                ColorCode::new(Color::Black, Color::White),
            ));
        }

        for (i, char) in b"aaaaaaaaaaaaaaaaa".iter().enumerate() {
            buf[11][10 + i].write(ScreenChar::new(
                *char,
                ColorCode::new(Color::White, Color::Black),
            ));
        }
    }

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    aaos::test_panic_handler(info)
}
