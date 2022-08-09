use bitmaps::Bitmap;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{OffsetPageTable, PageTable, PageTableFlags, PhysFrame},
    PhysAddr, VirtAddr,
};

extern "C" {
    static __kernel_pagetable_pml4: u8;
    static __kernel_pagetable_pdpt: u8;
    static __kernel_pagetable_pd: u8;
}

pub fn init() {
    init_kernel_page_table();
    let _ = PageFrameManager::new();
}

fn init_kernel_page_table() {
    unsafe {
        let frame = construct_kernel_page_table();
        Cr3::write(frame, Cr3Flags::empty());
    }
}

unsafe fn construct_kernel_page_table() -> PhysFrame {
    let pml4 = (&__kernel_pagetable_pml4 as *const u8 as *mut PageTable)
        .as_mut()
        .unwrap();
    let pml4_addr = &__kernel_pagetable_pml4 as *const u8 as *const PageTable;

    let pdpt = (&__kernel_pagetable_pdpt as *const u8 as *mut PageTable)
        .as_mut()
        .unwrap();
    let pdpt_addr = &__kernel_pagetable_pdpt as *const u8 as *const PageTable;

    let pds = (&__kernel_pagetable_pd as *const u8 as *mut [PageTable; 4])
        .as_mut()
        .unwrap();
    let pds_addrs = &__kernel_pagetable_pd as *const u8 as *const [PageTable; 4];

    pml4.zero();
    pdpt.zero();
    for pd in pds.iter_mut() {
        pd.zero();
    }

    // PML4
    pml4[0].set_addr(
        PhysAddr::new(pdpt_addr as u64),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );

    // PDPT
    for i in 0..4 {
        pdpt[i].set_addr(
            PhysAddr::new(&pds[i] as *const PageTable as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );
    }

    // PD identity mapping 4GiB (2MiB * 512 * 4)
    const PAGE_SIZE: u64 = 2097152; // 2MiB
    for i_pd in 0..4 {
        for i in 0..512 {
            pds[i_pd][i].set_addr(
                PhysAddr::new(PAGE_SIZE * (i + i_pd * 512) as u64),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE,
            );
        }
    }

    PhysFrame::containing_address(PhysAddr::new(pml4_addr as u64))
}

/// ビット1つにつき2MiBで、2MiB * 512 * 4 = 4GiB
#[derive(Debug)]
pub struct PageFrameManager {
    frame_map: [Bitmap<512>; 4],
}

impl PageFrameManager {
    pub fn new() -> Self {
        // ページテーブルを読む
        let (phys_frame, _) = Cr3::read();
        let pml4 = unsafe {
            (phys_frame.start_address().as_u64() as *const PageTable)
                .as_ref()
                .unwrap()
        };
        let pdpt = unsafe {
            (pml4[0].addr().as_u64() as *const PageTable)
                .as_ref()
                .unwrap()
        };
        let pds = unsafe {
            (pdpt[0].addr().as_u64() as *const [PageTable; 4])
                .as_ref()
                .unwrap()
        };

        let mut frame_map = [Bitmap::new(); 4];
        for i in 0..4 {
            let pd = &pds[i];
            let bitmap = &mut frame_map[i];
            for j in 0..512 {
                let flags = pd[j].flags();
                if flags.contains(PageTableFlags::DIRTY) {
                    bitmap.set(j, true);
                }
            }
        }

        Self { frame_map }
    }
}
