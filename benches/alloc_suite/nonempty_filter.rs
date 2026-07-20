use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group};
use ordofp_core::nonempty::NonEmpty;
use std::hint::black_box;

// NonEmpty::filter consumes self. The optimized form builds a single Vec
// (one allocation, no clones) instead of `to_vec().into_iter().filter()
// .collect()`, which allocated a full-size intermediate Vec plus the result.
// iter_batched clones a fresh template per iteration as untimed setup, so the
// timed routine is purely the filter.
fn bench_nonempty_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("NonEmpty/filter");
    for &n in &[100usize, 1_000, 10_000, 100_000] {
        let template: NonEmpty<i64> = NonEmpty::new(0, (1..n as i64).collect());
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter_batched(
                || template.clone(),
                |nel| black_box(nel.filter(|x| x % 2 == 0)),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_nonempty_filter);
