use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::hlt_loop;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
      let mut idt = InterruptDescriptorTable::new();
      idt.breakpoint.set_handler_fn(handle_breakpoint);
      unsafe {
        idt.double_fault.set_handler_fn(handle_double_fault)
            .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX); // new
    }
    idt.page_fault.set_handler_fn(handle_page_fault);

    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(handle_timer_interrupt);
    idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(handle_keyboard_interrupt);

      idt
  };
}

/// Set the divisor of the PIT.
///
/// # Panics
///
/// In debug mode, this will panic if the divisor becomes greater than [`u16::MAX`].
/// `hz` must not be less than `1_193_180.0 / 65_535.0` (about 18.207).
fn set_pit_phase(hz: f32) {
    use x86_64::instructions::port::Port;

    let divisor = 1193180.0 / hz;

    debug_assert!(
        divisor <= u16::MAX as f32,
        "divisor ({}) is too big",
        divisor
    );

    unsafe { Port::<u8>::new(0x43).write(0x36) }; // set command byte 0x36

    let mut port = Port::new(0x40);

    for b in (divisor as u16).to_le_bytes() {
        unsafe { port.write(b) };
    }
}

/// Initialize the Interrupt Descriptor Table (IDT).
pub fn init_idt() {
    IDT.load();

    set_pit_phase(60.0);
}

extern "x86-interrupt" fn handle_breakpoint(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn handle_double_fault(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame)
}

extern "x86-interrupt" fn handle_page_fault(
    stack_fame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("accessed address: {:?}", Cr2::read());
    println!("error code: {:?}", error_code);
    println!("{:#?}", stack_fame);

    hlt_loop();
}

extern "x86-interrupt" fn handle_timer_interrupt(_stack_frame: InterruptStackFrame) {
    print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8())
    }
}

extern "x86-interrupt" fn handle_keyboard_interrupt(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::RawKey(key) => print!("{:?}", key),
                DecodedKey::Unicode(character) => print!("{}", character),
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
