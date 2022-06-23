use uefi::table::boot::MemoryDescriptor;

#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    mmap: MemoryMap,
}

impl BootInfo {
    pub fn new(mmap: MemoryMap) -> Self {
        Self { mmap }
    }

    pub fn mmap(&self) -> &MemoryMap {
        &self.mmap
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    mem_desc: *const MemoryDescriptor,
    len: u64,
}

impl MemoryMap {
    pub fn new(mem_desc: *const MemoryDescriptor, len: u64) -> Self {
        Self { mem_desc, len }
    }
}

impl Iterator for MemoryMap {
    type Item = *const MemoryDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len != 0 {
            let ptr = self.mem_desc;
            self.len -= 1;
            self.mem_desc = ((self.mem_desc as usize) + core::mem::size_of::<MemoryDescriptor>())
                as *const MemoryDescriptor;
            Some(ptr)
        } else {
            None
        }
    }
}
