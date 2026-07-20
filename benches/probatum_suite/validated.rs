//! Probatum benchmarks.

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group};
use ordofp::validated::{IntoProbatum, Probatum};
use std::hint::black_box;

fn yah_nah(yah: bool) -> Result<i32, String> {
    if yah {
        Result::Ok(32)
    } else {
        Result::Err("Shiz".to_owned())
    }
}

fn ok_result_into_probatum(c: &mut Criterion) {
    let r = yah_nah(true);
    c.bench_function("ok_result_into_probatum", |b| {
        b.iter(|| black_box(r.clone().into_probatum()));
    });
}

fn error_result_into_probatum(c: &mut Criterion) {
    let r = yah_nah(false);
    c.bench_function("error_result_into_probatum", |b| {
        b.iter(|| black_box(r.clone().into_probatum()));
    });
}

fn adding_result_to_probatum_all_good(c: &mut Criterion) {
    let v = yah_nah(true).into_probatum();
    c.bench_function("adding_result_to_probatum_all_good", |b| {
        b.iter(|| {
            let r1 = yah_nah(true).into_probatum();
            let r2 = yah_nah(true).into_probatum();
            let r3 = yah_nah(true).into_probatum();
            let r4 = yah_nah(true).into_probatum();
            black_box(
                v.clone()
                    .map2(r1, |a, b| a + b)
                    .map2(r2, |ab, c| ab + c)
                    .map2(r3, |abc, d| abc + d)
                    .map2(r4, |abcd, e| abcd + e),
            )
        });
    });
}

fn adding_result_to_probatum_all_bad(c: &mut Criterion) {
    let v = yah_nah(false).into_probatum();
    c.bench_function("adding_result_to_probatum_all_bad", |b| {
        b.iter(|| {
            let r1 = yah_nah(false).into_probatum();
            let r2 = yah_nah(false).into_probatum();
            let r3 = yah_nah(false).into_probatum();
            let r4 = yah_nah(false).into_probatum();
            black_box(
                v.clone()
                    .map2(r1, |a, b| a + b)
                    .map2(r2, |ab, c| ab + c)
                    .map2(r3, |abc, d| abc + d)
                    .map2(r4, |abcd, e| abcd + e),
            )
        });
    });
}

fn adding_result_to_probatum_mixed(c: &mut Criterion) {
    let v = yah_nah(true).into_probatum();
    c.bench_function("adding_result_to_probatum_mixed", |b| {
        b.iter(|| {
            let r1 = yah_nah(false).into_probatum();
            let r2 = yah_nah(true).into_probatum();
            let r3 = yah_nah(false).into_probatum();
            let r4 = yah_nah(true).into_probatum();
            black_box(
                v.clone()
                    .map2(r1, |a, b| a + b)
                    .map2(r2, |ab, c| ab + c)
                    .map2(r3, |abc, d| abc + d)
                    .map2(r4, |abcd, e| abcd + e),
            )
        });
    });
}

fn adding_probatums(c: &mut Criterion) {
    let v1 = yah_nah(true).into_probatum();
    let v2 = yah_nah(true).into_probatum();
    let v3 = yah_nah(true).into_probatum();
    let v4 = yah_nah(true).into_probatum();
    c.bench_function("adding_probatums", |b| {
        b.iter(|| {
            black_box(
                v1.clone()
                    .map2(v2.clone(), |a, b| a + b)
                    .map2(v3.clone(), |ab, c| ab + c)
                    .map2(v4.clone(), |abc, d| abc + d),
            )
        });
    });
}

fn probatum_to_result(c: &mut Criterion) {
    let v1 = yah_nah(true).into_probatum();
    c.bench_function("Probatum_to_result", |b| {
        b.iter(|| {
            // Return only a small token: the closure's output would otherwise
            // be a Result with a large Err type (the error accumulator).
            let result = black_box(v1.clone().into_result());
            result.is_ok()
        });
    });
}

fn probatum_collect(c: &mut Criterion) {
    let mut group = c.benchmark_group("Probatum_collect");
    for &n in &[100usize, 1_000, 10_000, 100_000] {
        let template: Vec<Probatum<String, i32>> = (0..n as i32).map(Probatum::Valid).collect();
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter_batched(
                || template.clone(),
                |items| black_box(Probatum::<String, ()>::collect(items)),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    ok_result_into_probatum,
    error_result_into_probatum,
    adding_result_to_probatum_all_good,
    adding_result_to_probatum_all_bad,
    adding_result_to_probatum_mixed,
    adding_probatums,
    probatum_to_result,
    probatum_collect
);
