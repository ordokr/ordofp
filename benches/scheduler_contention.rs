//! Microbench for `OrdoLocalis` contention.
//!
//! Surfaces whether the `Mutex<VecDeque<MunusFibrae>>` scheduler deque
//! exhibits measurable lock contention at realistic worker counts.
//!
//! Workloads:
//! - `single_thread_push_pop_1000`: uncontended baseline, 1000 push/pop ops.
//! - `scheduler_contention/{1,2,4,8}`: 1 producer + N stealers racing on
//!   the same deque. Measures wall-time for a fixed total task budget.
//!
//! The ops/sec delta between the single-thread baseline and the contended
//! cases is the forcing-function number for the open question "should the
//! `Mutex<VecDeque>` scheduler deque be swapped for `crossbeam-deque`?"

#![cfg(feature = "std")]

use std::sync::Arc;
use std::thread;
use std::time::Instant;

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

// `OrdoLocalis` and `MunusFibrae` are re-exported at the `async_core` level.
// `FibraId` has no public constructor, but `Fibra::purus::<()>(()).id()`
// yields one through the public surface.
use ordofp_core::async_core::{Fibra, FibraId, MunusFibrae, OrdoLocalis};

/// Produce a `FibraId` through the public API. `FibraId::new` is
/// `pub(crate)`, so we reach through `Fibra::purus`, which exposes `id()`.
fn fresh_fibra_id() -> FibraId {
    Fibra::<()>::purus(()).id()
}

fn make_task(id: FibraId) -> MunusFibrae {
    MunusFibrae::new(id, || {})
}

fn bench_single_thread_push_pop(c: &mut Criterion) {
    let id = fresh_fibra_id();
    c.bench_function("single_thread_push_pop_1000", |b| {
        b.iter(|| {
            let q = OrdoLocalis::new();
            for _ in 0..1000 {
                q.push(make_task(id));
            }
            while let Some(task) = q.pop() {
                black_box(task);
            }
        });
    });
}

fn bench_contention(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler_contention");
    // Fixed total task budget per iteration; keeps wall-time roughly
    // stable across stealer counts so ops/sec is directly comparable.
    const TASKS_PER_ITER: usize = 10_000;

    for &stealers in &[1usize, 2, 4, 8] {
        group.throughput(criterion::Throughput::Elements(TASKS_PER_ITER as u64));
        group.bench_with_input(BenchmarkId::from_parameter(stealers), &stealers, |b, &n| {
            let id = fresh_fibra_id();
            b.iter_custom(|iters| {
                let mut total = std::time::Duration::ZERO;
                for _ in 0..iters {
                    let q = Arc::new(OrdoLocalis::new());
                    let start = Instant::now();
                    thread::scope(|s| {
                        // Producer: fills the deque.
                        let producer_q = Arc::clone(&q);
                        s.spawn(move || {
                            for _ in 0..TASKS_PER_ITER {
                                producer_q.push(make_task(id));
                            }
                        });
                        // Stealers: drain until the producer is done and
                        // the queue is empty. They contend on the same
                        // `Mutex<VecDeque>` as the producer's `push`.
                        let done = Arc::new(std::sync::atomic::AtomicUsize::new(0));
                        for _ in 0..n {
                            let steal_q = Arc::clone(&q);
                            let done = Arc::clone(&done);
                            s.spawn(move || {
                                loop {
                                    if let Some(task) = steal_q.steal() {
                                        black_box(task);
                                        done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    } else {
                                        // Stop once all producer tasks
                                        // have been drained by *some*
                                        // stealer.
                                        if done.load(std::sync::atomic::Ordering::Relaxed)
                                            >= TASKS_PER_ITER
                                        {
                                            break;
                                        }
                                        std::hint::spin_loop();
                                    }
                                }
                            });
                        }
                    });
                    total += start.elapsed();
                }
                total
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_single_thread_push_pop, bench_contention);
criterion_main!(benches);
