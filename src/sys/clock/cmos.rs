use core::hint::spin_loop;

use bit_field::BitField;
use time::{Date, Month, PrimitiveDateTime, Time};
use x86_64::instructions::{interrupts, port::Port};

use crate::sys::{
    pic::{Irq, PICS},
    time::halt,
};

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

    /// Read from the CMOS registers without checking if an
    /// update is in progress, etc.
    pub fn rtc_unchecked(&mut self) -> Rtc {
        Rtc::from_registers(
            self.read_register(Register::Second),
            self.read_register(Register::Minute),
            self.read_register(Register::Hour),
            self.read_register(Register::Day),
            self.read_register(Register::Month),
            self.read_register(Register::Year),
            self.read_register(Register::B),
        )
    }

    pub fn rtc_checked(&mut self) -> Rtc {
        loop {
            self.wait_for_update();
            let prev = self.rtc_unchecked();

            self.wait_for_update();
            let new = self.rtc_unchecked();

            if prev == new {
                return new;
            }
        }
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

    /// Wait until no CMOS update is in progress.
    fn wait_for_update(&mut self) {
        while self.is_updating() {
            spin_loop();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rtc {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    /// Year (0-99)
    year: u8,
}

impl Rtc {
    fn from_registers(
        mut second: u8,
        mut minute: u8,
        mut hour: u8,
        mut day: u8,
        mut month: u8,
        mut year: u8,
        b: u8,
    ) -> Self {
        let is_bcd = !b.get_bit(2);
        if is_bcd {
            second = (second & 0x0F) + ((second / 16) * 10);
            minute = (minute & 0x0F) + ((minute / 16) * 10);
            hour = ((hour & 0x0F) + (((hour & 0x70) / 16) * 10)) | (hour & 0x80);
            day = (day & 0x0F) + ((day / 16) * 10);
            month = (month & 0x0F) + ((month / 16) * 10);
            year = (year & 0x0F) + ((year / 16) * 10);
        }

        let is_12h = !b.get_bit(1) && hour.get_bit(7);
        if is_12h {
            hour = ((hour & 0x7f) + 12) % 24;
        }

        Self {
            second,
            minute,
            hour,
            day,
            month,
            year,
        }
    }

    fn date(&self) -> Date {
        let century = if self.year < 70 { 2000 } else { 1900 };

        Date::from_calendar_date(
            self.year as i32 + century,
            self.month.try_into().expect("invalid month"),
            self.day,
        )
        .expect("invalid date range")
    }

    fn time(&self) -> Time {
        Time::from_hms(self.hour, self.minute, self.second).expect("invalid time")
    }

    pub fn datetime(&self) -> PrimitiveDateTime {
        PrimitiveDateTime::new(self.date(), self.time())
    }
}

#[test_case]
fn rtc_bcd() {
    assert_eq!(
        Rtc::from_registers(
            0b01010001, // 5 << 4 | 1
            0b00001001, // 0 << 4 | 9
            0b00110001, // 3 << 4 | 1
            0b00110001, // 3 << 4 | 1
            0b00000011, // 0 << 4 | 3
            0b01110000, // 7 << 4 | 0
            0x0
        ),
        Rtc {
            second: 51,
            minute: 9,
            hour: 31,
            day: 31,
            month: 3,
            year: 70
        }
    )
}
