#![no_main]
#![no_std]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use goblin::elf::{self, ProgramHeaders};
use uefi::{
    alloc::exit_boot_services,
    prelude::*,
    proto::{self, media::file::*},
    table::boot::{AllocateType, MemoryDescriptor, MemoryType},
};

const EFI_PAGE_SIZE: usize = 0x1000;

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    let boot_services = system_table.boot_services();

    // init serial
    let serial = boot_services.locate_protocol::<proto::console::serial::Serial>();
    let serial = if serial.is_ok() {
        serial.unwrap()
    } else {
        panic!("panic");
    };
    let serial = unsafe { &mut *serial.get() };
    serial.reset().unwrap();

    serial.write(b"Hello kani2!\r\n").unwrap();

    // open root directory
    let simple_fs = boot_services.get_image_file_system(image_handle).unwrap();
    let mut root_dir = unsafe { &mut *simple_fs.interface.get() }
        .open_volume()
        .unwrap();

    // open kernel file
    let filename = cstr16!("kani2_kernel.elf");

    let kernel_file = root_dir
        .open(filename, FileMode::Read, FileAttribute::empty())
        .unwrap();
    serial.write(b"open kernel file success\r\n").unwrap();

    // read kernel file
    let kernel_file_type = kernel_file.into_type().unwrap();
    let mut kernel_file = match kernel_file_type {
        FileType::Regular(file) => file,
        FileType::Dir(_) => panic!("This is not a regular file"),
    };
    let size = kernel_file
        .get_boxed_info::<FileInfo>()
        .unwrap()
        .file_size() as usize;
    let mut buf = vec![0u8; size];
    let read_size = kernel_file.read(&mut buf).unwrap();
    if size != read_size {
        panic!("read kernel file failed");
    }
    serial.write(b"read kernel file success\r\n").unwrap();

    // parse elf file
    let kernel_elf = elf::Elf::parse(&buf).unwrap();
    let entry_point: extern "sysv64" fn() = unsafe { core::mem::transmute(kernel_elf.entry) };
    serial
        .write(
            format!(
                "parse kernel file success, entry: {:x}\r\n",
                kernel_elf.entry
            )
            .as_bytes(),
        )
        .unwrap();

    // load program headers
    let alloc_region = calc_alloc_region(&kernel_elf.program_headers);
    serial
        .write(
            format!(
                "alloc addr: {:x}, count: {}\r\n",
                alloc_region.0, alloc_region.1
            )
            .as_bytes(),
        )
        .unwrap();
    let result = boot_services.allocate_pages(
        AllocateType::Address(alloc_region.0),
        MemoryType::LOADER_DATA,
        alloc_region.1,
    );
    if let Err(e) = result {
        serial
            .write(
                format!(
                    "page allocation failed: {:x}, {}, {:?}\r\n",
                    alloc_region.0, alloc_region.1, e
                )
                .as_bytes(),
            )
            .unwrap();
        panic!();
    }

    for phdr in kernel_elf.program_headers.iter() {
        if phdr.p_type != elf::program_header::PT_LOAD {
            continue;
        }

        let vaddr = phdr.p_vaddr as usize;
        let memsize = phdr.p_memsz as usize + (vaddr % EFI_PAGE_SIZE);

        let filesize = phdr.p_filesz as usize;
        let offset = phdr.p_offset as usize;
        let dest = unsafe { core::slice::from_raw_parts_mut(vaddr as *mut u8, memsize) };
        dest[..filesize].copy_from_slice(&buf[offset..(offset + filesize)]);
        dest[filesize..].fill(0);
    }
    serial.write(b"locate kernel image success\r\n").unwrap();

    let memory_map = get_memory_map(boot_services);
    if let Err(_) = memory_map {
        serial.write(b"[ERROR]cannot get memory map\r\n").unwrap();
        panic!();
    }
    for m in &memory_map.unwrap() {
        serial
            .write(format!("{:?}: {:x}, {}\r\n", m.ty, m.phys_start, m.page_count).as_bytes())
            .unwrap();
    }

    exit_boot_services();

    entry_point();

    Status::SUCCESS
}

fn calc_alloc_region(phdrs: &ProgramHeaders) -> (usize, usize) {
    let mut memo: (usize, usize) = (0, 0);
    for phdr in phdrs.iter() {
        if phdr.p_type != elf::program_header::PT_LOAD {
            continue;
        }

        let vaddr = phdr.p_vaddr as usize;
        let alloc_top = vaddr - (vaddr % EFI_PAGE_SIZE);
        let memsize = phdr.p_memsz as usize + (vaddr % EFI_PAGE_SIZE);
        let page_count = (memsize + EFI_PAGE_SIZE - 1) / EFI_PAGE_SIZE;

        if memo.0 == 0 {
            memo.0 = alloc_top;
            memo.1 = page_count;
        } else {
            let len = page_count * EFI_PAGE_SIZE;
            if (memo.0 + memo.1 * EFI_PAGE_SIZE) < (alloc_top + len) {
                let extend_page_count =
                    ((alloc_top + len) - (memo.0 + memo.1 * EFI_PAGE_SIZE)) / EFI_PAGE_SIZE;
                memo.1 += extend_page_count;
            }
        }
    }
    memo
}

fn get_memory_map(boot_services: &BootServices) -> Result<Vec<MemoryDescriptor>, ()> {
    let mut buf: [u8; 1024 * 16] = [0; 1024 * 16];
    let map = boot_services.memory_map(&mut buf);
    if let Err(_) = map {
        return Err(());
    }
    let iter = map.unwrap().1;
    let v: Vec<MemoryDescriptor> = iter.copied().collect();
    Ok(v)
}
