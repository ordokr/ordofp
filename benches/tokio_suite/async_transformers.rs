//! Async Transformer Benchmarks
//!
//! Compares `OrdoFP` async transformers against raw async/await patterns
//! to measure overhead. Target: <10% overhead.

#![cfg(feature = "tokio")]

use criterion::{BenchmarkId, Criterion, criterion_group};
use std::hint::black_box;
use tokio::runtime::Runtime;

// ============================================================================
// Futurus Benchmarks
// ============================================================================

fn bench_futurus_vs_raw(c: &mut Criterion) {
    use ordofp_core::async_core::Futurus;

    let rt = Runtime::new().expect("failed to construct Tokio runtime for Futurus benchmark");

    let mut group = c.benchmark_group("Futurus vs Raw Async");

    // Pure value creation and await
    group.bench_function("Futurus::purus", |b| {
        b.iter(|| {
            rt.block_on(async {
                let fut = Futurus::purus(black_box(42));
                black_box(fut.await)
            })
        });
    });

    group.bench_function("async block pure", |b| {
        b.iter(|| {
            rt.block_on(async {
                let fut = async { black_box(42) };
                black_box(fut.await)
            })
        });
    });

    // Map operation
    group.bench_function("Futurus::fmap", |b| {
        b.iter(|| {
            rt.block_on(async {
                let fut = Futurus::purus(21).fmap(|x| x * 2);
                black_box(fut.await)
            })
        });
    });

    group.bench_function("async block map", |b| {
        b.iter(|| {
            rt.block_on(async {
                let x = async { 21 }.await;
                black_box(x * 2)
            })
        });
    });

    // Chain of operations
    group.bench_function("Futurus::chain x5", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = Futurus::purus(1)
                    .fmap(|x| x + 1)
                    .flat_map(|x| Futurus::purus(x * 2))
                    .fmap(|x| x + 1)
                    .flat_map(|x| Futurus::purus(x * 2))
                    .fmap(|x| x + 1);
                black_box(result.await)
            })
        });
    });

    group.bench_function("async block chain x5", |b| {
        b.iter(|| {
            rt.block_on(async {
                let x = 1;
                let x = x + 1;
                let x = x * 2;
                let x = x + 1;
                let x = x * 2;
                let x = x + 1;
                black_box(x)
            })
        });
    });

    group.finish();
}

// ============================================================================
// OptionTAsync Benchmarks
// ============================================================================

fn bench_option_t_async(c: &mut Criterion) {
    use ordofp_core::transformers::async_transforms::OptionTAsync;

    let rt = Runtime::new().expect("failed to construct Tokio runtime for OptionTAsync benchmark");

    let mut group = c.benchmark_group("OptionTAsync vs Raw Option");

    // Some path
    group.bench_function("OptionTAsync::some chain", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = OptionTAsync::some(10)
                    .fmap(|x| x * 2)
                    .flat_map(|x| OptionTAsync::some(x + 1))
                    .fmap(|x| x * 2)
                    .run()
                    .await;
                black_box(result)
            })
        });
    });

    group.bench_function("Option chain manual", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = Some(10).map(|x| x * 2).map(|x| x + 1).map(|x| x * 2);
                black_box(result)
            })
        });
    });

    // None short-circuit
    group.bench_function("OptionTAsync::none short-circuit", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = OptionTAsync::<i32>::none()
                    .fmap(|x| x * 2)
                    .flat_map(|x| OptionTAsync::some(x + 1))
                    .fmap(|x| x * 2)
                    .run()
                    .await;
                black_box(result)
            })
        });
    });

    group.bench_function("Option::None short-circuit", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = None::<i32>.map(|x| x * 2).map(|x| x + 1).map(|x| x * 2);
                black_box(result)
            })
        });
    });

    group.finish();
}

// ============================================================================
// EitherTAsync Benchmarks
// ============================================================================

fn bench_either_t_async(c: &mut Criterion) {
    use ordofp_core::transformers::async_transforms::EitherTAsync;

    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("EitherTAsync vs Raw Result");

    // Ok path
    group.bench_function("EitherTAsync::right chain", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = EitherTAsync::<String, i32>::right(10)
                    .fmap(|x| x * 2)
                    .flat_map(|x| EitherTAsync::right(x + 1))
                    .fmap(|x| x * 2)
                    .run()
                    .await;
                black_box(result)
            })
        });
    });

    group.bench_function("Result::Ok chain manual", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result: Result<i32, String> =
                    Ok(10).map(|x| x * 2).map(|x| x + 1).map(|x| x * 2);
                black_box(result)
            })
        });
    });

    // Error short-circuit
    group.bench_function("EitherTAsync::left short-circuit", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = EitherTAsync::<String, i32>::left("error".to_string())
                    .fmap(|x| x * 2)
                    .flat_map(|x| EitherTAsync::right(x + 1))
                    .fmap(|x| x * 2)
                    .run()
                    .await;
                black_box(result)
            })
        });
    });

    group.bench_function("Result::Err short-circuit", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result: Result<i32, String> = Err("error".to_string())
                    .map(|x: i32| x * 2)
                    .map(|x: i32| x + 1)
                    .map(|x: i32| x * 2);
                black_box(result)
            })
        });
    });

    group.finish();
}

// ============================================================================
// StatusAsync (State) Benchmarks
// ============================================================================

fn bench_status_async(c: &mut Criterion) {
    use ordofp_core::transformers::async_transforms::StatusAsync;

    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("StatusAsync vs Manual State");

    // State modification chain
    group.bench_function("StatusAsync::modify chain", |b| {
        b.iter(|| {
            rt.block_on(async {
                let computation = StatusAsync::<i32, ()>::modify(|s| s + 1)
                    .flat_map(|()| StatusAsync::<i32, ()>::modify(|s| s * 2))
                    .flat_map(|()| StatusAsync::<i32, ()>::modify(|s| s + 1))
                    .flat_map(|()| StatusAsync::<i32, i32>::get());

                let (final_state, _) = computation.run(0).await;
                black_box(final_state)
            })
        });
    });

    group.bench_function("Manual state threading", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut state = 0;
                state += 1;
                state *= 2;
                state += 1;
                black_box(state)
            })
        });
    });

    group.finish();
}

// ============================================================================
// Flumen (Stream) Benchmarks
// ============================================================================

fn bench_flumen(c: &mut Criterion) {
    use ordofp_core::async_core::Flumen;

    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("Flumen vs Iterator");

    for size in &[10, 100, 1000] {
        let data: Vec<i32> = (0..*size).collect();

        group.bench_with_input(
            BenchmarkId::new("Flumen::fmap+filter+fold", size),
            &data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        let result = Flumen::from_iterator(data.clone())
                            .fmap(|x| x * 2)
                            .filter(|x| x % 4 == 0)
                            .fold(0, |acc, x| acc + x)
                            .await;
                        black_box(result)
                    })
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Iterator::map+filter+fold", size),
            &data,
            |b, data| {
                b.iter(|| {
                    let result: i32 = data.iter().map(|x| x * 2).filter(|x| x % 4 == 0).sum();
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// TraversableAsync Benchmarks
// ============================================================================

fn bench_traversable_async(c: &mut Criterion) {
    use ordofp_core::async_core::TraversableAsync;

    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("TraversableAsync vs Loop");

    for size in &[10, 100, 1000] {
        let data: Vec<i32> = (0..*size).collect();

        group.bench_with_input(
            BenchmarkId::new("Vec::traverse_async", size),
            &data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        let result = data.clone().traverse_async(|x| async move { x * 2 }).await;
                        black_box(result)
                    })
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Manual async loop", size),
            &data,
            |b, data| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut result = Vec::with_capacity(data.len());
                        for x in data {
                            result.push(x * 2);
                        }
                        black_box(result)
                    })
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Async Macro Benchmarks
// ============================================================================

fn bench_async_macros(c: &mut Criterion) {
    use ordofp_core::{chain_async, compose_async, pipe_async};

    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("Async Macros vs Manual");

    async fn add_one(x: i32) -> i32 {
        x + 1
    }
    async fn double(x: i32) -> i32 {
        x * 2
    }
    async fn subtract_three(x: i32) -> i32 {
        x - 3
    }

    // pipe_async!
    group.bench_function("pipe_async! x3", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = pipe_async!(black_box(10), add_one, double, subtract_three).await;
                black_box(result)
            })
        });
    });

    group.bench_function("manual pipe x3", |b| {
        b.iter(|| {
            rt.block_on(async {
                let x = black_box(10);
                let x = add_one(x).await;
                let x = double(x).await;
                let x = subtract_three(x).await;
                black_box(x)
            })
        });
    });

    // chain_async!
    group.bench_function("chain_async! x3", |b| {
        b.iter(|| {
            rt.block_on(async {
                let chained = chain_async!(add_one, double, subtract_three);
                let result = chained(black_box(10)).await;
                black_box(result)
            })
        });
    });

    // compose_async!
    group.bench_function("compose_async! x3", |b| {
        b.iter(|| {
            rt.block_on(async {
                let composed = compose_async!(add_one, double, subtract_three);
                let result = composed(black_box(10)).await;
                black_box(result)
            })
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_futurus_vs_raw,
    bench_option_t_async,
    bench_either_t_async,
    bench_status_async,
    bench_flumen,
    bench_traversable_async,
    bench_async_macros
);
