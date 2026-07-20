//! Allocation-count fusion guard for `FlumenFusus` pipelines.
//!
//! The fusion layer's whole claim is that composed combinators compile to a
//! single state machine with no intermediate collections. This test makes
//! that claim a hard number: the allocation count of a map→filter→fold
//! pipeline must not scale with the stream length. If a combinator ever
//! starts materializing intermediates (one allocation per element or per
//! chunk), the two deltas below diverge and the gate goes red.

#![cfg(feature = "fusion")]

use core::pin::pin;
use core::task::{Context, Poll, Waker};
use ordofp_core::async_core::Flumen;
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

/// The fused pipelines never return `Pending` for vector-backed sources, so a
/// noop-waker poll loop suffices (and allocates nothing itself).
fn block_on<F: Future>(fut: F) -> F::Output {
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut fut = pin!(fut);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

/// Run the reference pipeline and return `(result, allocations)`.
fn run_pipeline(data: Vec<i64>) -> (i64, usize) {
    let before = ALLOCS.load(Ordering::Relaxed);
    let sum = block_on(
        Flumen::from_iterator(black_box(data))
            .fuse()
            .map(|x| x * 2)
            .filter(|x| x % 3 == 0)
            .fold(0i64, |acc, x| acc + x),
    );
    let delta = ALLOCS.load(Ordering::Relaxed) - before;
    (black_box(sum), delta)
}

/// Single test fn: the counting allocator is process-global, so concurrent
/// tests in this binary would pollute each other's deltas.
#[test]
fn fused_pipeline_allocations_do_not_scale_with_length() {
    let small: Vec<i64> = (0..1_000).collect();
    let large: Vec<i64> = (0..100_000).collect();

    // Correctness first: the fused pipeline must agree with plain iterators.
    let expected_small: i64 = small.iter().map(|x| x * 2).filter(|x| x % 3 == 0).sum();
    let (sum_small, allocs_small) = run_pipeline(small);
    assert_eq!(sum_small, expected_small);

    let expected_large: i64 = large.iter().map(|x| x * 2).filter(|x| x % 3 == 0).sum();
    let (sum_large, allocs_large) = run_pipeline(large);
    assert_eq!(sum_large, expected_large);

    // The fusion property: a 100x longer stream must not allocate more.
    // (Each run pays a small constant for the stream wrapper itself.)
    assert_eq!(
        allocs_small, allocs_large,
        "fused pipeline allocations scale with stream length: \
         {allocs_small} allocs at n=1_000 vs {allocs_large} at n=100_000 — \
         a combinator is materializing intermediates"
    );
}
