//! Monoid benchmarks.

use criterion::{Criterion, criterion_group};
use ordofp::monoid::combine_all;
use std::hint::black_box;

fn combine_all_i32(c: &mut Criterion) {
    let v = vec![
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        Some(9),
        Some(10),
    ];
    c.bench_function("combine_all_i32", |b| b.iter(|| black_box(combine_all(&v))));
}

fn std_add_all_i32(c: &mut Criterion) {
    let v = vec![
        Some(1),
        Some(2),
        Some(3),
        Some(4),
        Some(5),
        Some(6),
        Some(7),
        Some(8),
        Some(9),
        Some(10),
    ];
    c.bench_function("std_add_all_i32", |b| {
        b.iter(|| {
            black_box(
                v.iter()
                    .try_fold(0, |acc, maybe_n| maybe_n.map(|n| acc + n)),
            )
        });
    });
}

criterion_group!(benches, combine_all_i32, std_add_all_i32);
