use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

extern "C" {
    static __kernel_heap: u8;
    static __kernel_heap_end: u8;
}

pub fn init() {
    unsafe {
        let kernel_heap = &__kernel_heap as *const u8 as usize;
        let kernel_heap_end = &__kernel_heap_end as *const u8 as usize;
        let length = kernel_heap_end - kernel_heap;
        ALLOCATOR.lock().init(kernel_heap as *mut u8, length);
    }
}
