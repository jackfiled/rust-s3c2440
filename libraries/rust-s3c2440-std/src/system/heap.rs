use crate::error;
use buddy_system_allocator::Heap;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::NonNull;

/// Heap size.
/// Set to 10M
const HEAP_SIZE: usize = 10 * 1024 * 1024;

const BUDDY_MAX_ORDER: usize = 32;

#[unsafe(link_section = ".bss.heap")]
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// A very dangerous heap for global allocator.
/// But the S3C2440 only has a core, so it is much safer.
pub struct UnsafeHeap(UnsafeCell<Heap<BUDDY_MAX_ORDER>>);

impl UnsafeHeap {
    const fn empty() -> Self {
        Self(UnsafeCell::new(Heap::empty()))
    }

    fn heap(&self) -> &mut Heap<BUDDY_MAX_ORDER> {
        unsafe { &mut (*self.0.get()) }
    }
}

unsafe impl GlobalAlloc for UnsafeHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.heap()
            .alloc(layout)
            .ok()
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.heap().dealloc(NonNull::new_unchecked(ptr), layout) }
    }
}

unsafe impl Sync for UnsafeHeap {}

#[global_allocator]
static GLOBAL_ALLOCATOR: UnsafeHeap = UnsafeHeap::empty();

pub fn initialize_heap() {
    unsafe {
        let start = HEAP.as_ptr() as usize;
        GLOBAL_ALLOCATOR
            .heap()
            .add_to_heap(start, start + HEAP_SIZE);
    }
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    error!("Heap stats:");
    {
        let heap = GLOBAL_ALLOCATOR.heap();
        error!("\tTotal size: {}", heap.stats_total_bytes());
        error!("\tRequested size: {}", heap.stats_alloc_user());
        error!("\tAllocated size: {}", heap.stats_alloc_actual());
        error!(
            "Currently the heap only support allocate buffer with max length {} bytes.",
            1 << (BUDDY_MAX_ORDER - 1)
        );
    }
    panic!("Heap allocation error, layout = {:?}", layout);
}
