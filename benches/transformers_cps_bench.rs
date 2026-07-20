//! Benchmarks for CPS transformers.

#![cfg(all(feature = "transformers-cps", feature = "std"))]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ordofp_core::transformers::ecclesia::LectorEcclesiaT;
use std::hint::black_box;

fn cps_left_associative_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("cps_left_associative");

    for depth in &[10, 100, 1_000, 10_000] {
        group.bench_with_input(
            BenchmarkId::new("CPS_ReaderT", depth),
            depth,
            |b, &depth| {
                b.iter(|| {
                    let mut chain = LectorEcclesiaT::new(|env: i32| env);

                    for _ in 0..depth {
                        chain = chain.flat_map(|x| LectorEcclesiaT::new(move |env: i32| env + x));
                    }

                    black_box(chain.run(black_box(1)))
                });
            },
        );
    }

    group.finish();
}

fn cps_map_bench(c: &mut Criterion) {
    c.bench_function("cps_map", |b| {
        b.iter(|| {
            let reader = LectorEcclesiaT::new(|env: i32| env);
            let mapped = reader.map(|x| black_box(x * 2));
            black_box(mapped.run(black_box(42)))
        });
    });
}

fn cps_composition_bench(c: &mut Criterion) {
    c.bench_function("cps_composition", |b| {
        b.iter(|| {
            let reader = LectorEcclesiaT::new(|env: i32| env);
            let mapped = reader.map(|x| x * 2);
            let local = mapped.local(|env| env + 1);
            let composed =
                local.flat_map(|x| LectorEcclesiaT::<i32, i32>::ask().map(move |env| env + x));

            black_box(composed.run(black_box(5)))
        });
    });
}

criterion_group!(
    benches,
    cps_left_associative_bench,
    cps_map_bench,
    cps_composition_bench
);
criterion_main!(benches);
