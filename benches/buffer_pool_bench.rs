//! Benchmarks for buffer reuse algorithms.

#![cfg(all(feature = "par", feature = "gpu-buffer-pool", feature = "std"))]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ordofp_core::par::opt::buffer_pool::BufferPool;
use std::hint::black_box;

fn buffer_pool_coloring_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_pool_coloring");

    for num_buffers in &[10, 100, 1_000, 10_000] {
        group.bench_with_input(
            BenchmarkId::new("greedy_coloring", num_buffers),
            num_buffers,
            |b, &num_buffers| {
                b.iter(|| {
                    let mut pool = BufferPool::new();

                    // Add buffers with various overlaps
                    for i in 0..num_buffers {
                        let start = i * 5;
                        let end = start + 10;
                        pool.add_buffer(black_box(i), start, end);
                    }

                    pool.compute_coloring();
                    black_box(pool)
                });
            },
        );
    }

    group.finish();
}

fn buffer_pool_interference_bench(c: &mut Criterion) {
    c.bench_function("buffer_pool_interference_detection", |b| {
        b.iter(|| {
            let mut pool = BufferPool::new();

            // Add many overlapping buffers
            for i in 0..black_box(1000) {
                let start = i * 3;
                let end = start + 10;
                pool.add_buffer(black_box(i), start, end);
            }

            black_box(pool)
        });
    });
}

criterion_group!(
    benches,
    buffer_pool_coloring_bench,
    buffer_pool_interference_bench
);
criterion_main!(benches);
