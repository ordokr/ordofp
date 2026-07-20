use criterion::{BenchmarkId, Criterion, criterion_group};
use ordofp_core::par::{
    ParFlumen,
    backend::{CpuRayon, CpuScalar},
};
use std::hint::black_box;

const SIZES: &[usize] = &[10, 100, 1_000, 10_000, 100_000];

// min_len: 1 forces rayon even at small N so we can observe the parallel path.
fn rayon_backend() -> CpuRayon {
    CpuRayon { min_len: 1 }
}

// 200 iterations of wrapping arithmetic per element to ensure compute
// dominates over rayon thread-dispatch overhead at N=10_000+.
#[inline(never)]
fn work(x: i32) -> i32 {
    let mut s = x;
    for i in 0..200i32 {
        s = s.wrapping_add(i.wrapping_mul(x | 1));
    }
    black_box(s)
}

fn bench_nodus_map_collect_rayon(c: &mut Criterion) {
    let mut group = c.benchmark_group("NodusMap/CollectRayon");
    let rayon = rayon_backend();

    for &n in SIZES {
        // Pre-allocate data outside the timed loop.
        let data: Vec<i32> = (0..n as i32).collect();

        group.bench_with_input(BenchmarkId::new("Sequential", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    ParFlumen::from_vec(data.clone())
                        .filter(|x| x % 2 == 0)
                        .map(work)
                        .collect_vec(&CpuScalar),
                )
            });
        });

        group.bench_with_input(BenchmarkId::new("Parallel", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    ParFlumen::from_vec(data.clone())
                        .filter(|x| x % 2 == 0)
                        .map(work)
                        .collect_vec(&rayon),
                )
            });
        });
    }

    group.finish();
}

fn bench_nodus_chain_collect_rayon(c: &mut Criterion) {
    let mut group = c.benchmark_group("NodusChain/CollectRayon");
    let rayon = rayon_backend();

    for &n in SIZES {
        let half = n / 2;
        // Pre-compute work-heavy elements so only collection is timed.
        let a_data: Vec<i32> = (0..half as i32).map(work).collect();
        let b_data: Vec<i32> = (half as i32..n as i32).map(work).collect();

        group.bench_with_input(BenchmarkId::new("Sequential", n), &n, |b, _| {
            b.iter(|| {
                let a = ParFlumen::from_vec(a_data.clone());
                let b_ = ParFlumen::from_vec(b_data.clone());
                black_box(a.chain(b_).collect_vec(&CpuScalar))
            });
        });

        group.bench_with_input(BenchmarkId::new("Parallel", n), &n, |b, _| {
            b.iter(|| {
                let a = ParFlumen::from_vec(a_data.clone());
                let b_ = ParFlumen::from_vec(b_data.clone());
                black_box(a.chain(b_).collect_vec(&rayon))
            });
        });
    }

    group.finish();
}

fn bench_nodus_zip_collect_rayon(c: &mut Criterion) {
    let mut group = c.benchmark_group("NodusZip/CollectRayon");
    let rayon = rayon_backend();

    for &n in SIZES {
        let a_data: Vec<i32> = (0..n as i32).collect();
        let b_data: Vec<i32> = (0..n as i32).map(work).collect();

        // indexed × indexed (NodusZip::is_indexed = true → parallel by index)
        group.bench_with_input(BenchmarkId::new("Sequential/Indexed", n), &n, |b, _| {
            b.iter(|| {
                let a = ParFlumen::from_vec(a_data.clone());
                let b_ = ParFlumen::from_vec(b_data.clone());
                black_box(a.zip(b_).collect_vec(&CpuScalar))
            });
        });

        group.bench_with_input(BenchmarkId::new("Parallel/Indexed", n), &n, |b, _| {
            b.iter(|| {
                let a = ParFlumen::from_vec(a_data.clone());
                let b_ = ParFlumen::from_vec(b_data.clone());
                black_box(a.zip(b_).collect_vec(&rayon))
            });
        });

        // non-indexed × indexed: filter makes first non-indexed → rayon::join path
        group.bench_with_input(BenchmarkId::new("Sequential/NonIndexed", n), &n, |b, _| {
            b.iter(|| {
                let a = ParFlumen::from_vec(a_data.clone()).filter(|_| true);
                let b_ = ParFlumen::from_vec(b_data.clone());
                black_box(a.zip(b_).collect_vec(&CpuScalar))
            });
        });

        group.bench_with_input(BenchmarkId::new("Parallel/NonIndexed", n), &n, |b, _| {
            b.iter(|| {
                let a = ParFlumen::from_vec(a_data.clone()).filter(|_| true);
                let b_ = ParFlumen::from_vec(b_data.clone());
                black_box(a.zip(b_).collect_vec(&rayon))
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_nodus_map_collect_rayon,
    bench_nodus_chain_collect_rayon,
    bench_nodus_zip_collect_rayon,
);
