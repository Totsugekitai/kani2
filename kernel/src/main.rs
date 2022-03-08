#![no_std]
#![no_main]
use core::{arch::asm, panic::PanicInfo};

#[no_mangle]
pub extern "sysv64" fn kernel_main() {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}
