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
}

impl Irq {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init() {
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
