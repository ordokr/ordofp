//! Getting Started with `OrdoFP`
//!
//! This example provides a comprehensive introduction to `OrdoFP`'s features.
//!
//! # Running
//!
//! ```bash
//! cargo run --example 00_getting_started
//! ```

use ordofp_core::prelude::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           Welcome to OrdoFP - Functional Programming         ║");
    println!("║              for Rust with Effect Tracking                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // =========================================================================
    // PART 1: HList - Heterogeneous Lists
    // =========================================================================

    println!("━━━ Part 1: HList (Heterogeneous Lists) ━━━");
    println!();

    // HLists are type-safe lists that can hold different types
    let my_list = hlist![42, "hello", true, 3.1];
    println!("   Created HList: (42, \"hello\", true, 3.1)");
    println!("   First element: {}", my_list.head);
    println!("   Second element: {}", my_list.tail.head);
    println!();

    // Type aliases make complex types readable
    type PersonData = HList![String, u32, bool];
    let person: PersonData = hlist!["Alice".to_string(), 30, true];
    println!(
        "   Person data: ({}, {}, {})",
        person.head, person.tail.head, person.tail.tail.head
    );
    println!();

    // =========================================================================
    // PART 2: Optics - Lens, Prism, Iso
    // =========================================================================

    println!("━━━ Part 2: Optics (Lens, Prism, Iso) ━━━");
    println!();

    // Lens: Focus on a field in a struct
    #[derive(Clone, Debug)]
    struct User {
        name: String,
        age: u32,
    }

    let name_lens = lens(
        |u: &User| u.name.clone(),
        |u: &User, name: String| User { name, age: u.age },
    );

    let user = User {
        name: "Bob".to_string(),
        age: 25,
    };

    println!("   Original user: {user:?}");
    println!("   Name via lens: {}", name_lens.get(&user));

    let updated = name_lens.set(&user, "Robert".to_string());
    println!("   After rename: {updated:?}");
    println!();

    // Prism: Focus on a variant in an enum
    #[derive(Clone, Debug, PartialEq)]
    enum Result2<T, E> {
        Ok(T),
        Err(E),
    }

    let ok_prism = prism(
        |r: &Result2<i32, &str>| match r {
            Result2::Ok(v) => Some(*v),
            Result2::Err(_) => None,
        },
        Result2::Ok,
    );

    let success = Result2::Ok(42);
    let _failure: Result2<i32, &str> = Result2::Err("error"); // Demonstrate Err variant
    println!("   Prism preview Ok(42): {:?}", ok_prism.preview(&success));
    println!("   Prism review 100: {:?}", ok_prism.review(100));
    println!();

    // =========================================================================
    // PART 3: Either/Aut - Sum Types
    // =========================================================================

    println!("━━━ Part 3: Either/Aut (Sum Types) ━━━");
    println!();

    // Either (called Aut in Latin) represents "this OR that"
    let success: Aut<&str, i32> = Aut::Dexter(42); // Right value
    let failure: Aut<&str, i32> = Aut::Sinister("error"); // Left value

    println!("   Success value: {success:?}");
    println!("   Failure value: {failure:?}");
    println!("   Is success right? {}", success.is_dexter());
    println!("   Is failure left? {}", failure.is_sinister());
    println!();

    // =========================================================================
    // PART 4: Easy API - Simplified Effect Handling
    // =========================================================================

    println!("━━━ Part 4: Easy API (Simplified Effects) ━━━");
    println!();

    // State management
    let counter_result = run_with_state(0, |counter| {
        *counter += 1;
        *counter += 2;
        *counter *= 3;
        *counter
    });
    println!("   State computation: 0 -> +1 -> +2 -> *3 = {counter_result}");

    // Reader/Config pattern
    #[derive(Clone)]
    struct Config {
        multiplier: i32,
        offset: i32,
    }

    let config = Config {
        multiplier: 2,
        offset: 10,
    };

    let computed = run_with_config(&config, |cfg| {
        let base = 5;
        base * cfg.multiplier + cfg.offset
    });
    println!("   Config computation: 5 * 2 + 10 = {computed}");

    // Error handling with retry
    let mut attempts = 0;
    let result: Result<i32, &str> = retry(3, || {
        attempts += 1;
        if attempts >= 2 {
            Ok(42)
        } else {
            Err("not ready")
        }
    });
    println!("   Retry result (succeeded on attempt {attempts}): {result:?}");

    // Fallback pattern
    let fallback_result: Result<i32, &str> = fallback(|| Err("primary failed"), || Ok(100));
    println!("   Fallback result: {fallback_result:?}");
    println!();

    // =========================================================================
    // PART 5: IO Computations
    // =========================================================================

    println!("━━━ Part 5: IO Computations ━━━");
    println!();

    // IO represents lazy computations
    let lazy_computation = io(|| {
        println!("   (IO computation executed!)");
        42
    });
    println!("   Created lazy IO (not yet executed)");
    let io_result = lazy_computation.run();
    println!("   IO result: {io_result}");

    // IO chaining
    let chained = io(|| 10).map(|x| x * 2).and_then(|x| io(move || x + 22));

    println!("   Chained IO (10 -> *2 -> +22): {}", chained.run());
    println!();

    // =========================================================================
    // PART 6: Arena Allocation
    // =========================================================================

    println!("━━━ Part 6: Arena Allocation ━━━");
    println!();

    let arena_result = with_arena(|arena| {
        // Fast, scoped allocations
        let x = arena.alloc(100);
        let y = arena.alloc(200);
        let slice = arena.alloc_slice(&[1, 2, 3, 4, 5]);

        let sum = *x + *y + slice.iter().sum::<i32>();
        println!("   Arena allocated: x=100, y=200, slice=[1,2,3,4,5]");
        sum
    });
    println!("   Arena computation result: {arena_result}");
    println!("   (All arena allocations freed automatically)");
    println!();

    // =========================================================================
    // PART 7: Combinators
    // =========================================================================

    println!("━━━ Part 7: Combinators ━━━");
    println!();

    // Chain multiple computations
    let chain_result = chain(|| 1, |x| x + 10, |x| x * 2);
    println!("   chain(1, +10, *2) = {chain_result}");

    // Run two computations in parallel (conceptually)
    let (a, b) = both(|| 21, || 21);
    println!("   both(21, 21) = ({}, {}), sum = {}", a, b, a + b);

    // Conditional computation
    let value = 42;
    let conditional = when(value > 0, || "positive", || "non-positive");
    println!("   when(42 > 0) = \"{conditional}\"");

    // Repeat a computation
    let squares: Vec<i32> = repeat(5, |i| (i * i) as i32);
    println!("   repeat(5, i^2) = {squares:?}");
    println!();

    // =========================================================================
    // SUMMARY
    // =========================================================================

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                     OrdoFP Features Summary                   ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  HList      : Type-safe heterogeneous lists                  ║");
    println!("║  Optics     : Lens, Prism, Iso for data access               ║");
    println!("║  Either/Aut : Sum types (Left/Right, Sinister/Dexter)        ║");
    println!("║  Easy API   : State, Reader, Error handling                  ║");
    println!("║  IO         : Lazy, composable computations                  ║");
    println!("║  Arena      : Fast, scoped memory allocation                 ║");
    println!("║  Combinators: chain, both, when, repeat, etc.                ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  For more examples, see:                                     ║");
    println!("║    cargo run --example 09_easy_api                           ║");
    println!("║    cargo run --example 10_vernacular_api                     ║");
    println!("║    cargo run --example 11_effects_intro --features async     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}
