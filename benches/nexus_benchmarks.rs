//! Nexus Effect System Benchmarks
//!
//! Comprehensive benchmarks comparing Nexus effects against hand-written equivalents.
//!
//! Run with: `cargo bench --bench nexus_benchmarks --features "nexus,alloc,std"`
//!
//! ## Acceptance Criteria (from core/src/nexus/bench.rs)
//!
//! | Pattern        | Target Overhead                    |
//! |----------------|-----------------------------------|
//! | Pure           | ~0%                               |
//! | State-only     | <5% vs hand-written               |
//! | Reader-only    | <5% vs hand-written               |
//! | Error-only     | ~0% (isomorphic to Result)        |
//! | 2 effects      | 5-20%                             |
//! | 3+ effects     | 20-50%                            |

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

use ordofp_core::nexus::effects::region::{RegionVec, with_region, with_region_capacity};
use ordofp_core::nexus::prelude::*;

// =============================================================================
// Hand-Written Baselines
// =============================================================================

mod handwritten {
    //! Hand-written implementations for comparison.
    //! These represent the "optimal" code a programmer would write manually.

    /// Hand-written state monad: fn(S) -> (A, S)
    pub struct State<S, A>(Box<dyn FnOnce(S) -> (A, S)>);

    impl<S: 'static, A: 'static> State<S, A> {
        pub fn new<F: FnOnce(S) -> (A, S) + 'static>(f: F) -> Self {
            State(Box::new(f))
        }

        pub fn pure(value: A) -> Self {
            State::new(move |s| (value, s))
        }

        pub fn run(self, initial: S) -> (A, S) {
            (self.0)(initial)
        }

        pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> State<S, B> {
            State::new(move |s| {
                let (a, s2) = (self.0)(s);
                (f(a), s2)
            })
        }

        pub fn and_then<B: 'static, F: FnOnce(A) -> State<S, B> + 'static>(
            self,
            f: F,
        ) -> State<S, B> {
            State::new(move |s| {
                let (a, s2) = (self.0)(s);
                f(a).run(s2)
            })
        }
    }

    /// Returns the current state as the result, leaving the state unchanged.
    pub fn get<S: Clone + 'static>() -> State<S, S> {
        State::new(|s: S| (s.clone(), s))
    }

    /// Replaces the current state with `value`, discarding the previous state.
    pub fn put<S: 'static>(value: S) -> State<S, ()> {
        State::new(move |_| ((), value))
    }

    /// Applies `f` to the current state, replacing it with the result.
    pub fn modify<S: 'static, F: FnOnce(S) -> S + 'static>(f: F) -> State<S, ()> {
        State::new(move |s| ((), f(s)))
    }

    /// Hand-written writer: (A, Vec<W>)
    pub struct Writer<W, A> {
        pub value: A,
        pub log: Vec<W>,
    }

    impl<W, A> Writer<W, A> {
        pub fn new(value: A) -> Self {
            Writer {
                value,
                log: Vec::new(),
            }
        }

        pub fn tell(w: W) -> Writer<W, ()> {
            Writer {
                value: (),
                log: vec![w],
            }
        }

        pub fn and_then<B, F: FnOnce(A) -> Writer<W, B>>(self, f: F) -> Writer<W, B> {
            let Writer { value, mut log } = self;
            let mut next = f(value);
            log.append(&mut next.log);
            Writer {
                value: next.value,
                log,
            }
        }
    }
}

// =============================================================================
// State Effect Benchmarks
// =============================================================================

fn bench_state_vs_handwritten(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_effect");

    // Simple get/put cycle
    group.bench_function("nexus/get_put_100", |b| {
        b.iter(|| {
            let mut comp = StatefulComputation::<i32, i32>::pure(0);
            for _ in 0..100 {
                comp = comp.and_then(|_| {
                    StatefulComputation::<i32, i32>::get()
                        .and_then(|x| StatefulComputation::put(x + 1).map(move |()| x + 1))
                });
            }
            black_box(comp.run(0))
        });
    });

    group.bench_function("handwritten/get_put_100", |b| {
        b.iter(|| {
            let mut comp = handwritten::State::<i32, i32>::pure(0);
            for _ in 0..100 {
                comp = comp.and_then(|_| {
                    handwritten::get::<i32>()
                        .and_then(move |x| handwritten::put(x + 1).map(move |()| x + 1))
                });
            }
            black_box(comp.run(0))
        });
    });

    // Modify chain
    group.bench_function("nexus/modify_chain_100", |b| {
        b.iter(|| {
            let mut comp = StatefulComputation::<i32, ()>::pure(());
            for _ in 0..100 {
                comp = comp.and_then(|()| StatefulComputation::modify(|x: i32| x + 1));
            }
            black_box(comp.run(0))
        });
    });

    group.bench_function("handwritten/modify_chain_100", |b| {
        b.iter(|| {
            let mut comp = handwritten::State::<i32, ()>::pure(());
            for _ in 0..100 {
                comp = comp.and_then(|()| handwritten::modify(|x: i32| x + 1));
            }
            black_box(comp.run(0))
        });
    });

    // Direct mutation baseline (optimal)
    group.bench_function("raw/direct_mutation_100", |b| {
        b.iter(|| {
            let mut state = 0i32;
            for _ in 0..100 {
                state += 1;
            }
            black_box(((), state))
        });
    });

    group.finish();
}

// =============================================================================
// Reader Effect Benchmarks
// =============================================================================

#[derive(Clone)]
struct Config {
    base_value: i32,
    multiplier: i32,
}

fn bench_reader_vs_handwritten(c: &mut Criterion) {
    let mut group = c.benchmark_group("reader_effect");

    let config = Config {
        base_value: 42,
        multiplier: 3,
    };

    // Simple ask
    group.bench_function("nexus/ask_chain_100", |b| {
        b.iter(|| {
            let mut comp = ReaderComputation::<Config, i32>::pure(0);
            for _ in 0..100 {
                comp = comp.and_then(|acc: i32| {
                    ReaderComputation::<Config, Config>::ask()
                        .map(move |c: Config| acc + c.base_value)
                });
            }
            black_box(comp.run(&config))
        });
    });

    group.bench_function("handwritten/ask_chain_100", |b| {
        b.iter(|| {
            let config = &config;
            let mut result = 0i32;
            for _ in 0..100 {
                result += config.base_value;
            }
            black_box(result)
        });
    });

    // Asks with projection
    group.bench_function("nexus/asks_chain_100", |b| {
        b.iter(|| {
            let mut comp = ReaderComputation::<Config, i32>::pure(0);
            for _ in 0..100 {
                comp = comp.and_then(|acc| {
                    ReaderComputation::asks(|c: &Config| c.base_value * c.multiplier)
                        .map(move |v| acc + v)
                });
            }
            black_box(comp.run(&config))
        });
    });

    group.bench_function("handwritten/asks_chain_100", |b| {
        b.iter(|| {
            let config = &config;
            let mut result = 0i32;
            for _ in 0..100 {
                result += config.base_value * config.multiplier;
            }
            black_box(result)
        });
    });

    group.finish();
}

// =============================================================================
// Error Effect Benchmarks
// =============================================================================

fn bench_error_vs_handwritten(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_effect");

    // Success chain (no errors)
    group.bench_function("nexus/success_chain_100", |b| {
        b.iter(|| {
            let mut comp = ErrorComputation::<String, i32>::ok(0);
            for i in 0..100 {
                comp = comp.and_then(move |x| ErrorComputation::ok(x + i));
            }
            black_box(comp.run())
        });
    });

    group.bench_function("raw_result/success_chain_100", |b| {
        b.iter(|| {
            let mut result: Result<i32, String> = Ok(0);
            for i in 0..100 {
                result = result.map(|x| x + i);
            }
            black_box(result)
        });
    });

    // Early error termination
    group.bench_function("nexus/early_error_at_50", |b| {
        b.iter(|| {
            let mut comp = ErrorComputation::<String, i32>::ok(0);
            for i in 0..100 {
                comp = comp.and_then(move |x| {
                    if i == 50 {
                        ErrorComputation::err("error at 50".to_string())
                    } else {
                        ErrorComputation::ok(x + i)
                    }
                });
            }
            black_box(comp.run())
        });
    });

    group.bench_function("raw_result/early_error_at_50", |b| {
        b.iter(|| {
            let mut result: Result<i32, String> = Ok(0);
            for i in 0..100 {
                result = result.and_then(|x| {
                    if i == 50 {
                        Err("error at 50".to_string())
                    } else {
                        Ok(x + i)
                    }
                });
            }
            black_box(result)
        });
    });

    group.finish();
}

// =============================================================================
// Writer Effect Benchmarks
// =============================================================================

fn bench_writer_vs_handwritten(c: &mut Criterion) {
    let mut group = c.benchmark_group("writer_effect");

    // Tell chain
    group.bench_function("nexus/tell_chain_100", |b| {
        b.iter(|| {
            let mut comp = WriterComputation::<Vec<String>, ()>::pure(());
            for i in 0..100 {
                let msg = format!("log {i}");
                comp = comp.and_then(move |()| WriterComputation::tell(vec![msg]));
            }
            black_box(comp.run())
        });
    });

    group.bench_function("handwritten/tell_chain_100", |b| {
        b.iter(|| {
            let mut writer = handwritten::Writer::<String, ()>::new(());
            for i in 0..100 {
                let msg = format!("log {i}");
                writer = writer.and_then(|(): ()| handwritten::Writer::<String, ()>::tell(msg));
            }
            black_box((writer.value, writer.log))
        });
    });

    group.bench_function("raw/vec_push_100", |b| {
        b.iter(|| {
            let mut log = Vec::new();
            for i in 0..100 {
                log.push(format!("log {i}"));
            }
            black_box(((), log))
        });
    });

    group.finish();
}

// =============================================================================
// IO Effect Benchmarks
// =============================================================================

fn bench_io_vs_handwritten(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_effect");

    // Pure IO chain
    group.bench_function("nexus/io_pure_chain_100", |b| {
        b.iter(|| {
            let mut comp = IoComputation::pure(0i32);
            for _ in 0..100 {
                comp = comp.map(|x| x + 1);
            }
            black_box(comp.run())
        });
    });

    group.bench_function("raw/closure_chain_100", |b| {
        b.iter(|| {
            let mut f: Box<dyn FnOnce() -> i32> = Box::new(|| 0);
            for _ in 0..100 {
                let prev = f;
                f = Box::new(move || prev() + 1);
            }
            black_box(f())
        });
    });

    // IO with side effects simulation
    group.bench_function("nexus/io_and_then_100", |b| {
        b.iter(|| {
            let mut comp = IoComputation::pure(0i32);
            for _ in 0..100 {
                comp = comp.and_then(|x| IoComputation::pure(x + 1));
            }
            black_box(comp.run())
        });
    });

    group.finish();
}

// =============================================================================
// Region Effect Benchmarks
// =============================================================================

fn bench_region_vs_heap(c: &mut Criterion) {
    let mut group = c.benchmark_group("region_effect");

    // Small allocations
    group.bench_function("region/alloc_1000_small", |b| {
        b.iter(|| {
            with_region(|region| {
                let mut sum = 0i32;
                for i in 0..1000 {
                    let ptr = region.alloc(i);
                    sum += *ptr;
                }
                black_box(sum)
            })
        });
    });

    group.bench_function("heap/box_1000_small", |b| {
        b.iter(|| {
            let mut boxes: Vec<Box<i32>> = Vec::with_capacity(1000);
            let mut sum = 0i32;
            for i in 0..1000 {
                let b = Box::new(i);
                sum += *b;
                boxes.push(b);
            }
            black_box(sum)
        });
    });

    // RegionVec vs Vec
    group.bench_function("region/vec_push_1000", |b| {
        b.iter(|| {
            with_region_capacity(8192, |region| {
                let mut vec = RegionVec::<i32>::with_capacity(region, 1000);
                for i in 0..1000 {
                    vec.push(i);
                }
                black_box(vec.iter().sum::<i32>())
            })
        });
    });

    group.bench_function("heap/vec_push_1000", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(1000);
            for i in 0..1000 {
                vec.push(i);
            }
            black_box(vec.iter().sum::<i32>())
        });
    });

    // String allocations
    group.bench_function("region/alloc_strings_100", |b| {
        b.iter(|| {
            with_region(|region| {
                let mut total_len = 0usize;
                for i in 0..100 {
                    let s = format!("string number {i}");
                    let ptr = region.alloc_str(&s);
                    total_len += ptr.len();
                }
                black_box(total_len)
            })
        });
    });

    group.bench_function("heap/string_100", |b| {
        b.iter(|| {
            let mut strings: Vec<String> = Vec::with_capacity(100);
            let mut total_len = 0usize;
            for i in 0..100 {
                let s = format!("string number {i}");
                total_len += s.len();
                strings.push(s);
            }
            black_box(total_len)
        });
    });

    group.finish();
}

// =============================================================================
// Effect Combination Benchmarks
// =============================================================================

fn bench_effect_combinations(c: &mut Criterion) {
    let mut group = c.benchmark_group("effect_combinations");

    let config = Config {
        base_value: 10,
        multiplier: 2,
    };

    // State + Error simulation
    group.bench_function("state_error/nexus_simulation", |b| {
        b.iter(|| {
            // Simulate state+error with nested computations
            let state_comp = StatefulComputation::<i32, Result<i32, String>>::new(|s| {
                let mut state = s;
                let mut result: Result<i32, String> = Ok(0);
                for i in 0..50 {
                    if result.is_ok() {
                        state += 1;
                        result = result.map(|x| x + i);
                    }
                }
                (result, state)
            });
            black_box(state_comp.run(0))
        });
    });

    group.bench_function("state_error/handwritten", |b| {
        b.iter(|| {
            let mut state = 0i32;
            let mut result: Result<i32, String> = Ok(0);
            for i in 0..50 {
                if result.is_ok() {
                    state += 1;
                    result = result.map(|x| x + i);
                }
            }
            black_box((result, state))
        });
    });

    // Reader + Error simulation
    group.bench_function("reader_error/nexus_simulation", |b| {
        let config = &config;
        b.iter(|| {
            let reader_comp = ReaderComputation::<Config, Result<i32, String>>::new(|c| {
                let mut result: Result<i32, String> = Ok(0);
                for _ in 0..50 {
                    if result.is_ok() {
                        result = result.map(|x| x + c.base_value);
                    }
                }
                result
            });
            black_box(reader_comp.run(config))
        });
    });

    group.bench_function("reader_error/handwritten", |b| {
        let config = &config;
        b.iter(|| {
            let mut result: Result<i32, String> = Ok(0);
            for _ in 0..50 {
                if result.is_ok() {
                    result = result.map(|x| x + config.base_value);
                }
            }
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

        group.bench_with_input(
            BenchmarkId::new("state_modifications", size),
            size,
            |b, &size| {
                b.iter(move || {
                    let comp = StatefulComputation::<i64, i64>::new(move |s| {
                        let mut state = s;
                        for _ in 0..size {
                            state += 1;
                        }
                        (state, state)
                    });
                    black_box(comp.run(0))
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("region_allocations", size),
            size,
            |b, &size| {
                b.iter(|| {
                    with_region_capacity(size * 8, |region| {
                        let mut sum = 0i32;
                        for i in 0..size {
                            sum += *region.alloc(i as i32);
                        }
                        black_box(sum)
                    })
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("error_chain", size), size, |b, &size| {
            b.iter(|| {
                let mut comp = ErrorComputation::<String, i32>::ok(0);
                for i in 0..size {
                    comp = comp.map(move |x| x + i as i32);
                }
                black_box(comp.run())
            });
        });
    }

    group.finish();
}

// =============================================================================
// Real-World Scenario Benchmarks
// =============================================================================

fn bench_real_world(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world");

    // Config-based computation (Reader pattern)
    #[derive(Clone)]
    struct AppConfig {
        db_pool_size: i32,
        cache_ttl: i32,
        max_retries: i32,
    }

    let app_config = AppConfig {
        db_pool_size: 10,
        cache_ttl: 3600,
        max_retries: 3,
    };

    group.bench_function("config_reader/nexus", |b| {
        b.iter(|| {
            let comp = ReaderComputation::<AppConfig, i32>::asks(|c: &AppConfig| c.db_pool_size)
                .and_then(|pool| ReaderComputation::asks(move |c: &AppConfig| pool * c.cache_ttl))
                .and_then(|val| ReaderComputation::asks(move |c: &AppConfig| val + c.max_retries));
            black_box(comp.run(&app_config))
        });
    });

    group.bench_function("config_reader/manual", |b| {
        b.iter(|| {
            let config = &app_config;
            let pool = config.db_pool_size;
            let val = pool * config.cache_ttl;
            let result = val + config.max_retries;
            black_box(result)
        });
    });

    // Transaction-like state+error pattern
    group.bench_function("transaction/nexus_state", |b| {
        b.iter(|| {
            let comp = StatefulComputation::<i32, Result<i32, &str>>::new(|balance| {
                let mut bal = balance;

                // Simulate transaction operations
                if bal >= 100 {
                    bal -= 100; // withdraw
                    bal += 50; // deposit
                    bal -= 25; // fee
                    (Ok(bal), bal)
                } else {
                    (Err("insufficient funds"), bal)
                }
            });
            black_box(comp.run(500))
        });
    });

    group.bench_function("transaction/manual", |b| {
        b.iter(|| {
            let mut balance = 500i32;
            let result: Result<i32, &str> = if balance >= 100 {
                balance -= 100;
                balance += 50;
                balance -= 25;
                Ok(balance)
            } else {
                Err("insufficient funds")
            };
            black_box((result, balance))
        });
    });

    // Parser-like accumulation (Region for scratch space)
    group.bench_function("parser_scratch/region", |b| {
        b.iter(|| {
            with_region_capacity(4096, |region| {
                // Simulate parsing tokens into a region
                let tokens: &mut [&str] = region.alloc_slice(&[
                    "let", "x", "=", "42", ";", "let", "y", "=", "x", "+", "1", ";",
                ]);

                // "Parse" by counting identifiers
                let count = tokens
                    .iter()
                    .filter(|t| t.chars().all(char::is_alphabetic))
                    .count();
                black_box(count)
            })
        });
    });

    group.bench_function("parser_scratch/heap", |b| {
        b.iter(|| {
            let tokens: Vec<&str> = vec![
                "let", "x", "=", "42", ";", "let", "y", "=", "x", "+", "1", ";",
            ];

            let count = tokens
                .iter()
                .filter(|t| t.chars().all(char::is_alphabetic))
                .count();
            black_box(count)
        });
    });

    group.finish();
}

// =============================================================================
// Criterion Configuration
// =============================================================================

criterion_group!(
    nexus_benches,
    bench_state_vs_handwritten,
    bench_reader_vs_handwritten,
    bench_error_vs_handwritten,
    bench_writer_vs_handwritten,
    bench_io_vs_handwritten,
    bench_region_vs_heap,
    bench_effect_combinations,
    bench_throughput,
    bench_real_world,
);

criterion_main!(nexus_benches);
