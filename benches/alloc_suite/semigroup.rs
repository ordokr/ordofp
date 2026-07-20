//! Semigroup benchmarks.

use criterion::{Criterion, criterion_group};
use ordofp::semigroup::Compositio;
use std::hint::black_box;

fn combine_i32(c: &mut Criterion) {
    let x: i32 = 10;
    let y: i32 = 50;
    c.bench_function("combine_i32", |b| b.iter(|| black_box(x.combine(&y))));
}

fn std_add_i32(c: &mut Criterion) {
    let x: i32 = 10;
    let y: i32 = 50;
    c.bench_function("std_add_i32", |b| b.iter(|| black_box(x + y)));
}

fn combine_option_string(c: &mut Criterion) {
    let x: Option<String> = Some("hello".to_owned());
    let y: Option<String> = Some(" world".to_owned());
    c.bench_function("combine_option_string", |b| {
        b.iter(|| black_box(x.combine(&y)));
    });
}

fn std_add_option_string(c: &mut Criterion) {
    let x: Option<String> = Some("hello".to_owned());
    let y: Option<String> = Some(" world".to_owned());
    c.bench_function("std_add_option_string", |b| {
        b.iter(|| {
            // cloning is required otherwise we get `cannot move out of captured outer variable in an `FnMut` closure` errors
            let a = x.clone();
            let b = y.clone();
            black_box(a.and_then(|first| b.map(|second| first + &second)))
        });
    });
}

criterion_group!(
    benches,
    combine_i32,
    std_add_i32,
    combine_option_string,
    std_add_option_string
);
