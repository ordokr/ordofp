//! Microbench for Zio bind throughput.
//!
//! Measures wall time for running chains of `Zio::flat_map` at depths
//! 1, 8, 64, 256 to surface per-bind overhead.

#![cfg(feature = "tokio")]

use criterion::{Criterion, criterion_group};
use ordofp::async_core::zio::Zio;
use std::hint::black_box;
use tokio::runtime::Builder;

fn zio_chain(n: usize) -> Zio<(), (), usize> {
    let mut z: Zio<(), (), usize> = Zio::succeed(0);
    for _ in 0..n {
        z = z.flat_map(|x: usize| Zio::succeed(x + 1));
    }
    z
}

fn bench_zio_bind(c: &mut Criterion) {
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio current-thread runtime");

    let mut group = c.benchmark_group("zio_bind_chain");
    for depth in [1usize, 8, 64, 256] {
        group.bench_function(format!("depth_{depth}"), |b| {
            b.iter(|| {
                let z = zio_chain(black_box(depth));
                // `Zio::run` is an async method (the raw `run` field is private),
                // so drive it through the current-thread tokio runtime.
                let result = rt.block_on(z.run(()));
                black_box(result)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_zio_bind);
