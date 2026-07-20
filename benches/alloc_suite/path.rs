//! Path/lens benchmarks.

use criterion::{Criterion, criterion_group};
use ordofp::NominataUniversalis;
use ordofp_macros::path;
use std::hint::black_box;

#[derive(NominataUniversalis)]
struct Inner5 {
    v: isize,
}

#[derive(NominataUniversalis)]
struct Inner4 {
    v: Inner5,
}

#[derive(NominataUniversalis)]
struct Inner3 {
    v: Inner4,
}

#[derive(NominataUniversalis)]
struct Inner2 {
    v: Inner3,
}

#[derive(NominataUniversalis)]
struct Outer {
    v: Inner2,
}

impl Outer {
    fn new() -> Outer {
        Outer {
            v: Inner2 {
                v: Inner3 {
                    v: Inner4 { v: Inner5 { v: 3 } },
                },
            },
        }
    }
}

fn normal_path_read_value(c: &mut Criterion) {
    c.bench_function("normal_path_read_value", |b| {
        b.iter(|| {
            let o = Outer::new();
            let v = o.v.v.v.v.v;
            let r = v + 1;
            black_box(r)
        });
    });
}

fn lens_path_read_value(c: &mut Criterion) {
    let p = path!(v.v.v.v.v);
    c.bench_function("lens_path_read_value", |b| {
        b.iter(|| {
            let o = Outer::new();
            let v = p.get(o);
            let r = v + 1;
            black_box(r)
        });
    });
}

fn normal_path_read_ref(c: &mut Criterion) {
    c.bench_function("normal_path_read_ref", |b| {
        b.iter(|| {
            let o = Outer::new();
            let v = &o.v.v.v.v.v;
            let r = v + 1;
            black_box(r)
        });
    });
}

fn lens_path_read_ref(c: &mut Criterion) {
    let p = path!(v.v.v.v.v);
    c.bench_function("lens_path_read_ref", |b| {
        b.iter(|| {
            let o = Outer::new();
            let v = p.get(&o);
            let r = v + 1;
            black_box(r)
        });
    });
}

fn normal_path_read_mut(c: &mut Criterion) {
    c.bench_function("normal_path_read_mut", |b| {
        b.iter(|| {
            let mut o = Outer::new();
            let v = &mut o.v.v.v.v.v;
            *v = 999;
            let r = *v + 1;
            black_box(r)
        });
    });
}

fn lens_path_read_mut(c: &mut Criterion) {
    let p = path!(v.v.v.v.v);
    c.bench_function("lens_path_read_mut", |b| {
        b.iter(|| {
            let mut o = Outer::new();
            *p.get(&mut o) = 9999;
            let r = p.get(&o) + 1;
            black_box(r)
        });
    });
}

criterion_group!(
    benches,
    normal_path_read_value,
    lens_path_read_value,
    normal_path_read_ref,
    lens_path_read_ref,
    normal_path_read_mut,
    lens_path_read_mut
);
