//! Transfigure benchmarks.
//!
//! Compares manual field-by-field conversion vs automatic transfiguration.

use criterion::{Criterion, criterion_group};
use ordofp::NominataUniversalis;
use ordofp::labelled::Transfigurator;
use std::hint::black_box;

// Simple layer for benchmarking
#[derive(NominataUniversalis)]
struct SimpleLayer {
    a: i8,
    b: i32,
    c: f64,
    d: usize,
    e: String,
}

#[derive(NominataUniversalis)]
struct SimpleLayerReversed {
    e: String,
    d: usize,
    c: f64,
    b: i32,
    a: i8,
}

impl SimpleLayer {
    fn new() -> Self {
        SimpleLayer {
            a: 6,
            b: 7,
            c: 8f64,
            d: 9,
            e: "hello world".to_string(),
        }
    }
}

// Manual conversion (as a baseline)
fn manual_convert(src: SimpleLayer) -> SimpleLayerReversed {
    SimpleLayerReversed {
        a: src.a,
        b: src.b,
        c: src.c,
        d: src.d,
        e: src.e,
    }
}

// Nested layers for deeper benchmarking
#[derive(NominataUniversalis)]
struct NestedLayer {
    inner: SimpleLayer,
    x: i32,
    y: f64,
}

#[derive(NominataUniversalis)]
struct NestedLayerReversed {
    y: f64,
    x: i32,
    inner: SimpleLayerReversed,
}

impl NestedLayer {
    fn new() -> Self {
        NestedLayer {
            inner: SimpleLayer::new(),
            x: 42,
            y: std::f64::consts::PI,
        }
    }
}

fn manual_nested_convert(src: NestedLayer) -> NestedLayerReversed {
    NestedLayerReversed {
        inner: manual_convert(src.inner),
        x: src.x,
        y: src.y,
    }
}

// Double nested for even deeper testing
#[derive(NominataUniversalis)]
struct DeepLayer {
    nested: NestedLayer,
    z: String,
}

#[derive(NominataUniversalis)]
struct DeepLayerReversed {
    z: String,
    nested: NestedLayerReversed,
}

impl DeepLayer {
    fn new() -> Self {
        DeepLayer {
            nested: NestedLayer::new(),
            z: "deep".to_string(),
        }
    }
}

fn manual_deep_convert(src: DeepLayer) -> DeepLayerReversed {
    DeepLayerReversed {
        nested: manual_nested_convert(src.nested),
        z: src.z,
    }
}

fn bench_manual_simple(c: &mut Criterion) {
    c.bench_function("manual_simple_convert", |b| {
        b.iter(|| {
            let src = SimpleLayer::new();
            black_box(manual_convert(src))
        });
    });
}

fn bench_transfigure_simple(c: &mut Criterion) {
    c.bench_function("transfigure_simple_convert", |b| {
        b.iter(|| {
            let src = SimpleLayer::new();
            let result: SimpleLayerReversed = src.transfigure();
            black_box(result)
        });
    });
}

fn bench_manual_nested(c: &mut Criterion) {
    c.bench_function("manual_nested_convert", |b| {
        b.iter(|| {
            let src = NestedLayer::new();
            black_box(manual_nested_convert(src))
        });
    });
}

fn bench_transfigure_nested(c: &mut Criterion) {
    c.bench_function("transfigure_nested_convert", |b| {
        b.iter(|| {
            let src = NestedLayer::new();
            let result: NestedLayerReversed = src.transfigure();
            black_box(result)
        });
    });
}

fn bench_manual_deep(c: &mut Criterion) {
    c.bench_function("manual_deep_convert", |b| {
        b.iter(|| {
            let src = DeepLayer::new();
            black_box(manual_deep_convert(src))
        });
    });
}

fn bench_transfigure_deep(c: &mut Criterion) {
    c.bench_function("transfigure_deep_convert", |b| {
        b.iter(|| {
            let src = DeepLayer::new();
            let result: DeepLayerReversed = src.transfigure();
            black_box(result)
        });
    });
}

criterion_group!(
    benches,
    bench_manual_simple,
    bench_transfigure_simple,
    bench_manual_nested,
    bench_transfigure_nested,
    bench_manual_deep,
    bench_transfigure_deep
);
