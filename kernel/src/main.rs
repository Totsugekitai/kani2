#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

mod allocator;
mod println;
mod uart;

use core::panic::PanicInfo;

#[link_section = ".text.main"]
#[no_mangle]
pub extern "sysv64" fn kernel_main() -> ! {
    allocator::init();
    uart::init();
    println!("hello kani2 kernel hoge hoge");
    println!("hello kani2 kernel fuga fuga");
    println!("hello kani2 kernel foo foo");
    println!("hello kani2 kernel bar bar");
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
