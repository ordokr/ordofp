//! Labelled Universalis benchmarks.

use criterion::{Criterion, criterion_group};
use ordofp::NominataUniversalis;
use ordofp_core::field;
use ordofp_core::labelled::chars::{La, Le, Lf, Li, Lm, Ln, Lr, Ls, Lt, Underscore};
use std::hint::black_box;

#[derive(NominataUniversalis)]
struct NewUser<'a> {
    first_name: &'a str,
    last_name: &'a str,
    age: usize,
}

#[derive(NominataUniversalis)]
struct SavedUser<'a> {
    first_name: &'a str,
    last_name: &'a str,
    age: usize,
}

#[derive(NominataUniversalis)]
struct JumbledUser<'a> {
    age: usize,
    last_name: &'a str,
    first_name: &'a str,
}

#[derive(NominataUniversalis)]
struct BigStruct24Fields {
    a: i8,
    b: i32,
    c: f64,
    d: usize,
    e: String,
    f: i8,
    g: i32,
    h: f64,
    i: usize,
    j: String,
    k: i8,
    l: i32,
    m: f64,
    n: usize,
    o: String,
    p: i8,
    q: i32,
    r: f64,
    s: usize,
    t: String,
    u: i8,
    v: i32,
    w: f64,
    x: usize,
}

#[derive(NominataUniversalis)]
struct BigStruct25Fields {
    a: i8,
    b: i32,
    c: f64,
    d: usize,
    e: String,
    f: i8,
    g: i32,
    h: f64,
    i: usize,
    j: String,
    k: i8,
    l: i32,
    m: f64,
    n: usize,
    o: String,
    p: i8,
    q: i32,
    r: f64,
    s: usize,
    t: String,
    u: i8,
    v: i32,
    w: f64,
    x: usize,
    y: String,
}

#[derive(NominataUniversalis)]
struct BigStruct24FieldsReverse {
    x: usize,
    w: f64,
    v: i32,
    u: i8,
    t: String,
    s: usize,
    r: f64,
    q: i32,
    p: i8,
    o: String,
    n: usize,
    m: f64,
    l: i32,
    k: i8,
    j: String,
    i: usize,
    h: f64,
    g: i32,
    f: i8,
    e: String,
    d: usize,
    c: f64,
    b: i32,
    a: i8,
}

#[derive(NominataUniversalis)]
struct BigStruct25FieldsReverse {
    y: String,
    x: usize,
    w: f64,
    v: i32,
    u: i8,
    t: String,
    s: usize,
    r: f64,
    q: i32,
    p: i8,
    o: String,
    n: usize,
    m: f64,
    l: i32,
    k: i8,
    j: String,
    i: usize,
    h: f64,
    g: i32,
    f: i8,
    e: String,
    d: usize,
    c: f64,
    b: i32,
    a: i8,
}

impl From<BigStruct24Fields> for BigStruct24FieldsReverse {
    fn from(b: BigStruct24Fields) -> Self {
        BigStruct24FieldsReverse {
            a: b.a,
            b: b.b,
            c: b.c,
            d: b.d,
            e: b.e,
            f: b.f,
            g: b.g,
            h: b.h,
            i: b.i,
            j: b.j,
            k: b.k,
            l: b.l,
            m: b.m,
            n: b.n,
            o: b.o,
            p: b.p,
            q: b.q,
            r: b.r,
            s: b.s,
            t: b.t,
            u: b.u,
            v: b.v,
            w: b.w,
            x: b.x,
        }
    }
}

impl From<BigStruct25Fields> for BigStruct25FieldsReverse {
    fn from(b: BigStruct25Fields) -> Self {
        BigStruct25FieldsReverse {
            a: b.a,
            b: b.b,
            c: b.c,
            d: b.d,
            e: b.e,
            f: b.f,
            g: b.g,
            h: b.h,
            i: b.i,
            j: b.j,
            k: b.k,
            l: b.l,
            m: b.m,
            n: b.n,
            o: b.o,
            p: b.p,
            q: b.q,
            r: b.r,
            s: b.s,
            t: b.t,
            u: b.u,
            v: b.v,
            w: b.w,
            x: b.x,
            y: b.y,
        }
    }
}

fn build_big_struct_24fields() -> BigStruct24Fields {
    BigStruct24Fields {
        a: 10,
        b: 100,
        c: 42f64,
        d: 9001,
        e: "Hello World".to_string(),
        f: 10,
        g: 100,
        h: 42f64,
        i: 9001,
        j: "Hello World".to_string(),
        k: 10,
        l: 100,
        m: 42f64,
        n: 9001,
        o: "Hello World".to_string(),
        p: 10,
        q: 100,
        r: 42f64,
        s: 9001,
        t: "Hello World".to_string(),
        u: 10,
        v: 100,
        w: 42f64,
        x: 9001,
    }
}

fn build_big_struct_25fields() -> BigStruct25Fields {
    BigStruct25Fields {
        a: 10,
        b: 100,
        c: 42f64,
        d: 9001,
        e: "Hello World".to_string(),
        f: 10,
        g: 100,
        h: 42f64,
        i: 9001,
        j: "Hello World".to_string(),
        k: 10,
        l: 100,
        m: 42f64,
        n: 9001,
        o: "Hello World".to_string(),
        p: 10,
        q: 100,
        r: 42f64,
        s: 9001,
        t: "Hello World".to_string(),
        u: 10,
        v: 100,
        w: 42f64,
        x: 9001,
        y: "Hello World".to_string(),
    }
}

fn labelled_conversion(c: &mut Criterion) {
    c.bench_function("labelled_conversion", |b| {
        b.iter(|| {
            let n_u = NewUser {
                first_name: "Joe",
                last_name: "Schmoe",
                age: 30,
            };
            black_box(<SavedUser as NominataUniversalis>::convert_from(n_u))
        });
    });
}

fn sculpted_conversion(c: &mut Criterion) {
    c.bench_function("sculpted_conversion", |b| {
        b.iter(|| {
            let n_u = NewUser {
                first_name: "Joe",
                last_name: "Schmoe",
                age: 30,
            };
            black_box(JumbledUser::transform_from(n_u))
        });
    });
}

fn big_transform_from_24fields(c: &mut Criterion) {
    c.bench_function("big_transform_from_24fields", |b| {
        b.iter(|| {
            let j = BigStruct24FieldsReverse::transform_from(build_big_struct_24fields());
            black_box(j)
        });
    });
}

fn big_from_24fields(c: &mut Criterion) {
    c.bench_function("big_from_24fields", |b| {
        b.iter(|| {
            let j = <BigStruct24FieldsReverse as From<BigStruct24Fields>>::from(
                build_big_struct_24fields(),
            );
            black_box(j)
        });
    });
}

fn big_transform_from_25fields(c: &mut Criterion) {
    c.bench_function("big_transform_from_25fields", |b| {
        b.iter(|| {
            let j = BigStruct25FieldsReverse::transform_from(build_big_struct_25fields());
            black_box(j)
        });
    });
}

fn big_from_25fields(c: &mut Criterion) {
    c.bench_function("big_from_25fields", |b| {
        b.iter(|| {
            let j = <BigStruct25FieldsReverse as From<BigStruct25Fields>>::from(
                build_big_struct_25fields(),
            );
            black_box(j)
        });
    });
}

fn name(c: &mut Criterion) {
    let labelled_field = field!((Lf, Li, Lr, Ls, Lt, Underscore, Ln, La, Lm, Le), "Joe");
    c.bench_function("field_name", |b| b.iter(|| black_box(labelled_field.name)));
}

criterion_group!(
    benches,
    labelled_conversion,
    sculpted_conversion,
    big_transform_from_24fields,
    big_from_24fields,
    big_transform_from_25fields,
    big_from_25fields,
    name
);
