//! Effects System Introduction
//!
//! Demonstrates the core concepts of `OrdoFP`'s effect system using
//! both the vernacular (English) and scholastic (Latin) APIs.
//!
//! # Concepts
//!
//! - **Effects**: Side effects tracked in the type system
//! - **Effect Rows**: Collections of effects a computation may perform
//! - **Handlers**: Interpreters that give meaning to effects
//!
//! # Running
//!
//! ```bash
//! cargo run --example 11_effects_intro --features async
//! ```

// This example demonstrates the concepts of the effect system without
// requiring full effect infrastructure. See the documentation below.

#[cfg(feature = "async")]
fn main() {
    println!("=== OrdoFP Effects System Introduction ===\n");

    // Note: Full async examples require tokio runtime
    // This example demonstrates the concepts

    println!("Effect System Concepts:");
    println!("-----------------------\n");

    println!("1. EFFECT TYPES");
    println!("   Effects are tracked in the type system.");
    println!("   ");
    println!("   Common effects:");
    println!("   - State<S>  : Mutable state of type S");
    println!("   - Error<E>  : May fail with error of type E");
    println!("   - Reader<R> : Read-only environment of type R");
    println!("   - Writer<W> : Append-only log of type W");
    println!("   - IO        : Input/output operations");
    println!("   - Async     : Asynchronous operations");

    println!("\n2. EFFECT ROWS");
    println!("   Effects are combined into rows:");
    println!("   ");
    println!("   Type notation:");
    println!("   - Eff<R, A>                  : Computation returning A with effects R");
    println!("   - RowVacuus                  : Empty (pure) effect row");
    println!("   - RowExtensio<E, R>          : Add effect E to row R");
    println!("   ");
    println!("   Example:");
    println!("   - Eff<RowVacuus, i32>                    : Pure computation");
    println!("   - Eff<RowExtensio<State<u32>, RowVacuus>, i32>");
    println!("                                            : Stateful computation");

    println!("\n3. HANDLERS");
    println!("   Handlers give meaning to effects:");
    println!("   ");
    println!("   Example (conceptual):");
    println!("   ```");
    println!("   // Define computation with State effect");
    println!("   async fn counter<R>() -> Eff<R, u32>");
    println!("   where R: HasEffectus<StatusEffectus<u32>>");
    println!("   {{");
    println!("       let n = send::<StatusEffectus<u32>, _, _>(StatusOp::Get).await;");
    println!("       send::<StatusEffectus<u32>, _, _>(StatusOp::Put(n + 1)).await;");
    println!("       pure_eff(n + 1)");
    println!("   }}");
    println!("   ");
    println!("   // Handle State effect with initial value");
    println!("   let result = run_state(0, counter).await;");
    println!("   ```");

    println!("\n4. EFFECT COMPOSITION");
    println!("   Multiple effects compose naturally:");
    println!("   ");
    println!("   - Row extension: RowExtensio<E1, RowExtensio<E2, RowVacuus>>");
    println!("   - Vernacular: E1 | E2");
    println!("   - Effect inference: Compiler tracks which effects are used");
    println!("   ");
    println!("   Handler order matters:");
    println!("   - Handle inner effects first");
    println!("   - Outer handlers can access inner computation results");

    println!("\n5. NAMING CONVENTIONS");
    println!("   ");
    println!("   | Scholastic (Latin)     | Vernacular (English)  |");
    println!("   |------------------------|-----------------------|");
    println!("   | Effectus               | Effect                |");
    println!("   | Computatio             | Computation           |");
    println!("   | StatusEffectus         | StateEffect           |");
    println!("   | ErrorEffectus          | ErrorEffect           |");
    println!("   | TractatorAlgebraicus   | AlgebraicHandler      |");
    println!("   | RowVacuus              | EmptyRow              |");
    println!("   | RowExtensio            | ExtendRow             |");
    println!("   | HasEffectus            | HasEffect             |");

    println!("\n6. EXAMPLE: USING THE EASY API");
    println!("   For simpler use cases, use the Easy API:");
    println!("   ");
    println!("   ```");
    println!("   use ordofp::easy::*;");
    println!("   ");
    println!("   // State without effect machinery");
    println!("   let result = run_with_state(0, |count| {{");
    println!("       *count += 1;");
    println!("       *count");
    println!("   }});");
    println!("   ```");

    println!("\n=== Effects Introduction Complete ===");
    println!("\nFor more examples, see:");
    println!("- examples/09_easy_api.rs      : Simplified API");
    println!("- examples/10_vernacular_api.rs : English naming");
    println!("- examples/07_async_transformers.rs : Async effects");
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature.");
    println!("Run with: cargo run --example 11_effects_intro --features async");
}
