use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{
    registers::segmentation::{Segment, SegmentSelector, CS, DS},
    structures::gdt::{Descriptor, GlobalDescriptorTable},
    PrivilegeLevel,
};

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code = gdt.add_entry(Descriptor::kernel_code_segment());
        let data = gdt.add_entry(Descriptor::kernel_data_segment());
        gdt.add_entry(Descriptor::user_code_segment());
        gdt.add_entry(Descriptor::user_data_segment());
        (gdt, Selectors { code, data })
    };
}

struct Selectors {
    code: SegmentSelector,
    data: SegmentSelector,
}

pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code);
        DS::set_reg(GDT.1.data);
    }
}
