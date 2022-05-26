#![allow(clippy::cast_possible_truncation)]
use core::fmt;

use bit_field::BitField;
use lazy_static::lazy_static;
use spin::Mutex;
use vte::{Params, Parser, Perform};
use x86_64::instructions::{interrupts, port::Port};

pub mod font;

use font::Font;

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;
const FG: Color = Color::LightGray;
const BG: Color = Color::Black;

const SEQUENCER_ADDR_REG: u16 = 0x3c4;
const GRAPHICS_ADDR_REG: u16 = 0x3ce;
const INPUT_STATUS_REG: u16 = 0x3da;
const CRTC_REG: u16 = 0x3d4;

lazy_static! {
    static ref PARSER: Mutex<Parser> = Mutex::new(Parser::new());
    /// Remember to disable interrupts when the lock is held,
    /// as an unexpected interrupt might result in a deadlock.
    static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        writer_position: (0, 0),
        cursor_position: (0, 0),
        color_code: CharColor::new(FG, BG),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    Pink,
    Yellow,
    White,
}

impl Color {
    #[must_use]
    pub const fn from_ansi(code: u8) -> Self {
        use Color::{
            Black, Blue, Brown, Cyan, DarkGray, Green, LightBlue, LightCyan, LightGray, LightGreen,
            LightRed, Magenta, Pink, Red, White, Yellow,
        };

        match code {
            // 30 => Black,
            31 => Red,
            32 => Green,
            33 => Brown,
            34 => Blue,
            35 => Magenta,
            36 => Cyan,
            37 => LightGray,
            90 => DarkGray,
            91 => LightRed,
            92 => LightGreen,
            93 => Yellow,
            94 => LightBlue,
            95 => Pink,
            96 => LightCyan,
            97 => White,
            _ => Black, // fallback
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct CharColor(pub u8);

impl CharColor {
    #[must_use]
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    ascii_character: u8,
    color_code: CharColor,
}

impl ScreenChar {
    #[must_use]
    pub const fn new(ascii_character: u8, color_code: CharColor) -> Self {
        Self {
            ascii_character,
            color_code,
        }
    }
}

#[repr(transparent)]
pub struct Buffer {
    pub chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    pub writer_position: (usize, usize),
    cursor_position: (usize, usize),
    pub color_code: CharColor,
    pub buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x08 => {
                // backspace
                if self.writer_position.0 > 0 {
                    let c = ScreenChar {
                        ascii_character: b' ',
                        color_code: self.color_code,
                    };
                    self.writer_position.0 -= 1;
                    let (col, row) = self.writer_position;
                    unsafe { core::ptr::write_volatile(&mut self.buffer.chars[row][col], c) };
                }
            }
            byte => {
                if self.writer_position.0 >= BUFFER_WIDTH {
                    self.new_line();
                }

                let (col, row) = self.writer_position;
                let c = ScreenChar::new(byte, self.color_code);

                unsafe { core::ptr::write_volatile(&mut self.buffer.chars[row][col], c) };
                self.writer_position.0 += 1;
            }
        }
    }

    fn new_line(&mut self) {
        if self.writer_position.1 < BUFFER_HEIGHT - 1 {
            self.writer_position.1 += 1;
        } else {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let c = self.buffer.chars[row][col];

                    unsafe { core::ptr::write_volatile(&mut self.buffer.chars[row - 1][col], c) };
                }
            }
            self.clear_row(self.writer_position.1);
        }
        self.writer_position.0 = 0;
        self.cursor_at_writer();
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            unsafe { core::ptr::write_volatile(&mut self.buffer.chars[row][col], blank) };
        }
    }

    fn clear_screen(&mut self) {
        for i in 0..BUFFER_HEIGHT {
            self.clear_row(i);
        }
    }

    fn set_color(&mut self, color: CharColor) {
        self.color_code = color;
    }

    #[allow(clippy::unused_self)]
    fn set_font(&mut self, font: &Font) {
        let mut sequencer = Port::<u16>::new(SEQUENCER_ADDR_REG);
        let mut graphics = Port::<u16>::new(GRAPHICS_ADDR_REG);
        let buffer = 0xA0000 as *mut u8;

        unsafe {
            sequencer.write(0x0100); // do a sync reset
            sequencer.write(0x0402); // write plane 2 only
            sequencer.write(0x0704); // sequetial access
            sequencer.write(0x0300); // end the reset
            graphics.write(0x0204); // read plane 2 only
            graphics.write(0x0005); // disable odd/even
            graphics.write(0x0006); // VRAM at 0xA0000

            for i in 0..font.size as usize {
                for j in 0..font.height as usize {
                    let vga_offset = j + i * 32;
                    let fnt_offset = j + i * font.height as usize;
                    buffer.add(vga_offset).write_volatile(font.data[fnt_offset]);
                }
            }

            sequencer.write(0x0100); // do a sync reset
            sequencer.write(0x0302); // write plane 0 & 1
            sequencer.write(0x0304); // even/odd access
            sequencer.write(0x0300); // end the reset
            graphics.write(0x0004); // restore to default
            graphics.write(0x1005); // resume odd/even
            graphics.write(0x0E06); // VRAM at 0xB800
        }
    }

    fn write_cursor(&mut self) {
        let pos = (self.cursor_position.1 * BUFFER_WIDTH + self.cursor_position.0) as u16;
        let bytes = pos.to_le_bytes();

        let mut addr = Port::<u8>::new(CRTC_REG);
        let mut data = Port::<u8>::new(CRTC_REG + 1);

        unsafe {
            addr.write(0x0f);
            data.write(bytes[0]);
            addr.write(0x0e);
            data.write(bytes[1]);
        };
    }

    pub fn set_cursor_position(&mut self, x: usize, y: usize) {
        self.cursor_position = (x, y);
        self.write_cursor();
    }

    pub fn cursor_at_writer(&mut self) {
        let (x, y) = self.writer_position;
        self.set_cursor_position(x, y);
    }
}

impl Perform for Writer {
    fn print(&mut self, c: char) {
        self.write_byte(c as u8);
    }

    fn execute(&mut self, byte: u8) {
        self.write_byte(byte);
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        #[allow(clippy::single_match)]
        match c {
            'm' => {
                let mut fg = FG;
                let mut bg = BG;

                for param in params {
                    match param[0] {
                        0 => {
                            // reset
                            fg = FG;
                            bg = BG;
                        }
                        30..=37 | 90..=97 => fg = Color::from_ansi(param[0] as u8),
                        40..=47 | 100..=107 => bg = Color::from_ansi(param[0] as u8 - 10),
                        _ => {}
                    }
                }

                self.set_color(CharColor::new(fg, bg));
            }
            // 'A' => {}
            // 'B' => {}
            // 'C' => {}
            // 'D' => {}
            // 'E' => {}
            // 'F' => {}
            // 'G' => {}
            // 'J' => {}
            // 'K' => {}
            _ => {}
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut parser = PARSER.lock();
        for byte in s.bytes() {
            parser.advance(self, byte);
        }
        self.cursor_at_writer();

        Ok(())
    }
}

/// Enable or disable VGA text blink.
fn set_blink(enabled: bool) {
    let mut input = Port::<u8>::new(INPUT_STATUS_REG);
    let mut attr = Port::<u8>::new(0x3c0);
    let mut data = Port::<u8>::new(0x3c1);

    unsafe { input.read() }; // reset ISR to address mode
    unsafe { attr.write(0x10 | 0x20) }; // select attribute mode control register
    let mut flags = unsafe { data.read() };
    flags.set_bit(3, enabled);
    unsafe { attr.write(flags) };
}

pub fn set_font(font: &Font) {
    interrupts::without_interrupts(|| {
        WRITER.lock().set_font(font);
    });
}

pub fn init() {
    interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen();
    });
    set_blink(false);
    set_font(&font::IBM_BIOS);
}

/// # Panics
///
/// Failing to write to the buffer will result
/// in a panic.
pub fn print_fmt(args: fmt::Arguments) {
    use core::fmt::Write;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::sys::vga::print_fmt(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i];
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
