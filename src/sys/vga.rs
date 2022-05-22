use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use vte::{Params, Parser, Perform};

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;
const FG: Color = Color::LightGray;
const BG: Color = Color::Black;

lazy_static! {
    pub static ref PARSER: Mutex<Parser> = Mutex::new(Parser::new());
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
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
    pub fn from_ansi(code: u8) -> Self {
        use Color::*;

        match code {
            30 => Black,
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
    pub fn new(ascii_character: u8, color_code: CharColor) -> Self {
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
    pub column_position: usize,
    pub color_code: CharColor,
    pub buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let c = ScreenChar::new(byte, self.color_code);

                unsafe { core::ptr::write_volatile(&mut self.buffer.chars[row][col], c) };
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let c = self.buffer.chars[row][col];

                unsafe { core::ptr::write_volatile(&mut self.buffer.chars[row - 1][col], c) };
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
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

    fn set_color(&mut self, color: CharColor) {
        self.color_code = color;
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
            'A' => {}
            'B' => {}
            'C' => {}
            'D' => {}
            'E' => {}
            'F' => {}
            'G' => {}
            'J' => {}
            'K' => {}
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

        Ok(())
    }
}

pub fn print_fmt(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

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
