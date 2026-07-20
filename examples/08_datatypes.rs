//! # Core Datatypes Example
//!
//! Demonstrates `OrdoFP`'s fundamental datatypes:
//! - `Identitas` - The identity functor
//! - `Aut` - The disjunction type (Either)
//! - `Pigritia` - Lazy evaluation with memoization
//! - `Const` - The constant functor

use ordofp_core::datatypes::{Aut, Identitas, Pigritia};

fn main() {
    // =========================================================================
    // Identitas - The Identity Functor
    // =========================================================================
    println!("=== Identitas (Identity) ===");

    let id = Identitas::new(42);
    println!("Created: {id:?}");

    // Map preserves structure
    let doubled = id.map(|x| x * 2);
    println!("Doubled: {doubled:?}");

    // Extract the value
    println!("Unwrapped: {}", doubled.unwrap());

    // =========================================================================
    // Aut - Either Left or Right (Disjunction)
    // =========================================================================
    println!("\n=== Aut (Either) ===");

    // Dexter (Right) - typically the "success" case
    let success: Aut<String, i32> = Aut::dexter(42);
    println!("Success: {success:?}");

    // Sinister (Left) - typically the "error" case
    let failure: Aut<String, i32> = Aut::sinister("Error occurred".to_string());
    println!("Failure: {failure:?}");

    // Map only affects Dexter
    let mapped = success.map(|x| x * 2);
    println!("Mapped success: {mapped:?}");

    // Fold to handle both cases
    let result = failure.fold(
        |err| format!("Failed: {err}"),
        |val| format!("Got value: {val}"),
    );
    println!("Folded: {result}");

    // Convert to/from Result
    let as_result: Result<i32, String> = Aut::dexter(100).into_result();
    println!("As Result: {as_result:?}");

    // =========================================================================
    // Pigritia - Lazy Evaluation with Memoization
    // =========================================================================
    println!("\n=== Pigritia (Lazy) ===");

    // Create a lazy computation
    let lazy = Pigritia::new(|| {
        println!("  [Computing expensive value...]");
        (1..=10).sum::<i32>()
    });

    println!("Lazy value created (not yet evaluated)");
    println!("Is evaluated? {}", lazy.is_evaluated());

    // Force evaluation
    println!("Forcing evaluation...");
    let value = lazy.force();
    println!("Got value: {value}");
    println!("Is evaluated now? {}", lazy.is_evaluated());

    // Second force returns cached value (no recomputation)
    println!("Forcing again...");
    let same_value = lazy.force();
    println!("Same value: {same_value}");

    // Map creates a new lazy computation
    let lazy2 = Pigritia::new(|| 21);
    let doubled_lazy = lazy2.map(|x| x * 2);
    println!("Mapped lazy (forcing): {}", doubled_lazy.force());

    // =========================================================================
    // Combining Datatypes
    // =========================================================================
    println!("\n=== Combining Datatypes ===");

    // Parse with error handling and lazy evaluation
    fn parse_lazy(input: &str) -> Aut<String, Pigritia<i32, impl FnOnce() -> i32>> {
        match input.parse::<i32>() {
            Ok(n) => {
                // Wrap successful parse in lazy computation for deferred processing
                Aut::dexter(Pigritia::new(move || {
                    println!("  [Processing parsed value...]");
                    n * n
                }))
            }
            Err(e) => Aut::sinister(format!("Parse error: {e}")),
        }
    }

    let parsed = parse_lazy("7");
    match parsed {
        Aut::Dexter(lazy) => {
            println!("Parsed successfully, result = {}", lazy.force());
        }
        Aut::Sinister(err) => {
            println!("Error: {err}");
        }
    }

    let failed = parse_lazy("not a number");
    match failed {
        Aut::Dexter(lazy) => {
            println!("Parsed successfully, result = {}", lazy.force());
        }
        Aut::Sinister(err) => {
            println!("Error: {err}");
        }
    }

    println!("\n=== Done ===");
}
