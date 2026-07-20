//! `HList` benchmarks.

use criterion::{Criterion, criterion_group};
use ordofp_core::{coniunctio_pat, hlist};
use std::hint::black_box;

fn creating_hlist(c: &mut Criterion) {
    c.bench_function("creating_hlist", |b| {
        b.iter(|| black_box(hlist![1, 2, 3.3f32, "hi2", true]));
    });
}

fn creating_tuple2(c: &mut Criterion) {
    c.bench_function("creating_tuple2", |b| {
        b.iter(|| black_box((1, (2, (3.3f32, ("hi2", true))))));
    });
}

fn hlist_into_tuple2(c: &mut Criterion) {
    let h = hlist![1, 2, 3.3f32, "hi2", true];
    c.bench_function("hlist_into_tuple2", |b| {
        b.iter(|| black_box(h.into_tuple2()));
    });
}

fn hlist_into_tuple2_match(c: &mut Criterion) {
    let h = hlist![1, 2, 3.3f32, "hi2", true];
    c.bench_function("hlist_into_tuple2_match", |b| {
        b.iter(|| {
            let (a, (b, (c, (d, e)))) = h.into_tuple2();
            black_box((a, b, c, d, e))
        });
    });
}

fn hlist_into_coniunctio_pat_match(c: &mut Criterion) {
    let h = hlist![1, 2, 3.3f32, "hi2", true];
    c.bench_function("hlist_into_coniunctio_pat_match", |b| {
        b.iter(|| {
            let coniunctio_pat!(a, b, c, d, e) = h;
            black_box((a, b, c, d, e))
        });
    });
}

fn hlist_append(c: &mut Criterion) {
    let h1 = hlist![1, 2, 3.3f32, "hi2", true];
    let h2 = hlist![true, "blue", "varsity"];
    c.bench_function("hlist_append", |b| b.iter(|| black_box(h1 + h2)));
}

fn hlist_mapping_consuming(c: &mut Criterion) {
    let h1 = hlist![1, 2, 3.3f32, "hi2", true];
    c.bench_function("hlist_mapping_consuming", |b| {
        b.iter(|| {
            black_box(h1.map(hlist![
                |i| i + 1,
                |i| i + 2,
                |i| i + 3f32,
                |s| s,
                |b: bool| !b,
            ]))
        });
    });
}

fn hlist_mapping_non_consuming(c: &mut Criterion) {
    let h1 = hlist![1, 2, 3.3f32, "hi2", true];
    c.bench_function("hlist_mapping_non_consuming", |b| {
        b.iter(|| {
            black_box(h1.to_ref().map(hlist![
                |&i| i + 1,
                |&i| i + 2,
                |&i| i + 3f32,
                |&s| s,
                |&b: &bool| !b,
            ]))
        });
    });
}

criterion_group!(
    benches,
    creating_hlist,
    creating_tuple2,
    hlist_into_tuple2,
    hlist_into_tuple2_match,
    hlist_into_coniunctio_pat_match,
    hlist_append,
    hlist_mapping_consuming,
    hlist_mapping_non_consuming
);
