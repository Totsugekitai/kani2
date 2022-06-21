use crate::uart;
use lazy_static::lazy_static;
use x86_64::structures::idt::{self, InterruptDescriptorTable, InterruptStackFrame};

use crate::println;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = unsafe {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(0);

        idt[36].set_handler_fn(uart::uart_handler);

        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

pub fn notify_end_of_interrupt() {
    unsafe {
        core::ptr::write_volatile(0xFEE0_00B0 as *mut u32, 0);
    }
}
