use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ordofp_core::par::{FlumenParallelumFast, ParFlumen, backend::CpuScalar};
use std::hint::black_box;

// A Clone-expensive type: payload forces a heap allocation on every clone.
#[derive(Clone)]
struct Payload {
    s: String,
    v: i32,
}

impl Payload {
    fn new(v: i32) -> Self {
        // 64-byte heap string to make Clone measurably expensive.
        Payload {
            s: format!("{v:064}"),
            v,
        }
    }
}

const SIZES: &[usize] = &[100, 1_000, 10_000, 100_000];

fn bench_fast_filter_scalar_ref(c: &mut Criterion) {
    let mut group = c.benchmark_group("FastFilter/VisitScalarRef");

    for &n in SIZES {
        let data: Vec<Payload> = (0..n as i32).map(Payload::new).collect();

        // Fast path: monomorphic, uses overridden visit_scalar_ref (zero vtable)
        group.bench_with_input(BenchmarkId::new("FastPath", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    FlumenParallelumFast::from_vec(data.clone())
                        .filter(|p| p.v % 2 == 0)
                        .collect_vec(&CpuScalar),
                )
            });
        });

        // Dyn path: vtable dispatch, also uses visit_scalar_ref but through dyn
        group.bench_with_input(BenchmarkId::new("DynPath", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    ParFlumen::from_vec(data.clone())
                        .filter(|p| p.v % 2 == 0)
                        .collect_vec(&CpuScalar),
                )
            });
        });

        // Naive iterator baseline: explicit clone-per-element
        group.bench_with_input(BenchmarkId::new("IterBaseline", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    data.iter()
                        .filter(|p| p.v % 2 == 0)
                        .cloned()
                        .collect::<Vec<_>>(),
                )
            });
        });
    }

    group.finish();
}

fn bench_fast_inspect_scalar_ref(c: &mut Criterion) {
    let mut group = c.benchmark_group("FastInspect/VisitScalarRef");
    for &n in SIZES {
        let data: Vec<Payload> = (0..n as i32).map(Payload::new).collect();

        group.bench_with_input(BenchmarkId::new("FastPath", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    FlumenParallelumFast::from_vec(data.clone())
                        .inspect(|p| {
                            let _ = &p.s;
                        })
                        .collect_vec(&CpuScalar),
                )
            });
        });

        group.bench_with_input(BenchmarkId::new("DynPath", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    ParFlumen::from_vec(data.clone())
                        .inspect(|p| {
                            let _ = &p.s;
                        })
                        .collect_vec(&CpuScalar),
                )
            });
        });

        group.bench_with_input(BenchmarkId::new("IterBaseline", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    data.iter()
                        .inspect(|p| {
                            let _ = &p.s;
                        })
                        .cloned()
                        .collect::<Vec<_>>(),
                )
            });
        });
    }

    group.finish();
}

fn bench_fast_scan_scalar_ref(c: &mut Criterion) {
    let mut group = c.benchmark_group("FastScan/VisitScalarRef");

    for &n in SIZES {
        let data: Vec<i32> = (0..n as i32).collect();
        // Use a String accumulator to make Clone cost visible.
        let init = String::with_capacity(64);

        group.bench_with_input(BenchmarkId::new("FastPath", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    FlumenParallelumFast::from_vec(data.clone())
                        .scan(init.clone(), |acc, x| {
                            let mut next = acc.clone();
                            next.push_str(&x.to_string());
                            if next.len() > 64 {
                                next.truncate(64);
                            }
                            next
                        })
                        .collect_vec(&CpuScalar),
                )
            });
        });

        group.bench_with_input(BenchmarkId::new("DynPath", n), &n, |b, _| {
            b.iter(|| {
                black_box(
                    ParFlumen::from_vec(data.clone())
                        .scan(init.clone(), |acc, x| {
                            let mut next = acc.clone();
                            next.push_str(&x.to_string());
                            if next.len() > 64 {
                                next.truncate(64);
                            }
                            next
                        })
                        .collect_vec(&CpuScalar),
                )
            });
        });

        group.bench_with_input(BenchmarkId::new("IterBaseline", n), &n, |b, _| {
            b.iter(|| {
                let mut acc = init.clone();
                black_box(
                    data.iter()
                        .map(|&x| {
                            acc.push_str(&x.to_string());
                            if acc.len() > 64 {
                                acc.truncate(64);
                            }
                            acc.clone()
                        })
                        .collect::<Vec<_>>(),
                )
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_fast_filter_scalar_ref,
    bench_fast_inspect_scalar_ref,
    bench_fast_scan_scalar_ref,
);
criterion_main!(benches);
