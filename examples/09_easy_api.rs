//! Easy API Examples
//!
//! Demonstrates the simplified "Easy" API that hides effect system complexity.
//!
//! # Running
//!
//! ```bash
//! cargo run --example 09_easy_api
//! ```

use ordofp_core::easy::{
    ResultExt, ask, bool_to_result, both, chain, fallback, get, io, io_sequence, modify,
    partition_results_vec, repeat, retry, run_with_config, run_with_state, state_pure,
    validate_all, when,
};

fn main() {
    println!("=== OrdoFP Easy API Examples ===\n");

    // Example 1: Simple state management
    println!("1. Simple State Management");
    println!("--------------------------");

    let result = run_with_state(0, |counter| {
        *counter += 1;
        *counter += 1;
        *counter *= 2;
        *counter
    });
    println!("   Counter result: {result}");

    // State monad style
    let computation = get::<i32>()
        .and_then(|x| modify(|s: i32| s + 10).then(state_pure(x)))
        .map(|x| x * 2);

    let (result, final_state) = computation.run(5);
    println!("   State monad: result={result}, final_state={final_state}");

    // Example 2: Reader for configuration
    println!("\n2. Reader/Configuration Pattern");
    println!("--------------------------------");

    #[derive(Clone)]
    struct Config {
        timeout_ms: u64,
        max_retries: u32,
    }

    let config = Config {
        timeout_ms: 5000,
        max_retries: 3,
    };

    let result = run_with_config(&config, |cfg| cfg.timeout_ms * u64::from(cfg.max_retries));
    println!("   Computed value: {result}");

    // Reader monad style
    let reader = ask::<Config>().map(|cfg| cfg.timeout_ms);
    let timeout = reader.run(&config);
    println!("   Timeout from reader: {timeout}");

    // Example 3: Error handling
    println!("\n3. Error Handling");
    println!("-----------------");

    // Retry pattern
    let mut attempts = 0;
    let result: Result<i32, &str> = retry(3, || {
        attempts += 1;
        if attempts < 3 {
            Err("not ready yet")
        } else {
            Ok(42)
        }
    });
    println!("   Retry result: {result:?} (took {attempts} attempts)");

    // Fallback pattern
    let result: Result<i32, &str> = fallback(|| Err("primary failed"), || Ok(100));
    println!("   Fallback result: {result:?}");

    // Validation with error accumulation
    let validate_age = |age: i32| {
        validate_all(
            age,
            &[
                (|x| *x >= 0, "must be non-negative"),
                (|x| *x <= 150, "must be <= 150"),
            ],
        )
    };

    println!("   Validate 25: {:?}", validate_age(25));
    println!("   Validate -5: {:?}", validate_age(-5));

    // Example 4: IO computations
    println!("\n4. IO Computations");
    println!("------------------");

    // Lazy IO
    let lazy_computation = io(|| {
        println!("   (IO executed!)");
        42
    });
    println!("   Created lazy IO (not yet executed)");
    let result = lazy_computation.run();
    println!("   IO result: {result}");

    // Chaining IO
    let chained = io(|| 10).map(|x| x * 2).and_then(|x| io(move || x + 5));
    println!("   Chained IO result: {}", chained.run());

    // Sequence multiple IOs
    let ios = vec![io(|| 1), io(|| 2), io(|| 3)];
    let results = io_sequence(ios).run();
    println!("   Sequenced IO results: {results:?}");

    // Example 5: Combinators
    println!("\n5. Computation Combinators");
    println!("--------------------------");

    // Chain computations
    let result = chain(|| 1, |x| x + 10, |x| x * 2);
    println!("   Chain (1 -> +10 -> *2): {result}");

    // Tuple combinators
    let (a, b) = both(|| "hello", || 42);
    println!("   Both: ({a}, {b})");

    // Conditional
    let x = 10;
    let result = when(x > 5, || "big", || "small");
    println!("   Conditional (x=10): {result}");

    // Repeat
    let squares: Vec<i32> = repeat(5, |i| (i * i) as i32);
    println!("   Squares 0-4: {squares:?}");

    // Example 6: Result extensions
    println!("\n6. Result Extensions");
    println!("--------------------");

    let result: Result<i32, &str> = Ok(42);

    // Tap into values
    let tapped = result.tap(|x| println!("   Tapped value: {x}"));
    println!("   Original preserved: {tapped:?}");

    // Check predicates
    let is_positive = result.is_ok_and(|x| x > 0);
    println!("   Is positive? {is_positive}");

    // Swap Ok/Err
    let swapped: Result<&str, i32> = result.swap();
    println!("   Swapped: {swapped:?}");

    // Bool to Result
    let probatum = bool_to_result(true, "success", "failure");
    println!("   Bool to Result: {probatum:?}");

    // Partition results
    let results: Vec<Result<i32, &str>> = vec![Ok(1), Err("a"), Ok(2), Err("b")];
    let (successes, errors) = partition_results_vec(results);
    println!("   Partitioned: successes={successes:?}, errors={errors:?}");

    println!("\n=== Examples Complete ===");
}
