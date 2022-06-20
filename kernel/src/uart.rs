use alloc::sync::Arc;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::{
    interrupts::without_interrupts,
    port::{PortReadOnly, PortWriteOnly},
};

const COM1: u16 = 0x3f8;

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
        });
    }

    #[cfg(not(feature = "qemu"))]
    pub unsafe fn write(&self, c: u8) {
        while PortReadOnly::<u16>::new(self.com + 5).read() & 0x20 != 0x20 {
            x86_64::instructions::hlt();
        }
        PortWriteOnly::<u8>::new(self.com).write(c);
    }

    #[cfg(feature = "qemu")]
    pub unsafe fn write(&self, c: u8) {
        PortWriteOnly::<u8>::new(self.com).write(c);
    }

    #[cfg(not(feature = "qemu"))]
    pub unsafe fn read(&self) -> u8 {
        while PortReadOnly::<u16>::new(self.com + 5).read() & 1 != 1 {
            x86_64::instructions::hlt();
        }

        PortReadOnly::<u16>::new(self.com).read() as u8
    }

    #[cfg(feature = "qemu")]
    pub unsafe fn read(&self) -> u8 {
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
