//! Async Transformer Compatibility Example
//!
//! This example demonstrates that `OrdoFP`'s monad transformers are compatible
//! with async runtimes like tokio and smol because they enforce
//! `Send + Sync` bounds on closure arguments.
//!
//! Note: This example uses simulated async to work without tokio dependency.
//! In a real application, replace the simulated functions with actual async calls.
//!
//! # Key Points
//!
//! - `ReaderT` and `StateT` require `F: Send + Sync + 'static` on closures
//! - `OptionT` and `EitherT` auto-derive Send/Sync from inner types
//! - No blocking operations in transformer implementations
//!
//! # Usage with Tokio
//!
//! ```ignore
//! use tokio::spawn;
//!
//! // ReaderT closures are Send + Sync, so this works:
//! let reader: ReaderT<Config, Option<i32>> = ReaderT::new(|cfg: &Config| {
//!     Some(cfg.value * 2)
//! });
//!
//! let handle = spawn(async move {
//!     reader.run(&Config { value: 21 })
//! });
//! ```

#[cfg(feature = "alloc")]
use ordofp_core::transformers::{EitherT, OptionT, ReaderT, StateT};

/// Configuration for async computations
#[derive(Clone, Debug)]
struct AppConfig {
    database_url: String,
    max_retries: u32,
    timeout_ms: u64,
}

/// Demonstrates that `ReaderT` closures can be passed across thread boundaries
#[cfg(feature = "alloc")]
fn reader_t_send_sync_example() {
    println!("=== ReaderT Send/Sync Example ===");

    let config = AppConfig {
        database_url: "postgres://localhost:5432".to_string(),
        max_retries: 3,
        timeout_ms: 1000,
    };
    println!("Using config: {config:?}");

    // ReaderT requires Send + Sync + 'static on closures
    let fetch_config: ReaderT<AppConfig, Option<String>> = ReaderT::new(|cfg: &AppConfig| {
        if cfg.max_retries > 0 && cfg.timeout_ms > 0 {
            Some(cfg.database_url.clone())
        } else {
            None
        }
    });

    // This works because ReaderT<_, _> is Send when the closure is Send
    let config = AppConfig {
        database_url: "postgres://localhost/db".to_string(),
        max_retries: 3,
        timeout_ms: 5000,
    };

    // Simulate spawning to another thread (would use tokio::spawn in real code)
    let result = std::thread::spawn(move || fetch_config.run(&config))
        .join()
        .unwrap();

    println!("  Database URL: {result:?}");
}

/// Demonstrates `StateT` with thread-safe state management
#[cfg(feature = "alloc")]
fn state_t_send_sync_example() {
    println!("\n=== StateT Send/Sync Example ===");

    // StateT with a counter state
    #[derive(Clone)]
    struct Counter {
        value: i32,
        operations: Vec<String>,
    }

    // Increment counter - closure is Send + Sync
    let increment: StateT<Counter, Option<(Counter, i32)>> = StateT::new(|mut c: Counter| {
        let old = c.value;
        c.value += 1;
        c.operations.push("increment".to_string());
        Some((c, old))
    });

    let initial = Counter {
        value: 0,
        operations: vec![],
    };

    // Can be sent to another thread
    let result = std::thread::spawn(move || increment.run(initial))
        .join()
        .unwrap();

    if let Some((final_state, old_value)) = result {
        println!("  Old value: {old_value}");
        println!("  New value: {}", final_state.value);
        println!("  Operations: {:?}", final_state.operations);
    }
}

/// Demonstrates `OptionT` auto-deriving Send from inner type
#[cfg(feature = "alloc")]
fn option_t_send_example() {
    println!("\n=== OptionT Send Example ===");

    // OptionT<Result<Option<T>, E>> is Send if T and E are Send
    let computation: OptionT<Result<Option<i32>, String>> = OptionT::some(42);
    let mapped = computation.map(|x| x * 2);

    // Can be sent to another thread
    let result = std::thread::spawn(move || mapped.run()).join().unwrap();

    println!("  Result: {result:?}");
}

/// Demonstrates `EitherT` auto-deriving Send from inner type
#[cfg(feature = "alloc")]
fn either_t_send_example() {
    println!("\n=== EitherT Send Example ===");

    // EitherT<Option<Result<T, E>>> is Send if T and E are Send
    let computation: EitherT<Option<Result<i32, String>>> = EitherT::right(21);
    let doubled = computation.map(|x| x * 2);

    // Can be sent to another thread
    let result = std::thread::spawn(move || doubled.run()).join().unwrap();

    println!("  Result: {result:?}");
}

/// Demonstrates combining transformers in an async-compatible way
#[cfg(feature = "alloc")]
fn combined_transformers_example() {
    println!("\n=== Combined Transformers Example ===");

    #[derive(Clone)]
    struct Config {
        multiplier: i32,
    }

    // Chain of ReaderT computations - all closures are Send + Sync
    let step1: ReaderT<Config, Option<i32>> = ReaderT::new(|cfg: &Config| Some(cfg.multiplier));

    let step2 = step1.flat_map(|val| {
        ReaderT::new(move |cfg: &Config| {
            if cfg.multiplier > 0 {
                Some(val * 2)
            } else {
                None
            }
        })
    });

    let step3 = step2.map(|val| val + 10);

    let config = Config { multiplier: 5 };

    // Entire chain can be sent to another thread
    let result = std::thread::spawn(move || step3.run(&config))
        .join()
        .unwrap();

    println!("  Result: {result:?}"); // Some(20): 5 * 2 + 10
}

/// Pattern that would NOT be Send/Sync safe:
///
/// ```ignore
/// // The following would NOT compile because Rc is not Send:
/// use std::rc::Rc;
/// let non_send_data = Rc::new(42);
///
/// // This fails: Rc<i32> is not Send
/// let reader = ReaderT::<(), Option<Rc<i32>>>::new(move |_| {
///     Some(non_send_data.clone())
/// });
///
/// std::thread::spawn(move || reader.run(&())); // ERROR: Rc is not Send
/// ```
///
/// To make it work, use `Arc` instead of `Rc`, and `Mutex`/`RwLock` instead
/// of `Cell`/`RefCell` if interior mutability is needed.
fn main() {
    #[cfg(feature = "alloc")]
    {
        println!("OrdoFP Async Transformer Compatibility Demo");
        println!("============================================\n");
        println!("This demonstrates that transformers work with async runtimes.");
        println!("All closures passed to ReaderT/StateT must be Send + Sync.\n");

        reader_t_send_sync_example();
        state_t_send_sync_example();
        option_t_send_example();
        either_t_send_example();
        combined_transformers_example();

        println!("\n============================================");
        println!("All transformers are compatible with tokio/smol!");
    }

    #[cfg(not(feature = "alloc"))]
    {
        println!("This example requires the 'alloc' feature.");
        println!("Run with: cargo run --example 07_async_transformers --features alloc");
    }
}
