use pic8259::ChainedPics;
use spin::Mutex;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// An **I**nterrupt **R**e**q**uest. [Wikipedia](https://en.wikipedia.org/wiki/Interrupt_request_(PC_architecture)).
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Irq {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Rtc = PIC_1_OFFSET + 8,
}

impl Irq {
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub const fn as_usize(self) -> usize {
        self.as_u8() as usize
    }
}

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init() {
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
