use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use morok_reactive::Runtime;

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);

struct CountingAllocator;

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

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

const WARM_UP_ROUNDS: i64 = 8;
const MEASURED_ROUNDS: i64 = 1000;
const EFFECTS: i64 = 64;

/// A diamond over one signal read by many effects, a branch that switches its source every
/// round, and an effect whose write wakes another effect in the same flush.
fn main() {
    let mut runtime: Runtime = Runtime::new();
    let total = Rc::new(Cell::new(0_i64));
    let base = runtime.signal(0_i64);
    let other = runtime.signal(0_i64);
    let mirror = runtime.signal(0_i64);
    let next = runtime.memo(move |runtime| runtime.get(base) + 1);
    let doubled = runtime.memo(move |runtime| runtime.get(base) * 2);
    let sum = runtime.memo(move |runtime| runtime.get(next) + runtime.get(doubled));
    for _ in 0..EFFECTS {
        let total = Rc::clone(&total);
        runtime.effect(move |runtime, ()| total.set(total.get() + runtime.get(sum)));
    }
    let branch_total = Rc::clone(&total);
    runtime.effect(move |runtime, ()| {
        let chosen = if runtime.get(base) % 2 == 0 { other } else { mirror };
        branch_total.set(branch_total.get() + runtime.get(chosen));
    });
    runtime.effect(move |runtime, ()| {
        let value = runtime.get(base);
        runtime.set(mirror, value);
    });

    let round = |runtime: &mut Runtime, value: i64| {
        runtime.set(base, value);
        runtime.set(other, -value);
        runtime.flush(&mut ()).expect("the graph has no cycle");
    };
    for value in 0..WARM_UP_ROUNDS {
        round(&mut runtime, value);
    }
    let total_before = total.get();
    let allocations_before = ALLOCATIONS.load(Ordering::Relaxed);
    for value in WARM_UP_ROUNDS..WARM_UP_ROUNDS + MEASURED_ROUNDS {
        round(&mut runtime, value);
    }
    let allocations = ALLOCATIONS.load(Ordering::Relaxed) - allocations_before;

    assert_ne!(total.get(), total_before, "the measured rounds ran the effects");
    assert_eq!(allocations, 0, "a write and a flush on a built graph allocate nothing");
    println!("steady_allocations: 0 allocations in {MEASURED_ROUNDS} rounds of writes and a flush");
}
