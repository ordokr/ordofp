//! Allocation-count regression guard for the pfds `Borrow`-generic lookup path.
//!
//! Wraps the global allocator in a counting shim so "removed an allocation" is a
//! hard number, not a wall-time vibe. Proves the `&str` lookup on
//! `OrdMap<String,_>` / `OrdSet<String>` is allocation-free, while the old
//! owned-key pattern (`get(&name.to_string())`) allocates once per call.
//!
//! Historically this guarded a win for a now-retired UI consumer's
//! form-state lookups; it remains as a general regression guard for any
//! string-keyed `OrdMap`/`OrdSet` consumer doing per-call `&str` lookups.

use ordofp_core::pfds::{OrdMap, OrdSet};
use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCS: AtomicUsize = AtomicUsize::new(0);

struct Counting;

// SAFETY: forwards every call to the System allocator unchanged; the only added
// behavior is a relaxed counter bump on allocating calls.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

const ITERS: usize = 1000;

fn build_map() -> OrdMap<String, String> {
    let mut m = OrdMap::new();
    for i in 0..32 {
        m = m.insert(format!("field_{i:03}"), format!("value_{i}"));
    }
    m
}

// NOTE: a single `#[test]` fn on purpose. The allocation counter is a global
// shared across the binary; `cargo test` runs multiple `#[test]` fns on parallel
// threads, so a second test fn here would pollute these counts. Keeping one fn
// (this binary is its own process) makes the measurement deterministic without
// needing `--test-threads=1`.
#[test]
fn pfds_borrow_lookup_is_alloc_free() {
    let map = build_map();
    let _ = black_box(map.get("field_016")); // warm + sanity
    assert!(map.get("field_016").is_some());

    // OrdMap::get via Borrow path: look up by &str — must allocate nothing.
    let before = ALLOCS.load(Ordering::Relaxed);
    for _ in 0..ITERS {
        black_box(map.get(black_box("field_016")));
    }
    let str_allocs = ALLOCS.load(Ordering::Relaxed) - before;

    // Old forced pattern: caller materializes an owned String key per lookup.
    let before = ALLOCS.load(Ordering::Relaxed);
    for _ in 0..ITERS {
        black_box(map.get(&black_box("field_016").to_string()));
    }
    let owned_allocs = ALLOCS.load(Ordering::Relaxed) - before;

    // OrdSet::contains via Borrow path: also alloc-free.
    let mut set: OrdSet<String> = OrdSet::new();
    for i in 0..32 {
        set = set.insert(format!("tag_{i:03}"));
    }
    assert!(set.contains("tag_016"));
    let before = ALLOCS.load(Ordering::Relaxed);
    for _ in 0..ITERS {
        black_box(set.contains(black_box("tag_016")));
    }
    let set_str_allocs = ALLOCS.load(Ordering::Relaxed) - before;

    // OrdMap::remove via Borrow. The persistent rebuild allocates O(log n) nodes
    // either way (the original `map` is unchanged), so remove is NOT alloc-free;
    // the Borrow win is only the removed per-call owned-key String. Both paths do
    // identical rebuild work, so (owned − str) isolates exactly the saved probe alloc.
    let before = ALLOCS.load(Ordering::Relaxed);
    for _ in 0..ITERS {
        black_box(map.remove(black_box("field_016")));
    }
    let str_remove_allocs = ALLOCS.load(Ordering::Relaxed) - before;

    let before = ALLOCS.load(Ordering::Relaxed);
    for _ in 0..ITERS {
        black_box(map.remove(&black_box("field_016").to_string()));
    }
    let owned_remove_allocs = ALLOCS.load(Ordering::Relaxed) - before;

    assert_eq!(
        str_allocs, 0,
        "Borrow &str OrdMap::get must not allocate (got {str_allocs} for {ITERS} calls)"
    );
    assert_eq!(
        set_str_allocs, 0,
        "Borrow &str OrdSet::contains must not allocate (got {set_str_allocs} for {ITERS} calls)"
    );
    assert!(
        owned_allocs >= ITERS,
        "owned-key lookup should allocate ~once per call (got {owned_allocs} for {ITERS} calls)"
    );
    assert!(
        owned_remove_allocs >= str_remove_allocs + ITERS,
        "owned-key remove should allocate >= one extra String per call than the &str probe \
         (owned={owned_remove_allocs}, str={str_remove_allocs}, ITERS={ITERS})"
    );
}
