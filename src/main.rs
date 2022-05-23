#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(aaos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use aaos::sys::time::sleep;
use aaos::{log, println, sys::vga::Color};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::instructions::interrupts::without_interrupts;

entry_point!(kernel_main);

use aaos::sys::vga::{Buffer, CharColor, ScreenChar, BUFFER_HEIGHT, WRITER};

pub fn write_bytes(
    buffer: &mut Buffer,
    row: usize,
    col: usize,
    bytes: impl AsRef<[u8]>,
    color: CharColor,
) {
    for (i, b) in bytes.as_ref().iter().enumerate() {
        unsafe {
            core::ptr::write_volatile(&mut buffer.chars[row][col + i], ScreenChar::new(*b, color))
        };
    }
}

/// Prints a 34x23 logo with its top left corner at `(row, col)`.
pub fn print_logo(row: usize, col: usize) {
    const LOGO: &str = "  &#   *@@@@@@@@*                 
  @#         #@@@@@@@,            
.@@@*          ,@@@&       .**    
.@@@@@@#.        *,     #&*@@@@%  
.& /@@@@@@@@@@&#,       (%     .  
,@@/ *@@@@@@@@@@@@@@@@&@%         
.@@@@@#. ./#@@@@@@@@@@@@@@@@@(    
&@@@@@@@@&(,   ./#@@@@@@@@@@@@%   
  (@@@@@@@@@@@@@@@@/. ./&@@@@@@@( 
     .(%@@@@@@@@@@@@@@@&* .&@@@@@,
            .*#@@@@@@@@@@@( /@@@@.
              @%    *%@@@@& .@@@. 
            *@*        .@@, (@%   
   ..,*/((#@&,.        ,*  %%     
,@@@@@@@@@@@@@@@@@@@@@/     */    
#@%.       .@@,#@@@@@@@@@@@@,     
@,      .&@#        (@@@@*        
(@@@@/                            ";

    without_interrupts(|| {
        let mut writer = WRITER.lock();

        writer.clear_screen();

        for (i, line) in LOGO.lines().enumerate() {
            write_bytes(
                writer.buffer,
                row + i,
                col,
                line,
                CharColor::new(Color::LightGray, Color::Black),
            )
        }

        write_bytes(
            writer.buffer,
            row + 20,
            col + 5,
            b" S\x99DRA LATINS GYMNASIUM ",
            CharColor::new(Color::Black, Color::LightGray),
        );

        write_bytes(
            writer.buffer,
            row + 22,
            col + 1,
            b"vetenskap \x07 kultur \x07 kreativitet",
            CharColor::new(Color::LightGray, Color::Black),
        );
    });
}

fn print_trollface(row: usize, col: usize, color: CharColor) {
    const TROLLFACE: &[u8] = b"\
\xb0\xb0\xb0\xb0\xb0\xdc\xdc\xdc\xdc\xdf\xdf\xdf\xdf\xdf\xdf\xdf\xdf\xdc\xdc\xdc\xdc\xdc\xdc\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0
\xb0\xb0\xb0\xb0\xb0\xdb\xb0\xb0\xb0\xb0\xb1\xb1\xb1\xb1\xb1\xb1\xb1\xb1\xb1\xb1\xb1\xb1\xb0\xb0\xdf\xdf\xdc\xb0\xb0\xb0\xb0
\xb0\xb0\xb0\xb0\xdb\xb0\xb0\xb0\xb1\xb1\xb1\xb1\xb1\xb1\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb1\xb1\xb1\xb0\xb0\xdb\xb0\xb0\xb0
\xb0\xb0\xb0\xdb\xb0\xb0\xb0\xb0\xb0\xb0\xdc\xdb\xdb\xdf\xdc\xdc\xb0\xb0\xb0\xb0\xb0\xdc\xdc\xdc\xb0\xb0\xb0\xb0\xdb\xb0\xb0
\xb0\xdc\xdf\xb1\xdc\xdc\xdc\xb1\xb0\xdb\xdf\xdf\xdf\xdf\xdc\xdc\xdb\xb0\xb0\xb0\xdb\xdb\xdc\xdc\xdb\xb0\xb0\xb0\xb0\xdb\xb0
\xdb\xb0\xb1\xdb\xb1\xdc\xb0\xdf\xdc\xdc\xdc\xdf\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xdb\xb0\xb0\xb0\xb1\xb1\xb1\xb1\xb1\xb0\xdb
\xdb\xb0\xb1\xdb\xb0\xdb\xdf\xdc\xdc\xb0\xb0\xb0\xb0\xb0\xdb\xdf\xb0\xb0\xb0\xb0\xdf\xdc\xb0\xb0\xdc\xdf\xdf\xdf\xdc\xb1\xdb
\xb0\xdb\xb0\xdf\xdc\xb0\xdb\xdc\xb0\xdb\xdf\xdc\xdc\xb0\xdf\xb0\xdf\xdf\xb0\xdc\xdc\xdf\xb0\xb0\xb0\xb0\xdb\xb0\xb0\xdb\xb0
\xb0\xb0\xdb\xb0\xb0\xb0\xdf\xdc\xdf\xdb\xdc\xdc\xb0\xdb\xdf\xdf\xdf\xdc\xdc\xdc\xdc\xdf\xdf\xdb\xdf\xdb\xdb\xb0\xdb\xb0\xb0
\xb0\xb0\xb0\xdb\xb0\xb0\xb0\xb0\xdb\xdb\xb0\xb0\xdf\xdb\xdc\xdc\xdc\xdb\xdc\xdc\xdb\xdc\xdb\xdb\xdb\xdb\xb0\xdb\xb0\xb0\xb0
\xb0\xb0\xb0\xb0\xdb\xb0\xb0\xb0\xb0\xdf\xdf\xdc\xb0\xdb\xb0\xb0\xb0\xdb\xb0\xdb\xdf\xdb\xdb\xdb\xdb\xdb\xdb\xb0\xdb\xb0\xb0
\xb0\xb0\xb0\xb0\xb0\xdf\xdc\xb0\xb0\xb0\xb0\xb0\xdf\xdf\xdc\xdc\xdc\xdb\xdc\xdb\xdc\xdb\xdc\xdb\xdc\xdf\xb0\xb0\xdb\xb0\xb0
\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xdf\xdc\xdc\xb0\xb1\xb1\xb1\xb1\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb1\xb0\xb0\xb0\xdb\xb0
\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xdf\xdf\xdc\xdc\xb0\xb1\xb1\xb1\xb1\xb1\xb1\xb1\xb1\xb1\xb1\xb0\xb0\xb0\xb0\xdb\xb0
\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xdf\xdc\xdc\xdc\xdc\xdc\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xb0\xdb\xb0\xb0";

    without_interrupts(|| {
        let mut writer = WRITER.lock();

        for (i, b) in TROLLFACE.split(|b| *b == b'\n').enumerate() {
            write_bytes(writer.buffer, row + i, col, b, color);
        }
    })
}

fn printb(row: usize, col: usize, bytes: impl AsRef<[u8]>, color: CharColor) {
    without_interrupts(|| {
        write_bytes(WRITER.lock().buffer, row, col, bytes, color);
    });
}

fn clear_slowly() {
    for row in 0..BUFFER_HEIGHT {
        without_interrupts(|| {
            WRITER.lock().clear_row(row);
        });
        sleep(0.1);
    }
}

fn set_color_and_clear(color: CharColor) {
    without_interrupts(|| {
        let mut writer = WRITER.lock();
        writer.set_color(color);
    });
    clear_slowly();
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    aaos::init(boot_info);

    #[cfg(test)]
    test_main();

    log!("it did not crash");

    loop {
        {
            let color = CharColor::new(Color::White, Color::LightBlue);
            set_color_and_clear(color);
            printb(12, 35, b" \x86keOS (\xe0) ", color);
            sleep(0.5);
            printb(14, 35, b" (skolorna.com) ", color);
        }
        sleep(3.);
        set_color_and_clear(CharColor::new(Color::LightGray, Color::Black));
        print_logo(1, 23);
        sleep(5.);
        clear_slowly();
        printb(
            12,
            1,
            b" \x10\x10\x10 sodralatinsgymnasium.stockholm ",
            CharColor::new(Color::White, Color::LightRed),
        );
        sleep(3.);
    }

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
