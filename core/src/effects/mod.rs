//! Effect System Foundation - Lightweight algebraic effect infrastructure
//!
//! > *"Effectus est actus causae"*
//! > — An effect is the act of a cause. (Scholastic philosophy)
//!
//! This module provides a lightweight effect system based on the `ReaderT` design
//! pattern, enabling composition of effectful computations without full algebraic
//! effects (which require unstable coroutines).
//!
//! # Design Philosophy
//!
//! Rather than implementing full algebraic effects, we use a practical approach
//! inspired by functional programming best practices:
//!
//! > "I'm overall not a huge fan of monad transformers. I think they are drastically
//! > overused in Haskell... I instead advocate the ReaderT design pattern."
//! > — FP Block Academy
//!
//! # Overview
//!
//! The effect system provides:
//!
//! - [`Effectus`] - Marker trait for effect types
//! - [`EffectusHandler`] - Synchronous effect handler capability
//! - [`EffectusHandlerAsync`] - Asynchronous effect handler capability
//! - Common effect markers: [`IoEffectus`], [`StatusEffectus`], [`ErrorEffectus`], [`AsyncEffectus`]
//!
//! ## `OrdoFP` 3.0 Row-Polymorphic Effects
//!
//! The effect system now includes row-polymorphic effect tracking inspired by
//! Koka and PureScript:
//!
//! - [`row_v2`] - Const-generic bitset encoding of effect rows
//! - [`computatio`] - Row-polymorphic effectful computations
//! - `continuation` - One-shot continuations for effect handlers
//! - [`inference`] - Effect type inference utilities
//!
//! ## `OrdoFP` 3.0 Algebraic Effect Handlers
//!
//! Full algebraic effect support inspired by OCaml 5.0 and Koka:
//!
//! - [`algebraic`] - Core algebraic effect traits and handler infrastructure
//! - [`builtin`] - Built-in effects: State, Reader, Writer, Error, Choice, Console
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Effect | Effectus | *effectus* = result, accomplishment |
//! | Handler | Tractator | *tractator* = one who handles |
//! | IO | Actio | *actio ad extra* = action towards outside |
//! | State | Status | *status* = condition, state |
//! | Error | Error | *error* = wandering, mistake |
//! | Row | Ordo | *ordo* = row, order |
//! | Continuation | Continuatio | *continuatio* = uninterrupted succession |
//! | Computation | Computatio | *computatio* = calculation |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::effects::{EffectusHandlerAsync, IoEffectus};
//!
//! // Define an IO handler
//! struct ConsoleHandler;
//!
//! impl EffectusHandlerAsync<IoEffectus> for ConsoleHandler {
//!     type Output = ();
//!
//!     async fn handle_async(&self, _effect: IoEffectus) -> Self::Output {
//!         // Handle IO effect
//!     }
//! }
//! ```
//!
//! # Row-Polymorphic Effects Example
//!
//! ```rust
//! use ordofp_core::effects::row_v2::{EffectRow, EffectSet, IoRow, assert_has_effect_type};
//! use ordofp_core::effects::computatio::Computatio;
//! use ordofp_core::effects::IoEffectus;
//!
//! // Define computations with tracked effects
//! fn pure_computation() -> Computatio<EffectSet<0>, i32> {
//!     Computatio::purus(42)
//! }
//!
//! // Functions can require specific effects (checked at monomorphization)
//! fn requires_io<R: EffectRow>() {
//!     assert_has_effect_type::<R, IoEffectus>();
//!     // Guaranteed to have IO capability
//! }
//!
//! requires_io::<IoRow>();
//! let _ = pure_computation();
//! ```
//!
//! # Feature Flags
//!
//! This module requires the `async` feature:
//!
//! ```toml
//! [dependencies]
//! ordofp = { version = "2.0", features = ["async"] }
//! ```

mod common;
mod effect;
mod handler;
mod lift;

// OrdoFP 3.0: Row-Polymorphic Effects
pub mod computatio;
pub mod continuation_v2;
pub mod inference;
pub mod row_v2;

// OrdoFP 3.0: Algebraic Effect Handlers
pub mod algebraic;
pub mod builtin;
pub mod handler_multi;

// OrdoFP 4.0 Phase 2: Advanced Effect System
pub mod eff;
pub mod sem;
pub mod testimonium;

// Async integration bridge
pub mod async_bridge;

pub use common::{
    AsyncEffectus, ErrorEffectus, IoEffectus, PurusEffectus, RandomEffectus, ReaderEffectus,
    ResourceEffectus, ScriptorEffectus, StatusEffectus, TempusEffectus,
};
pub use effect::{
    CombinedEffectus, Effectus, EffectusCombine, EffectusProduces, EffectusWithValue,
};
pub use handler::{
    ComposedHandler, EffectusHandler, EffectusHandlerAsync, EffectusHandlerExt, PanicHandler,
    UnitHandler, run_effectus, run_effectus_async,
};
pub use lift::{
    AsyncLift, ErrorLift, IoLift, LiftEffectus, PureLift, ReaderLift, StateLift, lift_async,
    lift_error, lift_io, lift_pure, lift_reader, lift_state,
};

// Re-export commonly used row types (v2 bitset encoding).
pub use row_v2::{
    AsyncEffectV2, EffectId, EffectRow, EffectSet, EffectSetVacuus, ErrorEffectV2, ErrorRow,
    IoEffectV2, IoRow, Pure, ReaderEffectV2, ReaderRow, StateEffectV2, StateRow, WriterEffectV2,
    assert_disjoint, assert_has_effect, assert_has_effect_type, assert_subrow,
    assert_without_effect, builtin_ids,
};

// Re-export computation types
pub use computatio::{Computatio, combine_effectus, perform, sequence, traverse};

// Re-export inference types
pub use inference::{
    AnyEffects, BuilderRow, EffectTuple, EffectVar, EffectusBuilder, EffectusWitness, HasRow,
    InferEffectus, IsPure, Merged, RowEquivalent, assert_requires_effects,
};

// Re-export algebraic effect types
pub use algebraic::{
    ClosureHandler, ComputatioStatus, DefaultHandler, EffectusAlgebraicus, HandledBy,
    HandlerConfig, IdentityHandler, Operatio, TractatorAlgebraicus, default_handler, make_handler,
    pure_effect, run_with_handler,
};

// Re-export built-in effects
pub use builtin::{
    AsyncOp, ConsolaOp, ElectioHandler, ElectioOp, ErrorHandler, ErrorOp, LectorHandler, LectorOp,
    MockConsolaHandler, ScriptorHandler, ScriptorOp, StatusHandler, StatusOp,
};

// Re-export OrdoFP 4.0 Phase 2 types
pub use eff::{
    Eff, EffResult, EffSuspension, Membrum, pure_eff, run_purus, send, sequence_eff, then,
    traverse_eff,
};

pub use testimonium::{
    Clausula, ClausulaGenus, Resumptio, SignumEffectus, Testimonium, TractatorEvidentia,
    VectorTestimonium, run_with_evidence,
};

pub use sem::{
    Sem, SemResult, SemSuspension, embed, pure_sem, raise, run_sem, send_sem, subsume, then_sem,
};

// Re-export async bridge utilities
pub use async_bridge::{
    EffAsync, eff_to_async, eff_to_lector, lector_to_eff_async, lift_async as lift_async_to_eff,
    lift_eff, sequence_eff_async, traverse_eff_async,
};

// Re-export continuation types (from continuation_v2)
pub use continuation_v2::{
    Affinis, Continuatio, ContinuatioAffinis, ContinuatioPluries, ContinuatioSemel, Pluries, Semel,
    TractatorContinuatio, TractatorResult,
};

// Re-export multi-shot handler types
pub use handler_multi::{TractatorMulti, TractatorMultiExt};
