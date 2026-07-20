//! Effect System Benchmarks
//!
//! Comprehensive benchmarks for `OrdoFP`'s effect system.
//!
//! Run with: `cargo bench --bench effect_benchmarks`

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group};
use ordofp_core::arena::with_arena;
use ordofp_core::easy::{
    ask, asks, both, chain, fallback, io, modify, repeat, retry, run_with_config, run_with_state,
    state_pure, when,
};
use std::hint::black_box;

// =============================================================================
// State Effect Benchmarks
// =============================================================================

fn bench_state_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_operations");

    // Benchmark simple state increment
    group.bench_function("increment_1000", |b| {
        b.iter(|| {
            run_with_state(0i64, |counter| {
                for _ in 0..1000 {
                    *counter += 1;
                }
                black_box(*counter)
            })
        });
    });

    // Benchmark state get/put cycle
    group.bench_function("get_put_cycle_1000", |b| {
        b.iter(|| {
            run_with_state(0i64, |counter| {
                for _ in 0..1000 {
                    let val = *counter;
                    *counter = val + 1;
                }
                black_box(*counter)
            })
        });
    });

    // Benchmark state monad chain
    group.bench_function("state_monad_chain_100", |b| {
        b.iter(|| {
            let computation = (0..100).fold(state_pure::<i32, i32>(0), |acc, _| {
                acc.and_then(|x| modify(|s: i32| s + 1).then(state_pure(x + 1)))
            });
            black_box(computation.run(0))
        });
    });

    group.finish();
}

// =============================================================================
// Reader Effect Benchmarks
// =============================================================================

fn bench_reader_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("reader_operations");

    #[derive(Clone)]
    struct Config {
        value: i64,
        multiplier: i64,
    }

    let config = Config {
        value: 42,
        multiplier: 2,
    };

    // Benchmark simple config read
    group.bench_function("config_read_1000", |b| {
        b.iter(|| {
            run_with_config(&config, |cfg| {
                let mut sum = 0i64;
                for _ in 0..1000 {
                    sum += cfg.value * cfg.multiplier;
                }
                black_box(sum)
            })
        });
    });

    // Benchmark reader monad chain
    group.bench_function("reader_monad_chain_100", |b| {
        b.iter(|| {
            let reader = (0..100).fold(ask::<Config>().map(|c| c.value), |acc, _| {
                acc.and_then(|x| asks(move |c: &Config| x + c.multiplier))
            });
            black_box(reader.run(&config))
        });
    });

    group.finish();
}

// =============================================================================
// Error Handling Benchmarks
// =============================================================================

fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    // Benchmark successful chain
    group.bench_function("success_chain_100", |b| {
        b.iter(|| {
            let result: Result<i32, &str> = (0..100).try_fold(0i32, |acc, i| Ok(acc + i));
            black_box(result)
        });
    });

    // Benchmark retry logic (always succeeds on first try)
    group.bench_function("retry_success_first", |b| {
        b.iter(|| {
            let result: Result<i32, &str> = retry(3, || Ok(42));
            black_box(result)
        });
    });

    // Benchmark retry logic (succeeds on second try)
    group.bench_function("retry_success_second", |b| {
        b.iter(|| {
            let mut attempt = 0;
            let result: Result<i32, &str> = retry(3, || {
                attempt += 1;
                if attempt == 2 { Ok(42) } else { Err("not yet") }
            });
            black_box(result)
        });
    });

    // Benchmark fallback chain
    group.bench_function("fallback_chain", |b| {
        b.iter(|| {
            let result: Result<i32, &str> = fallback(|| Err("first"), || Ok(42));
            black_box(result)
        });
    });

    group.finish();
}

// =============================================================================
// IO Benchmarks
// =============================================================================

fn bench_io_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_operations");

    // Benchmark IO creation and execution
    group.bench_function("io_create_run_1000", |b| {
        b.iter(|| {
            let mut sum = 0i64;
            for i in 0..1000i64 {
                let io = io(move || i * 2);
                sum += io.run();
            }
            black_box(sum)
        });
    });

    // Benchmark IO chaining
    group.bench_function("io_chain_100", |b| {
        b.iter(|| {
            let chained = (0..100).fold(io(|| 0i64), |acc, _| acc.map(|x| x + 1));
            black_box(chained.run())
        });
    });

    // Benchmark IO and_then chain
    group.bench_function("io_and_then_100", |b| {
        b.iter(|| {
            let chained = (0..100).fold(io(|| 0i64), |acc, _| acc.and_then(|x| io(move || x + 1)));
            black_box(chained.run())
        });
    });

    group.finish();
}

// =============================================================================
// Arena Allocation Benchmarks
// =============================================================================

fn bench_arena_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("arena_allocation");

    // Benchmark arena small allocations
    group.bench_function("small_allocs_1000", |b| {
        b.iter(|| {
            with_arena(|arena| {
                let mut sum = 0i64;
                for i in 0..1000i64 {
                    let x = arena.alloc(i);
                    sum += *x;
                }
                black_box(sum)
            })
        });
    });

    // Benchmark arena mixed allocations
    group.bench_function("mixed_allocs_1000", |b| {
        b.iter(|| {
            with_arena(|arena| {
                let mut sum = 0i64;
                for i in 0..1000i64 {
                    match i % 3 {
                        0 => {
                            let x: &mut i64 = arena.alloc(i).into_mut();
                            sum += *x;
                        }
                        1 => {
                            let x: &mut (i64, i64) = arena.alloc((i, i * 2)).into_mut();
                            sum += x.0 + x.1;
                        }
                        _ => {
                            let x: &mut [u8; 16] = arena.alloc([i as u8; 16]).into_mut();
                            sum += i64::from(x[0]);
                        }
                    }
                }
                black_box(sum)
            })
        });
    });

    // Benchmark heap vs arena comparison
    group.bench_function("heap_allocs_1000", |b| {
        b.iter(|| {
            let mut sum = 0i64;
            for i in 0..1000i64 {
                let x = Box::new(i);
                sum += *x;
            }
            black_box(sum)
        });
    });

    group.finish();
}

// =============================================================================
// Combinator Benchmarks
// =============================================================================

fn bench_combinators(c: &mut Criterion) {
    let mut group = c.benchmark_group("combinators");

    // Benchmark chain combinator
    group.bench_function("chain_3_steps", |b| {
        b.iter(|| {
            let result = chain(|| 1, |x| x + 10, |x| x * 2);
            black_box(result)
        });
    });

    // Benchmark both combinator
    group.bench_function("both_parallel", |b| {
        b.iter(|| {
            let (a, b) = both(|| 42, || 100);
            black_box(a + b)
        });
    });

    // Benchmark repeat
    group.bench_function("repeat_1000", |b| {
        b.iter(|| {
            let results: Vec<i32> = repeat(1000, |i| (i * i) as i32);
            black_box(results.len())
        });
    });

    // Benchmark when conditional
    group.bench_function("when_true", |b| {
        b.iter(|| {
            let result = when(true, || 42, || 0);
            black_box(result)
        });
    });

    group.bench_function("when_false", |b| {
        b.iter(|| {
            let result = when(false, || 42, || 0);
            black_box(result)
        });
    });

    group.finish();
}

// =============================================================================
// Throughput Benchmarks
// =============================================================================

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    for size in &[100, 1000, 10000] {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("state_updates", size), size, |b, &size| {
            b.iter(|| {
                run_with_state(0i64, |counter| {
                    for _ in 0..size {
                        *counter += 1;
                    }
                    black_box(*counter)
                })
            });
        });

        group.bench_with_input(BenchmarkId::new("io_operations", size), size, |b, &size| {
            b.iter(|| {
                let mut sum = 0i64;
                for i in 0..i64::from(size) {
                    let io = io(move || i);
                    sum += io.run();
                }
                black_box(sum)
            });
        });
    }

    group.finish();
}

// =============================================================================
// Criterion Groups
// =============================================================================

criterion_group!(
    benches,
    bench_state_operations,
    bench_reader_operations,
    bench_error_handling,
    bench_io_operations,
    bench_arena_allocation,
    bench_combinators,
    bench_throughput,
);
