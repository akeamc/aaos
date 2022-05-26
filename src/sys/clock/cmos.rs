use bit_field::BitField;
use x86_64::instructions::{interrupts, port::Port};

use crate::sys::pic::{Irq, PICS};

#[repr(u8)]
enum Register {
    Second = 0x00,
    Minute = 0x02,
    Hour = 0x04,
    Day = 0x07,
    Month = 0x08,
    Year = 0x09,
    A = 0x0a,
    B = 0x0b,
    C = 0x0c,
}

#[repr(u8)]
enum Interrupt {
    Periodic = 1 << 6,
    Alarm = 1 << 5,
    Update = 1 << 4,
}

pub struct Cmos {
    addr: Port<u8>,
    data: Port<u8>,
}

impl Cmos {
    pub const fn new() -> Self {
        Self {
            addr: Port::new(0x70),
            data: Port::new(0x71),
        }
    }

    pub fn enable_update_interrupt(&mut self) {
        self.enable_interrupt(Interrupt::Update)
    }

    fn enable_interrupt(&mut self, interrupt: Interrupt) {
        interrupts::without_interrupts(|| {
            self.set_nmi(false);
            unsafe {
                self.select_register(Register::B);
                let prev = self.data.read();
                self.select_register(Register::B);
                self.data.write(prev | interrupt as u8);
            }
        });
    }

    fn rtc_unchecked(&mut self) -> Rtc {
        Rtc {
            second: self.read_register(Register::Second),
            minute: self.read_register(Register::Minute),
            hour: self.read_register(Register::Hour),
            day: self.read_register(Register::Day),
            month: self.read_register(Register::Month),
            year: self.read_register(Register::Year),
        }
    }

    pub fn rtc(&mut self) -> Rtc {
        let mut rtc = self.rtc_unchecked();

        let bcd = !self.read_register(Register::B).get_bit(4);
        if bcd {
            rtc = rtc.into_binary();
        }

        rtc
    }

    pub fn notify_end_of_interrupt(&mut self) {
        unsafe {
            self.select_register(Register::C);
            self.data.read();
            PICS.lock().notify_end_of_interrupt(Irq::Rtc.as_u8());
        }
    }

    #[inline]
    fn set_nmi(&mut self, enabled: bool) {
        unsafe {
            let mut prev = self.addr.read();
            prev.set_bit(7, enabled);
            self.addr.write(prev);
        }
    }

    fn read_register(&mut self, register: Register) -> u8 {
        unsafe {
            self.select_register(register);
            self.data.read()
        }
    }

    #[inline]
    unsafe fn select_register(&mut self, register: Register) {
        self.addr.write(register as u8);
    }

    /// Check if a CMOS RTC update is in progress.
    /// [OSDev.org](https://wiki.osdev.org/CMOS#RTC_Update_In_Progress)
    fn is_updating(&mut self) -> bool {
        unsafe {
            self.select_register(Register::A);
            self.data.read().get_bit(7)
        }
    }
}

#[derive(Debug)]
pub struct Rtc {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u8,
}

impl Rtc {
    fn into_binary(self) -> Self {
        todo!();
    }
}
