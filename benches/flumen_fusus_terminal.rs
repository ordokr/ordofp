#[cfg(all(feature = "async", feature = "fusion"))]
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
#[cfg(all(feature = "async", feature = "fusion"))]
use std::hint::black_box;

#[cfg(all(feature = "async", feature = "fusion"))]
mod terminal_benches {
    use super::{BenchmarkId, Criterion, black_box};
    use core::pin::Pin;
    use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    use ordofp_core::async_core::Flumen;

    fn noop_waker() -> Waker {
        unsafe fn clone(_: *const ()) -> RawWaker {
            RawWaker::new(core::ptr::null(), &VTABLE)
        }
        unsafe fn wake(_: *const ()) {}
        unsafe fn wake_by_ref(_: *const ()) {}
        unsafe fn drop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
        unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) }
    }

    fn block_on<F: core::future::Future>(mut fut: F) -> F::Output {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(v) => return v,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    const SIZES: &[usize] = &[100, 1_000, 10_000, 100_000];

    pub fn bench_all(c: &mut Criterion) {
        let mut group = c.benchmark_group("FlumenFusus/All");

        for &n in SIZES {
            let data: Vec<i32> = (0..n as i32).collect();

            // Worst case: predicate always true, must drain entire stream
            group.bench_with_input(BenchmarkId::new("AllTrue", n), &n, |b, _| {
                b.iter(|| {
                    let fut = Flumen::from_iterator(data.clone()).fuse().all(|x| *x >= 0);
                    black_box(block_on(fut))
                });
            });

            // Best case: predicate false at first element, short-circuits immediately
            group.bench_with_input(BenchmarkId::new("ImmediateFalse", n), &n, |b, _| {
                b.iter(|| {
                    let fut = Flumen::from_iterator(data.clone()).fuse().all(|x| *x < 0);
                    black_box(block_on(fut))
                });
            });
        }

        group.finish();
    }

    pub fn bench_last(c: &mut Criterion) {
        let mut group = c.benchmark_group("FlumenFusus/Last");

        for &n in SIZES {
            let data: Vec<i32> = (0..n as i32).collect();

            // last always drains the entire stream (no early exit possible)
            group.bench_with_input(BenchmarkId::new("DrainAll", n), &n, |b, _| {
                b.iter(|| {
                    let fut = Flumen::from_iterator(data.clone()).fuse().last();
                    black_box(block_on(fut))
                });
            });
        }

        group.finish();
    }

    pub fn bench_nth(c: &mut Criterion) {
        let mut group = c.benchmark_group("FlumenFusus/Nth");
        let n = 10_000usize;
        let data: Vec<i32> = (0..n as i32).collect();

        // nth at index 0: exits after first element
        group.bench_function("Index0", |b| {
            b.iter(|| {
                let fut = Flumen::from_iterator(data.clone()).fuse().nth(0);
                black_box(block_on(fut))
            });
        });

        // nth at middle
        group.bench_function("IndexMid", |b| {
            b.iter(|| {
                let fut = Flumen::from_iterator(data.clone()).fuse().nth(n / 2);
                black_box(block_on(fut))
            });
        });

        // nth at last element
        group.bench_function("IndexLast", |b| {
            b.iter(|| {
                let fut = Flumen::from_iterator(data.clone()).fuse().nth(n - 1);
                black_box(block_on(fut))
            });
        });

        // nth past end: drains entire stream, returns None
        group.bench_function("IndexPastEnd", |b| {
            b.iter(|| {
                let fut = Flumen::from_iterator(data.clone()).fuse().nth(n + 100);
                black_box(block_on(fut))
            });
        });

        group.finish();
    }

    pub fn bench_position(c: &mut Criterion) {
        let mut group = c.benchmark_group("FlumenFusus/Position");
        let n = 10_000usize;
        let data: Vec<i32> = (0..n as i32).collect();

        // position at start: exits immediately
        group.bench_function("FoundFirst", |b| {
            b.iter(|| {
                let fut = Flumen::from_iterator(data.clone())
                    .fuse()
                    .position(|x| *x == 0);
                black_box(block_on(fut))
            });
        });

        // position at middle
        group.bench_function("FoundMid", |b| {
            b.iter(|| {
                let mid = (n / 2) as i32;
                let fut = Flumen::from_iterator(data.clone())
                    .fuse()
                    .position(move |x| *x == mid);
                black_box(block_on(fut))
            });
        });

        // position not found: drains entire stream
        group.bench_function("NotFound", |b| {
            b.iter(|| {
                let fut = Flumen::from_iterator(data.clone())
                    .fuse()
                    .position(|x| *x < 0);
                black_box(block_on(fut))
            });
        });

        group.finish();
    }
}

#[cfg(all(feature = "async", feature = "fusion"))]
fn bench_all(c: &mut Criterion) {
    terminal_benches::bench_all(c);
}
#[cfg(all(feature = "async", feature = "fusion"))]
fn bench_last(c: &mut Criterion) {
    terminal_benches::bench_last(c);
}
#[cfg(all(feature = "async", feature = "fusion"))]
fn bench_nth(c: &mut Criterion) {
    terminal_benches::bench_nth(c);
}
#[cfg(all(feature = "async", feature = "fusion"))]
fn bench_position(c: &mut Criterion) {
    terminal_benches::bench_position(c);
}

#[cfg(all(feature = "async", feature = "fusion"))]
criterion_group!(
    terminal_ops,
    bench_all,
    bench_last,
    bench_nth,
    bench_position,
);

#[cfg(all(feature = "async", feature = "fusion"))]
criterion_main!(terminal_ops);

#[cfg(not(all(feature = "async", feature = "fusion")))]
fn main() {
    eprintln!("flumen_fusus_terminal requires --features \"fusion\"");
}
