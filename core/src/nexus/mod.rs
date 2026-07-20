//! `OrdoFP` Nexus: Efficient Type-Level Effect System
//!
//! Nexus provides a type-level effect system for Rust with efficient
//! representations for common effect patterns.
//!
//! # Design Philosophy
//!
//! Nexus is a **pure Rust stable library** that uses const generics for
//! type-level effect tracking. It provides:
//!
//! 1. **Efficient Effect Representations** — Single-effect rows use specialized
//!    representations that benchmark within 10% of hand-written equivalents
//! 2. **Typed Optimization Combinators** — Users explicitly opt into optimizations
//!    like parallelization via type-safe combinators
//! 3. **Tiered Verification** — From documented laws to optional property tests
//!
//! # What Nexus IS
//!
//! - A library providing `Eff<R, A>` for effectful computations
//! - Type-level effect tracking via const-generic bitmasks
//! - Efficient handlers for State, Reader, Error, Writer effects
//! - Explicit optimization combinators (`par_map`, memoize, etc.)
//!
//! # What Nexus is NOT
//!
//! - NOT a compiler plugin or language extension
//! - NOT automatic optimization (users must opt-in via combinators)
//! - NOT "zero-cost" in all cases (complex effect combinations use boxing)
//! - NOT formally verified (laws are documented and tested, not proven)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        User Code                                 │
//! │   let result = state_comp.and_then(|x| reader_comp(x));         │
//! └─────────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Type-Level Effect Row                         │
//! │              Eff<Row<STATE_BIT | READER_BIT>, A>                 │
//! └─────────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                 Effect-Specific Representations                  │
//! │   Pure → A | State → fn(S)->(A,S) | Combined → Box<Cont>        │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Performance Characteristics
//!
//! | Effect Pattern | Representation | Expected Overhead |
//! |----------------|----------------|-------------------|
//! | Pure only | Direct value | ~0% |
//! | State only | State-passing function | <5% |
//! | Reader only | Environment-passing function | <5% |
//! | Error only | Result<A, E> | ~0% |
//! | 2 effects | Specialized or boxed | 5-20% |
//! | 3+ effects | Boxed continuation | 20-50% |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::prelude::*;
//!
//! // Pure computation - no overhead
//! let pure_comp: Eff<Pure, i32> = pure(42);
//! assert_eq!(pure_comp.run_pure(), 42);
//!
//! // Stateful computation - uses StatefulComputation for efficiency
//! let state_comp = StatefulComputation::<i32, i32>::get()
//!     .map(|x| x + 1);
//! let (result, final_state) = state_comp.run(0);
//! assert_eq!(result, 1);
//! assert_eq!(final_state, 0);
//! ```
//!
//! # Feature Flags
//!
//! - `nexus` — Core effect infrastructure (required)

// Core effect infrastructure
mod effect;
mod handler;
mod ops;
mod repr;
mod row;

// Effect types
pub mod effects;

// Optimization combinators (explicit opt-in)
pub mod optim;

/// Benchmark infrastructure for validating effect-overhead acceptance criteria.
///
/// Provides workload drivers ([`bench::measure_baseline`], [`bench::measure_state`],
/// etc.) that run effect-wrapped and hand-written code identically for an outer timing
/// harness (no timing happens in this module), plus semantic verification functions
/// ([`bench::verify_pure_semantics`], etc.) that assert correctness. Use
/// [`bench::OverheadReport`] to express and check target overhead budgets.
pub mod bench;

// Handler laws and verification (Tier 0-1)
pub mod laws;

// Runtime verification infrastructure (Tier 2)
pub mod verification;

// Prelude for convenient imports
pub mod prelude;

// Re-exports
pub use effect::{Eff, EffectMarker};
pub use effect::{Error, ErrorEff, err, ok};
pub use effect::{IO, IoEff};
pub use effect::{Reader, ReaderEff, ask, asks};
pub use effect::{State, StateEff, get, modify, put};
pub use handler::{ComposedHandler, Handler, InlineHandler, handle};
pub use ops::{perform, pure};
pub use repr::{ContRepr, EffRepr, PureRepr, ReaderRepr, StateRepr};
pub use row::{EffectRow, Pure};
pub use row::{row_diff_bits, row_intersect_bits, row_union_bits};
