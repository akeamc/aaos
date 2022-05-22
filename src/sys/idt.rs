use crate::sys::pic::{PICS, PIC_1_OFFSET};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::hlt_loop;

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

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
      let mut idt = InterruptDescriptorTable::new();
      idt.breakpoint.set_handler_fn(handle_breakpoint);
      unsafe {
        idt.double_fault.set_handler_fn(handle_double_fault)
            .set_stack_index(crate::sys::gdt::DOUBLE_FAULT_IST_INDEX); // new
    }
    idt.page_fault.set_handler_fn(handle_page_fault);

    idt[Irq::Timer.as_usize()].set_handler_fn(crate::sys::time::handle_timer_interrupt);
    idt[Irq::Keyboard.as_usize()].set_handler_fn(handle_keyboard_interrupt);

      idt
  };
}

/// Initialize the Interrupt Descriptor Table (IDT).
pub fn init() {
    IDT.load();
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
        PICS.lock().notify_end_of_interrupt(Irq::Keyboard.as_u8());
    }
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
