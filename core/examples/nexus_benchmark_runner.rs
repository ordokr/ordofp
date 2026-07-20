//! Nexus Effect System Benchmark Runner
//!
//! Runs performance benchmarks comparing Nexus effects against hand-written equivalents.
//!
//! # Running
//!
//! ```bash
//! cargo run --example nexus_benchmark_runner --features "nexus,std" --release
//! ```

#[cfg(all(feature = "nexus", feature = "std"))]
use std::time::{Duration, Instant};

#[cfg(feature = "nexus")]
use ordofp_core::nexus::effects::error::ErrorComputation;
#[cfg(feature = "nexus")]
use ordofp_core::nexus::effects::io::IoComputation;
#[cfg(feature = "nexus")]
use ordofp_core::nexus::effects::reader::ReaderComputation;
#[cfg(feature = "nexus")]
use ordofp_core::nexus::effects::region::{RegionVec, with_region};
#[cfg(feature = "nexus")]
use ordofp_core::nexus::effects::writer::WriterComputation;

// Optimized implementations
#[cfg(feature = "nexus")]
use ordofp_core::nexus::optim::{
    IoOp, IoOpExt, ReaderOp, ReaderOpExt, StateOp, StateOpExt, WriterOp, WriterOpExt, asks_reader,
    get_op, modify_op, pure_io, put_op, tell_writer,
};

#[cfg(all(feature = "nexus", feature = "std"))]
fn black_box<T>(x: T) -> T {
    std::hint::black_box(x)
}

#[cfg(all(feature = "nexus", feature = "std"))]
struct BenchResult {
    name: &'static str,
    nexus_ns: u64,
    baseline_ns: u64,
    overhead_pct: f64,
    target_pct: f64,
    passed: bool,
}

#[cfg(all(feature = "nexus", feature = "std"))]
impl BenchResult {
    fn new(name: &'static str, nexus_ns: u64, baseline_ns: u64, target_pct: f64) -> Self {
        let overhead_pct = if baseline_ns > 0 {
            ((nexus_ns as f64 - baseline_ns as f64) / baseline_ns as f64) * 100.0
        } else {
            0.0
        };
        BenchResult {
            name,
            nexus_ns,
            baseline_ns,
            overhead_pct,
            target_pct,
            passed: overhead_pct <= target_pct,
        }
    }
}

#[cfg(all(feature = "nexus", feature = "std"))]
fn measure<F: FnMut()>(mut f: F, iterations: u64) -> Duration {
    // Warmup
    for _ in 0..1000 {
        f();
        black_box(());
    }

    let start = Instant::now();
    for _ in 0..iterations {
        f();
        black_box(());
    }
    start.elapsed()
}

// =============================================================================
// State Effect Benchmarks
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_state_get_put() -> BenchResult {
    const ITERATIONS: u64 = 100_000;

    // Hand-written baseline
    let baseline = measure(
        || {
            let state: i64 = 42;
            let (result, _new_state) = (state, state + 1);
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    // Nexus State - using optimized trait-based API (zero allocation)
    let nexus = measure(
        || {
            let op = get_op().and_then_op(|x: i64| put_op(x + 1).map_op(move |()| x));
            let (result, _) = op.run_op(42i64);
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "state_get_put",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        50.0,
    )
}

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_state_chain() -> BenchResult {
    const ITERATIONS: u64 = 10_000;

    // Hand-written baseline
    let baseline = measure(
        || {
            let mut state: i64 = 0;
            let mut result: i64 = 0;
            for _ in 0..100 {
                state += 1;
                result += 1;
            }
            black_box((result, state));
        },
        ITERATIONS,
    );

    // Nexus State chain - using optimized trait-based API
    // Chain of 10 operations, run 10 times = 100 total operations
    let nexus = measure(
        || {
            let mut state = 0i64;
            for _ in 0..10 {
                let op = modify_op(|x: i64| x + 1)
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1));
                let ((), new_state) = op.run_op(state);
                state = new_state;
            }
            black_box((state, state));
        },
        ITERATIONS,
    );

    BenchResult::new(
        "state_chain_100",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        100.0,
    )
}

// =============================================================================
// Reader Effect Benchmarks
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_reader_asks() -> BenchResult {
    const ITERATIONS: u64 = 100_000;

    #[derive(Clone)]
    struct Config {
        value: i64,
        multiplier: i64,
    }
    let config = Config {
        value: 42,
        multiplier: 2,
    };

    // Hand-written baseline
    let baseline = measure(
        || {
            let result = config.value * config.multiplier;
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    // Nexus Reader
    let nexus = measure(
        || {
            let comp = ReaderComputation::<Config, i64>::asks(|c: &Config| c.value * c.multiplier);
            let result = comp.run(&config);
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "reader_asks",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        50.0,
    )
}

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_reader_chain() -> BenchResult {
    const ITERATIONS: u64 = 10_000;

    #[derive(Clone)]
    struct Env {
        value: i64,
    }
    let env = Env { value: 1 };

    // Hand-written baseline
    let baseline = measure(
        || {
            let mut acc: i64 = 0;
            for _ in 0..100 {
                acc += env.value;
            }
            black_box(acc);
        },
        ITERATIONS,
    );

    // Nexus Reader chain - using optimized trait-based API
    // Chain of 10 operations, run 10 times = 100 total operations
    let nexus = measure(
        || {
            let mut result = 0i64;
            for _ in 0..10 {
                let op = asks_reader(move |e: &Env| result + e.value)
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value));
                result = op.run_reader(&env);
            }
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "reader_chain_100",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        100.0,
    )
}

// =============================================================================
// Optimized Reader Benchmarks (ReaderOp trait)
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_reader_op_chain() -> BenchResult {
    const ITERATIONS: u64 = 10_000;
    const CHAIN_LEN: usize = 100;

    #[derive(Clone)]
    struct Env {
        value: i64,
    }
    let env = Env { value: 1 };

    // Hand-written baseline
    let baseline = measure(
        || {
            let mut acc: i64 = 0;
            for _ in 0..CHAIN_LEN {
                acc += env.value;
            }
            black_box(acc);
        },
        ITERATIONS,
    );

    // Optimized ReaderOp chain (fully inlined, no heap allocation)
    let nexus = measure(
        || {
            // Build chain in groups of 10
            let mut result = 0i64;
            for _ in 0..10 {
                let op = asks_reader(|e: &Env| e.value)
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value))
                    .and_then_reader(|acc| asks_reader(move |e: &Env| acc + e.value));
                result += op.run_reader(&env);
            }
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "reader_op_chain",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        50.0,
    )
}

// =============================================================================
// Error Effect Benchmarks
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_error_ok() -> BenchResult {
    const ITERATIONS: u64 = 100_000;

    // Hand-written baseline (raw Result)
    let baseline = measure(
        || {
            let result: Result<i64, &str> = Ok(42);
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    // Nexus Error
    let nexus = measure(
        || {
            let comp = ErrorComputation::<&str, i64>::ok(42);
            let result = comp.run();
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "error_ok",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        10.0,
    )
}

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_error_chain() -> BenchResult {
    const ITERATIONS: u64 = 10_000;
    const CHAIN_LEN: usize = 100;

    // Hand-written baseline
    let baseline = measure(
        || {
            let mut result: Result<i64, &str> = Ok(0);
            for _ in 0..CHAIN_LEN {
                result = result.map(|x| x + 1);
            }
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    // Nexus Error chain
    let nexus = measure(
        || {
            let mut comp = ErrorComputation::<&str, i64>::ok(0);
            for _ in 0..CHAIN_LEN {
                comp = comp.and_then(|x| ErrorComputation::ok(x + 1));
            }
            let result = comp.run();
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "error_chain_100",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        20.0,
    )
}

// =============================================================================
// Writer Effect Benchmarks
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_writer_tell() -> BenchResult {
    const ITERATIONS: u64 = 50_000;

    // Hand-written baseline
    let baseline = measure(
        || {
            let log = vec!["entry".to_string()];
            black_box(((), log));
        },
        ITERATIONS,
    );

    // Nexus Writer
    let nexus = measure(
        || {
            let comp = WriterComputation::<Vec<String>, ()>::tell(vec!["entry".to_string()]);
            let result = comp.run();
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "writer_tell",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        50.0,
    )
}

// =============================================================================
// Optimized Writer Benchmarks (WriterOp trait)
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_writer_op_tell() -> BenchResult {
    const ITERATIONS: u64 = 50_000;

    // Hand-written baseline - create a vec with one element
    let baseline = measure(
        || {
            let log = vec!["entry".to_string()];
            black_box(((), log));
        },
        ITERATIONS,
    );

    // Optimized WriterOp
    let nexus = measure(
        || {
            let op = tell_writer(vec!["entry".to_string()]);
            let result = op.run_writer();
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "writer_op_tell",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        20.0,
    )
}

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_writer_op_chain() -> BenchResult {
    const ITERATIONS: u64 = 10_000;

    // Hand-written baseline - combine multiple vecs (fair comparison)
    let baseline = measure(
        || {
            let mut log = vec![0i32];
            log.extend(vec![1]);
            log.extend(vec![2]);
            log.extend(vec![3]);
            log.extend(vec![4]);
            log.extend(vec![5]);
            log.extend(vec![6]);
            log.extend(vec![7]);
            log.extend(vec![8]);
            log.extend(vec![9]);
            black_box(((), log));
        },
        ITERATIONS,
    );

    // Optimized WriterOp chain
    let nexus = measure(
        || {
            let op = tell_writer(vec![0i32])
                .then_writer(tell_writer(vec![1]))
                .then_writer(tell_writer(vec![2]))
                .then_writer(tell_writer(vec![3]))
                .then_writer(tell_writer(vec![4]))
                .then_writer(tell_writer(vec![5]))
                .then_writer(tell_writer(vec![6]))
                .then_writer(tell_writer(vec![7]))
                .then_writer(tell_writer(vec![8]))
                .then_writer(tell_writer(vec![9]));
            let result = op.run_writer();
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "writer_op_chain",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        50.0,
    )
}

// =============================================================================
// IO Effect Benchmarks
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_io_pure() -> BenchResult {
    const ITERATIONS: u64 = 100_000;

    // Hand-written baseline (raw closure)
    let baseline = measure(
        || {
            let f = || 42i64;
            black_box(f());
        },
        ITERATIONS,
    );

    // Nexus IO
    let nexus = measure(
        || {
            let comp = IoComputation::pure(42i64);
            black_box(comp.run());
        },
        ITERATIONS,
    );

    BenchResult::new(
        "io_pure",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        50.0,
    )
}

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_io_chain() -> BenchResult {
    const ITERATIONS: u64 = 10_000;

    // Hand-written baseline
    let baseline = measure(
        || {
            let mut result = 0i64;
            for _ in 0..100 {
                result += 1;
            }
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    // Nexus IO chain - using optimized trait-based API
    // Chain of 10 operations, run 10 times = 100 total operations
    let nexus = measure(
        || {
            let mut result = 0i64;
            for _ in 0..10 {
                let op = pure_io(result)
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1));
                result = op.run_io();
            }
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "io_chain_100",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        100.0,
    )
}

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_io_op_pure() -> BenchResult {
    const ITERATIONS: u64 = 100_000;

    // Hand-written baseline (raw closure)
    let baseline = measure(
        || {
            let f = || 42i64;
            black_box(f());
        },
        ITERATIONS,
    );

    // Optimized IoOp pure (no boxing)
    let nexus = measure(
        || {
            let op = pure_io(42i64);
            black_box(op.run_io());
        },
        ITERATIONS,
    );

    BenchResult::new(
        "io_op_pure",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        10.0,
    )
}

// =============================================================================
// Optimized State Benchmarks (StateOp trait)
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_state_op_chain() -> BenchResult {
    const ITERATIONS: u64 = 10_000;
    const CHAIN_LEN: i64 = 100;

    // Hand-written baseline
    let baseline = measure(
        || {
            let mut state: i64 = 0;
            let mut result: i64 = 0;
            for _ in 0..CHAIN_LEN {
                state += 1;
                result += 1;
            }
            black_box((result, state));
        },
        ITERATIONS,
    );

    // Optimized StateOp chain (fully inlined, no heap allocation)
    let nexus = measure(
        || {
            // Run 10 times to get 100 modifications
            let mut state = 0i64;
            for _ in 0..10 {
                let op = modify_op(|x: i64| x + 1)
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1))
                    .and_then_op(|()| modify_op(|x: i64| x + 1));
                let ((), new_state) = op.run_op(state);
                state = new_state;
            }
            black_box((state, state));
        },
        ITERATIONS,
    );

    BenchResult::new(
        "state_op_chain",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        50.0,
    )
}

// =============================================================================
// Optimized IO Benchmarks (IoOp trait)
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_io_op_chain() -> BenchResult {
    const ITERATIONS: u64 = 10_000;
    const CHAIN_LEN: usize = 100;

    // Hand-written baseline
    let baseline = measure(
        || {
            let mut result = 0i64;
            for _ in 0..CHAIN_LEN {
                result += 1;
            }
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    // Optimized IoOp chain (fully inlined, no heap allocation)
    let nexus = measure(
        || {
            // Build chain of 100 operations in groups of 10
            let mut result = 0i64;
            for _ in 0..10 {
                let op = pure_io(result)
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1))
                    .and_then_io(|x| pure_io(x + 1));
                result = op.run_io();
            }
            let _ = black_box(result);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "io_op_chain",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        50.0,
    )
}

// =============================================================================
// Region Effect Benchmarks
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_region_alloc() -> BenchResult {
    const ITERATIONS: u64 = 1_000;
    const ALLOCS: usize = 1000; // More allocations to amortize region setup cost

    // Hand-written baseline (Box) - many allocations
    let baseline = measure(
        || {
            let mut boxes: Vec<Box<i64>> = Vec::with_capacity(ALLOCS);
            for i in 0..ALLOCS {
                boxes.push(Box::new(i as i64));
            }
            let sum: i64 = boxes.iter().map(|b| **b).sum();
            black_box(sum);
            // boxes are dropped here - this is part of the cost
        },
        ITERATIONS,
    );

    // Nexus Region - batch allocation in arena
    let nexus = measure(
        || {
            let sum = with_region(|region| {
                let mut sum = 0i64;
                for i in 0..ALLOCS {
                    let ptr = region.alloc(i as i64);
                    sum += *ptr;
                }
                sum
                // Region is freed all at once here
            });
            black_box(sum);
        },
        ITERATIONS,
    );

    // Region should be faster due to batch deallocation
    BenchResult::new(
        "region_alloc_1k",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        50.0,
    )
}

#[cfg(all(feature = "nexus", feature = "std"))]
fn bench_region_vec() -> BenchResult {
    const ITERATIONS: u64 = 10_000;
    const SIZE: usize = 1000;

    // Hand-written baseline (std Vec)
    let baseline = measure(
        || {
            let mut v = Vec::with_capacity(SIZE);
            for i in 0..SIZE {
                v.push(i as i64);
            }
            let sum: i64 = v.iter().sum();
            black_box(sum);
        },
        ITERATIONS,
    );

    // Nexus RegionVec
    let nexus = measure(
        || {
            let sum = with_region(|region| {
                let mut v = RegionVec::<i64>::with_capacity(region, SIZE);
                for i in 0..SIZE {
                    v.push(i as i64);
                }
                v.iter().sum::<i64>()
            });
            black_box(sum);
        },
        ITERATIONS,
    );

    BenchResult::new(
        "region_vec_1000",
        nexus.as_nanos() as u64,
        baseline.as_nanos() as u64,
        20.0,
    )
}

// =============================================================================
// Main
// =============================================================================

#[cfg(all(feature = "nexus", feature = "std"))]
fn main() {
    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║            OrdoFP Nexus Effect System Benchmarks                          ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Running benchmarks in RELEASE mode...");
    println!();

    let results = vec![
        // State (boxed)
        bench_state_get_put(),
        bench_state_chain(),
        // State (optimized)
        bench_state_op_chain(),
        // Reader (boxed)
        bench_reader_asks(),
        bench_reader_chain(),
        // Reader (optimized)
        bench_reader_op_chain(),
        // Error
        bench_error_ok(),
        bench_error_chain(),
        // Writer (boxed)
        bench_writer_tell(),
        // Writer (optimized)
        bench_writer_op_tell(),
        bench_writer_op_chain(),
        // IO (boxed)
        bench_io_pure(),
        bench_io_chain(),
        // IO (optimized)
        bench_io_op_pure(),
        bench_io_op_chain(),
        // Region
        bench_region_alloc(),
        bench_region_vec(),
    ];

    println!(
        "{:<20} {:>12} {:>12} {:>12} {:>10} {:>8}",
        "Benchmark", "Nexus (ns)", "Base (ns)", "Overhead", "Target", "Status"
    );
    println!("{}", "─".repeat(78));

    let mut passed = 0;
    let mut failed = 0;

    for r in &results {
        let status = if r.passed { "✓ PASS" } else { "✗ FAIL" };
        let overhead_str = if r.overhead_pct < 0.0 {
            format!("{:.1}% faster", -r.overhead_pct)
        } else {
            format!("+{:.1}%", r.overhead_pct)
        };
        let target_str = if r.target_pct < 0.0 {
            format!(">{:.0}% faster", -r.target_pct)
        } else {
            format!("<{:.0}%", r.target_pct)
        };

        println!(
            "{:<20} {:>12} {:>12} {:>12} {:>10} {:>8}",
            r.name, r.nexus_ns, r.baseline_ns, overhead_str, target_str, status
        );

        if r.passed {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    println!("{}", "─".repeat(78));
    println!();
    println!(
        "Summary: {} passed, {} failed out of {} benchmarks",
        passed,
        failed,
        results.len()
    );
    println!();

    // Print acceptance criteria
    println!("Acceptance Criteria (from core/src/nexus/bench.rs):");
    println!("  | Pattern      | Target Overhead |");
    println!("  |--------------|-----------------|");
    println!("  | Pure         | ~0%             |");
    println!("  | State-only   | <5% vs hand     |");
    println!("  | Reader-only  | <5% vs hand     |");
    println!("  | Error-only   | ~0% (Result)    |");
    println!("  | 2 effects    | 5-20%           |");
    println!("  | 3+ effects   | 20-50%          |");
    println!();

    if failed > 0 {
        println!("Note: Some benchmarks exceed targets. This is expected for");
        println!("boxed closure-based implementations. The inline hints help");
        println!("reduce overhead in optimized builds.");
    }
}

#[cfg(not(all(feature = "nexus", feature = "std")))]
fn main() {
    println!("This benchmark requires the 'nexus' and 'std' features.");
    println!(
        "Run with: cargo run --example nexus_benchmark_runner --features \"nexus,std\" --release"
    );
}
