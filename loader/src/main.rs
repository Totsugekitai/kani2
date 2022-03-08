#![no_main]
#![no_std]
#![feature(abi_efiapi)]

#[macro_use]
extern crate alloc;

use goblin::elf;
use uefi::{
    alloc::exit_boot_services,
    prelude::*,
    proto::{self, media::file::*},
    table::boot::{AllocateType, MemoryType},
    ResultExt,
};

const EFI_PAGE_SIZE: usize = 0x1000;
const FILENAME: &str = "kani2_kernel.elf";

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap_success();

    let boot_services = system_table.boot_services();

    // init serial
    let serial = boot_services.locate_protocol::<proto::console::serial::Serial>();
    let serial = if serial.is_ok() {
        serial.unwrap().unwrap()
    } else {
        panic!("panic");
    };
    let serial = unsafe { &mut *serial.get() };
    serial.reset().unwrap_success();

    serial.write(b"Hello kani2!").unwrap_success();

    // open root directory
    let simple_fs = boot_services
        .get_image_file_system(image_handle)
        .unwrap()
        .unwrap();
    let mut root_dir = unsafe { &mut *simple_fs.interface.get() }
        .open_volume()
        .unwrap()
        .unwrap();

    // open kernel file
    let mut kernel_file = root_dir
        .open(FILENAME, FileMode::Read, FileAttribute::empty())
        .unwrap_success();

    // read kernel file
    let size = kernel_file
        .get_boxed_info::<FileInfo>()
        .unwrap_success()
        .file_size() as usize;
    let mut buf = vec![0u8; size];
    let kernel_file = kernel_file.into_type().unwrap().unwrap();
    let mut kernel_file = match kernel_file {
        FileType::Regular(file) => file,
        FileType::Dir(_) => panic!("This is not a regular file"),
    };
    let read_size = kernel_file.read(&mut buf).unwrap().unwrap();
    if size != read_size {
        panic!("read kernel file failed");
    }

    // parse elf file
    let kernel_elf = elf::Elf::parse(&buf).unwrap();
    let entry_point: extern "sysv64" fn() = unsafe { core::mem::transmute(kernel_elf.entry) };

    // set program headers
    for phdr in kernel_elf.program_headers.iter() {
        if phdr.p_type != elf::program_header::PT_LOAD {
            continue;
        }

        let vaddr = phdr.p_vaddr as usize;
        let memsize = phdr.p_memsz as usize;
        let page_count = (memsize + EFI_PAGE_SIZE - 1) / EFI_PAGE_SIZE;
        boot_services
            .allocate_pages(
                AllocateType::Address(vaddr),
                MemoryType::LOADER_DATA,
                page_count,
            )
            .expect_success("page allocation failed");

        let filesize = phdr.p_filesz as usize;
        let offset = phdr.p_offset as usize;
        let dest = unsafe { core::slice::from_raw_parts_mut(vaddr as *mut u8, memsize) };
        dest[..filesize].copy_from_slice(&buf[offset..(offset + filesize)]);
        dest[filesize..].fill(0);
    }

    exit_boot_services();

    entry_point();

    Status::SUCCESS
}
