//! Universalis conversion benchmarks.

use criterion::{Criterion, criterion_group};
use ordofp::Universalis;
use std::hint::black_box;

#[derive(Universalis)]
struct NewUser<'a> {
    first_name: &'a str,
    last_name: &'a str,
    age: usize,
}

#[derive(Universalis)]
struct SavedUser<'a> {
    first_name: &'a str,
    last_name: &'a str,
    age: usize,
}

fn universalis_conversion(c: &mut Criterion) {
    c.bench_function("Universalis_conversion", |b| {
        b.iter(|| {
            let n_u = NewUser {
                first_name: "Joe",
                last_name: "Schmoe",
                age: 30,
            };
            black_box(SavedUser::convert_from(n_u))
        });
    });
}

criterion_group!(benches, universalis_conversion);
