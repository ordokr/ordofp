use criterion::{BenchmarkId, Criterion, criterion_group};
use ordofp_core::par::{
    ParFlumen,
    backend::{CpuRayon, CpuScalar},
};
use std::hint::black_box;

const SIZES: &[usize] = &[10, 100, 1_000, 10_000, 100_000];

fn rayon_backend() -> CpuRayon {
    CpuRayon { min_len: 1 }
}

fn work(x: i32) -> i32 {
    let mut s = x;
    for i in 0..50i32 {
        s = s.wrapping_add(i.wrapping_mul(x));
    }
    s
}

// Non-indexed pipeline: NodusFilter is not indexed (is_indexed() = false).
// CpuRayon::for_each now collects in parallel first then runs for_each;
// formerly non-indexed pipelines fell back to scalar.
fn bench_cpu_rayon_for_each_non_indexed(c: &mut Criterion) {
    let mut group = c.benchmark_group("CpuRayon/ForEach/NonIndexed");
    let rayon = rayon_backend();

    for &n in SIZES {
        group.bench_with_input(BenchmarkId::new("Scalar", n), &n, |b, &n| {
            use std::sync::atomic::{AtomicI32, Ordering};
            b.iter(|| {
                let total = AtomicI32::new(0);
                ParFlumen::from_vec((0..n as i32).collect::<Vec<_>>())
                    .filter(|x| x % 2 == 0)
                    .for_each(&CpuScalar, |x| {
                        total.fetch_add(work(x), Ordering::Relaxed);
                    });
                black_box(total.load(Ordering::Relaxed))
            });
        });

        group.bench_with_input(BenchmarkId::new("Parallel", n), &n, |b, &n| {
            use std::sync::atomic::{AtomicI32, Ordering};
            b.iter(|| {
                let total = AtomicI32::new(0);
                ParFlumen::from_vec((0..n as i32).collect::<Vec<_>>())
                    .filter(|x| x % 2 == 0)
                    .for_each(&rayon, |x| {
                        total.fetch_add(work(x), Ordering::Relaxed);
                    });
                black_box(total.load(Ordering::Relaxed))
            });
        });
    }

    group.finish();
}

// Non-indexed reduce: formerly only reduce_scalar was available for non-indexed
// pipelines; now the pipeline collects with collect_rayon (which may itself be
// parallel for indexed upstream) then tree-reduces the Vec in parallel.
fn bench_cpu_rayon_reduce_non_indexed(c: &mut Criterion) {
    let mut group = c.benchmark_group("CpuRayon/Reduce/NonIndexed");
    let rayon = rayon_backend();

    for &n in SIZES {
        group.bench_with_input(BenchmarkId::new("Scalar", n), &n, |b, &n| {
            b.iter(|| {
                black_box(
                    ParFlumen::from_vec((0..n as i32).collect::<Vec<_>>())
                        .filter(|x| x % 2 == 0)
                        .map(work)
                        .reduce(&CpuScalar, i32::wrapping_add),
                )
            });
        });

        group.bench_with_input(BenchmarkId::new("Parallel", n), &n, |b, &n| {
            b.iter(|| {
                black_box(
                    ParFlumen::from_vec((0..n as i32).collect::<Vec<_>>())
                        .filter(|x| x % 2 == 0)
                        .map(work)
                        .reduce(&rayon, i32::wrapping_add),
                )
            });
        });
    }

    group.finish();
}

// Non-indexed map+for_each: `filter().map().for_each`. The terminal node is
// NodusMap, so this exercises the fused `NodusMap::for_each_rayon` (map fused
// into the parallel for_each, skipping the throwaway intermediate Vec).
fn bench_cpu_rayon_map_for_each_non_indexed(c: &mut Criterion) {
    let mut group = c.benchmark_group("CpuRayon/MapForEach/NonIndexed");
    let rayon = rayon_backend();

    for &n in SIZES {
        group.bench_with_input(BenchmarkId::new("Scalar", n), &n, |b, &n| {
            use std::sync::atomic::{AtomicI32, Ordering};
            b.iter(|| {
                let total = AtomicI32::new(0);
                ParFlumen::from_vec((0..n as i32).collect::<Vec<_>>())
                    .filter(|x| x % 2 == 0)
                    .map(work)
                    .for_each(&CpuScalar, |x| {
                        total.fetch_add(x, Ordering::Relaxed);
                    });
                black_box(total.load(Ordering::Relaxed))
            });
        });

        group.bench_with_input(BenchmarkId::new("Parallel", n), &n, |b, &n| {
            use std::sync::atomic::{AtomicI32, Ordering};
            b.iter(|| {
                let total = AtomicI32::new(0);
                ParFlumen::from_vec((0..n as i32).collect::<Vec<_>>())
                    .filter(|x| x % 2 == 0)
                    .map(work)
                    .for_each(&rayon, |x| {
                        total.fetch_add(x, Ordering::Relaxed);
                    });
                black_box(total.load(Ordering::Relaxed))
            });
        });
    }

    group.finish();
}

// Map-free `filter().reduce()`: the terminal node is NodusFilter, so this
// exercises the fused `NodusFilter::reduce_rayon` (predicate fused into the
// tree-reduce by index, skipping the throwaway intermediate Vec).
fn bench_cpu_rayon_filter_reduce_non_indexed(c: &mut Criterion) {
    let mut group = c.benchmark_group("CpuRayon/FilterReduce/NonIndexed");
    let rayon = rayon_backend();

    for &n in SIZES {
        group.bench_with_input(BenchmarkId::new("Scalar", n), &n, |b, &n| {
            b.iter(|| {
                black_box(
                    ParFlumen::from_vec((0..n as i32).collect::<Vec<_>>())
                        .filter(|x| x % 2 == 0)
                        .reduce(&CpuScalar, i32::wrapping_add),
                )
            });
        });

        group.bench_with_input(BenchmarkId::new("Parallel", n), &n, |b, &n| {
            b.iter(|| {
                black_box(
                    ParFlumen::from_vec((0..n as i32).collect::<Vec<_>>())
                        .filter(|x| x % 2 == 0)
                        .reduce(&rayon, i32::wrapping_add),
                )
            });
        });

        // Work-aware default: must NOT lag `Scalar` at small/cheap sizes (it
        // falls back to the scalar terminal below the cost-derived threshold)
        // while still parallelizing once the input is large enough to pay.
        group.bench_with_input(BenchmarkId::new("DefaultWorkAware", n), &n, |b, &n| {
            let default = CpuRayon::default();
            b.iter(|| {
                black_box(
                    ParFlumen::from_vec((0..n as i32).collect::<Vec<_>>())
                        .filter(|x| x % 2 == 0)
                        .reduce(&default, i32::wrapping_add),
                )
            });
        });
    }

    group.finish();
}

// `filter().chain(filter()).reduce()`: a non-indexed chain whose terminal is
// NodusChain, exercising the fused `NodusChain::reduce_rayon` (join the two
// halves' fused reduces, no concatenated Vec).
fn bench_cpu_rayon_chain_reduce_non_indexed(c: &mut Criterion) {
    let mut group = c.benchmark_group("CpuRayon/ChainReduce/NonIndexed");
    let rayon = rayon_backend();

    let build = |n: usize| {
        ParFlumen::from_vec((0..n as i32).collect::<Vec<_>>())
            .filter(|x| x % 2 == 0)
            .chain(
                ParFlumen::from_vec((n as i32..2 * n as i32).collect::<Vec<_>>())
                    .filter(|x| x % 3 == 0),
            )
    };

    for &n in SIZES {
        group.bench_with_input(BenchmarkId::new("Scalar", n), &n, |b, &n| {
            b.iter(|| black_box(build(n).reduce(&CpuScalar, i32::wrapping_add)));
        });
        group.bench_with_input(BenchmarkId::new("Parallel", n), &n, |b, &n| {
            b.iter(|| black_box(build(n).reduce(&rayon, i32::wrapping_add)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cpu_rayon_for_each_non_indexed,
    bench_cpu_rayon_reduce_non_indexed,
    bench_cpu_rayon_map_for_each_non_indexed,
    bench_cpu_rayon_filter_reduce_non_indexed,
    bench_cpu_rayon_chain_reduce_non_indexed,
);
