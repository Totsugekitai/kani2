use crate::{interrupt, ioapic, print};
use alloc::sync::Arc;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    instructions::{
        interrupts::without_interrupts,
        port::{PortReadOnly, PortWriteOnly},
    },
    structures::idt::InterruptStackFrame,
};

pub const COM1: u16 = 0x3f8;
pub const COM2: u16 = 0x2f8;
pub const COM3: u16 = 0x3e8;
pub const COM4: u16 = 0x2e8;

const IRQ_COM1: u32 = 4;
const IRQ_COM2: u32 = 3;

lazy_static! {
    pub static ref UART: Arc<Mutex<Uart>> = Arc::new(Mutex::new(Uart { com: COM1 }));
}

pub struct Uart {
    com: u16,
}

impl Uart {
    unsafe fn init(&self) {
        without_interrupts(|| {
            // 8259 PIC Disable
            PortWriteOnly::<u8>::new(0xa1).write(0xff);
            PortWriteOnly::<u8>::new(0x21).write(0xff);
            // 16550A UART Enable
            PortWriteOnly::<u8>::new(self.com + 1).write(0); // disable all interrupts
            PortWriteOnly::<u8>::new(self.com + 3).write(0x80); // DLAB set 1
            PortWriteOnly::<u8>::new(self.com).write(1); // 115200 / 115200
            PortWriteOnly::<u8>::new(self.com + 1).write(0); // baud rate hi bytes
            PortWriteOnly::<u8>::new(self.com + 3).write(0x03); // DLAB set 0
            PortWriteOnly::<u8>::new(self.com + 4).write(0x0b); // IRQ enable
            PortWriteOnly::<u8>::new(self.com + 1).write(0x01); // interrupt enable

            if PortReadOnly::<u16>::new(self.com + 5).read() == 0xff {
                panic!();
            }

            PortReadOnly::<u16>::new(self.com + 2).read();
            PortReadOnly::<u16>::new(self.com).read();

            ioapic::enable(IRQ_COM1, 0);
        });
    }

    pub unsafe fn write(&self, c: u8) {
        if !cfg!(feature = "qemu") {
            while PortReadOnly::<u16>::new(self.com + 5).read() & 0x20 != 0x20 {
                x86_64::instructions::nop(); // タイマー割り込みがない場合はnopにしないと止まる
            }
        }
        PortWriteOnly::<u8>::new(self.com).write(c);
    }

    pub unsafe fn read(&self) -> u8 {
        if !cfg!(feature = "qemu") {
            while PortReadOnly::<u16>::new(self.com + 5).read() & 1 != 1 {
                x86_64::instructions::nop(); // タイマー割り込みがない場合はnopにしないと止まる
            }
        }

        PortReadOnly::<u16>::new(self.com).read() as u8
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            unsafe {
                self.write(c);
            }
        }
        Ok(())
    }
}

pub fn init() {
    unsafe {
        UART.lock().init();
    }
}

pub extern "x86-interrupt" fn uart_handler(_: InterruptStackFrame) {
    let mut c = b'\0';
    without_interrupts(|| unsafe {
        c = UART.lock().read();
        interrupt::notify_end_of_interrupt();
        print!("{}", c as char);
    });
}

pub fn remove_screen() {
    let mut uart = UART.lock();
    let _ = uart.write_str("\x1b[2J\x1b[1;1H");
}
