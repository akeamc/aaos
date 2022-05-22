use crate::sys;
use lazy_static::lazy_static;
use sys::pic::Irq;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::hlt_loop;

lazy_static! {
  static ref IDT: InterruptDescriptorTable = {
      let mut idt = InterruptDescriptorTable::new();
      idt.breakpoint.set_handler_fn(handle_breakpoint);
      unsafe {
        idt.double_fault.set_handler_fn(handle_double_fault)
            .set_stack_index(sys::gdt::DOUBLE_FAULT_IST_INDEX); // new
    }
    idt.page_fault.set_handler_fn(handle_page_fault);

    idt[Irq::Timer.as_usize()].set_handler_fn(sys::time::handle_timer_interrupt);
    idt[Irq::Keyboard.as_usize()].set_handler_fn(sys::keyboard::handle_interrupt);

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

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
