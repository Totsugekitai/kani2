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
    println!("[info]hello kani2 kernel");
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{:?}", info);
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}