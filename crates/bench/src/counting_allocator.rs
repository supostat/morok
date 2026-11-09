use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);

/// The system allocator with a count of the allocations made through it. A frame that holds
/// its allocation budget leaves the count where it found it.
#[derive(Debug, Default)]
pub struct CountingAllocator;

impl CountingAllocator {
    /// How many allocations the process has made so far; the difference of two readings
    /// counts the allocations made between them.
    pub fn allocations_so_far() -> u64 {
        ALLOCATIONS.load(Ordering::Relaxed)
    }
}

// SAFETY: every method hands the request to the system allocator unchanged and only counts it.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the layout is the caller's, under the contract of this method.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the layout is the caller's, under the contract of this method.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: the pointer and the layout are those of an allocation this allocator made.
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the pointer and the layout are those of an allocation this allocator made,
        // and the new size is the caller's.
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}
