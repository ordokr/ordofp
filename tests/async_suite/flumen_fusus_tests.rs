#![cfg(all(feature = "async", feature = "fusion"))]

use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use ordofp::async_core::Flumen;

fn noop_waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(core::ptr::null(), &VTABLE)
    }
    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    // SAFETY: All four vtable functions satisfy the `RawWakerVTable` contract:
    // `clone` returns a valid `RawWaker`, `wake`/`wake_by_ref`/`drop` are no-ops.
    // The data pointer is intentionally null because none of the vtable functions
    // dereference it. `VTABLE` is `'static`, so the vtable pointer is always valid
    // for the lifetime of any `Waker` constructed here.
    unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) }
}

fn block_on<F: core::future::Future>(mut fut: F) -> F::Output {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    // Safety: we never move `fut` after pinning it.
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

#[test]
fn flumen_fuse_equivalence_map_filter_take_collect() {
    let data: Vec<i32> = (0..10_000).collect();

    let boxed = block_on(
        Flumen::from_iterator(data.clone())
            .fmap(|x| x + 1)
            .filter(|x| *x % 2 == 0)
            .take(1_000)
            .collect_vec(),
    );

    let fused = block_on(
        Flumen::from_iterator(data)
            .fuse()
            .map(|x| x + 1)
            .filter(|x| *x % 2 == 0)
            .take(1_000)
            .collect_vec(),
    );

    assert_eq!(boxed, fused);
}

#[test]
fn flumen_fuse_filter_map_equivalence() {
    let data: Vec<i32> = (0..10_000).collect();

    let boxed = block_on(
        Flumen::from_iterator(data.clone())
            .filter_map(|x| if x % 3 == 0 { Some(x / 3) } else { None })
            .take(1_000)
            .collect_vec(),
    );

    let fused = block_on(
        Flumen::from_iterator(data)
            .fuse()
            .filter_map(|x| if x % 3 == 0 { Some(x / 3) } else { None })
            .take(1_000)
            .collect_vec(),
    );

    assert_eq!(boxed, fused);
}

#[test]
fn flumen_fuse_scan_equivalence() {
    let data: Vec<i32> = (0..2_000).collect();

    let boxed = block_on(
        Flumen::from_iterator(data.clone())
            .scan(0i32, |acc, x| acc + x)
            .take(500)
            .collect_vec(),
    );

    let fused = block_on(
        Flumen::from_iterator(data)
            .fuse()
            .scan(0i32, |acc, x| acc + x)
            .take(500)
            .collect_vec(),
    );

    assert_eq!(boxed, fused);
}

#[test]
fn flumen_fuse_scan_with_equivalence() {
    let data: Vec<i32> = (0..2_000).collect();

    let boxed = block_on(
        Flumen::from_iterator(data.clone())
            .scan_with(0i32, |acc, x| {
                *acc += x;
                if *acc > 10_000 { None } else { Some(*acc) }
            })
            .collect_vec(),
    );

    let fused = block_on(
        Flumen::from_iterator(data)
            .fuse()
            .scan_with(0i32, |acc, x| {
                *acc += x;
                if *acc > 10_000 { None } else { Some(*acc) }
            })
            .collect_vec(),
    );

    assert_eq!(boxed, fused);
}

#[test]
fn flumen_fuse_take_while_equivalence() {
    let data: Vec<i32> = (0..10_000).collect();

    let boxed = block_on(
        Flumen::from_iterator(data.clone())
            .take_while(|x| *x < 1_234)
            .collect_vec(),
    );

    let fused = block_on(
        Flumen::from_iterator(data)
            .fuse()
            .take_while(|x| *x < 1_234)
            .collect_vec(),
    );

    assert_eq!(boxed, fused);
}

#[test]
fn flumen_fuse_skip_while_equivalence() {
    let data: Vec<i32> = (0..10_000).collect();

    let boxed = block_on(
        Flumen::from_iterator(data.clone())
            .skip_while(|x| *x < 9_000)
            .take(100)
            .collect_vec(),
    );

    let fused = block_on(
        Flumen::from_iterator(data)
            .fuse()
            .skip_while(|x| *x < 9_000)
            .take(100)
            .collect_vec(),
    );

    assert_eq!(boxed, fused);
}

#[test]
fn flumen_fuse_enumerate_equivalence() {
    let data: Vec<i32> = (0..1_000).collect();

    let boxed = block_on(
        Flumen::from_iterator(data.clone())
            .enumerate()
            .take(200)
            .collect_vec(),
    );

    let fused = block_on(
        Flumen::from_iterator(data)
            .fuse()
            .enumerate()
            .take(200)
            .collect_vec(),
    );

    assert_eq!(boxed, fused);
}

#[test]
fn flumen_fuse_chunks_equivalence() {
    let data: Vec<i32> = (0..25).collect();

    let boxed = block_on(Flumen::from_iterator(data.clone()).chunks(4).collect_vec());

    let fused = block_on(Flumen::from_iterator(data).fuse().chunks(4).collect_vec());

    assert_eq!(boxed, fused);
}
