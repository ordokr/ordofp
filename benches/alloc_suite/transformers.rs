use criterion::{Criterion, criterion_group};
use ordofp_core::transformers::OptionT;
use std::hint::black_box;

fn bench_option_t_map(c: &mut Criterion) {
    let data: Vec<Option<i32>> = (0..1000)
        .map(|i| if i % 2 == 0 { Some(i) } else { None })
        .collect();

    c.bench_function("OptionT::map", |b| {
        b.iter(|| {
            let t = OptionT::from_vec(data.clone()); // Cloning to ensure fair start
            let res = t.map(|x| x + 1);
            black_box(res)
        });
    });

    c.bench_function("Manual::map", |b| {
        b.iter(|| {
            let res: Vec<Option<i32>> = data
                .clone()
                .into_iter()
                .map(|opt| opt.map(|x| x + 1))
                .collect();
            black_box(res)
        });
    });
}

fn bench_option_t_flat_map(c: &mut Criterion) {
    let data: Vec<Option<i32>> = (0..1000)
        .map(|i| if i % 2 == 0 { Some(i) } else { None })
        .collect();

    c.bench_function("OptionT::flat_map", |b| {
        b.iter(|| {
            let t = OptionT::from_vec(data.clone());
            let res = t.flat_map(|x| OptionT::some_vec(x * 2));
            black_box(res)
        });
    });

    c.bench_function("Manual::flat_map", |b| {
        b.iter(|| {
            let res: Vec<Option<i32>> = data
                .clone()
                .into_iter()
                .flat_map(|opt| match opt {
                    None => vec![None],
                    Some(x) => vec![Some(x * 2)],
                })
                .collect();
            black_box(res)
        });
    });
}

criterion_group!(benches, bench_option_t_map, bench_option_t_flat_map);
