#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

mod allocator;
mod gdt;
mod interrupt;
mod ioapic;
mod memory;
mod println;
mod task;
mod uart;

use core::{arch::asm, panic::PanicInfo};
use kani2_common::boot::{BootInfo, MemoryMap};

extern "C" {
    static mut __kernel_stack: u8;
}

#[link_section = ".text.main"]
#[no_mangle]
pub extern "sysv64" fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // スタックポインタをカーネルのものにする
    unsafe {
        asm!(
            "mov rsp, {stack_bottom}",
            stack_bottom = in(reg) &__kernel_stack as *const u8 as usize,
        );
    }

    // 初期化
    init();

    println!("[info]hello kani2 kernel");

    unsafe {
        for m in boot_info.mmap().into_iter() {
            let m = m.as_ref().unwrap();
            println!(
                "{:?}: 0x{:016x} - {} page",
                m.ty, m.phys_start, m.page_count
            );
        }
    }

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

fn init() {
    allocator::init();
    gdt::init();
    interrupt::init();
    uart::init();
    memory::init();
}
