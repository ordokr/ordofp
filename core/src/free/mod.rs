//! Free Monads and Tagless Final - DSL Interpreters
//!
//! > *"Libertas est potestas faciendi id quod iure licet."*
//! > — Freedom is the power to do what is permitted by law.
//!
//! This module provides Free monads and Tagless Final patterns for
//! building and interpreting domain-specific languages (DSLs).
//!
//! # Core Concepts
//!
//! ## Free Monad
//!
//! The Free monad `Liber<F, A>` builds a monad from any functor `F`.
//! It represents a computation that can be interpreted in multiple ways.
//!
//! ## Freer Monad
//!
//! The Freer monad `Liberior<F, A>` is more efficient - it doesn't
//! require `F` to be a Functor, using defunctionalization instead.
//!
//! ## Tagless Final
//!
//! Tagless Final represents DSLs as traits (algebras) rather than
//! data structures. This enables extensibility and multiple interpreters.
//!
//! # Scholastic Naming
//!
//! | Concept | Latin Name | Etymology |
//! |---------|------------|-----------|
//! | Free Monad | Liber | *liber* = free |
//! | Freer Monad | Liberior | *liberior* = more free |
//! | Natural Transformation | `TransformatioNaturalis` | *transformatio* = change of form |
//! | Algebra | Algebra | Arabic *al-jabr* = reunion of broken parts |
//! | Interpreter | Interpres | *interpres* = explainer, translator |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::free::*;
//!
//! // Build a program using the Free monad over the `Option` functor
//! let program: Liber<OptionFWitness, i32> = Liber::purus(42);
//!
//! // Interpret it via the identity natural transformation back to `Option`
//! let result = plica_liber::<OptionFWitness, OptionFWitness, i32, _>(|fa| fa, program);
//! assert_eq!(result, Some(42));
//! ```

#[cfg(feature = "alloc")]
extern crate alloc;

mod liber;
mod liber_ecclesia;
mod liberior;
mod nat;
mod tagless;

pub use liber::*;
pub use liber_ecclesia::*;
pub use liberior::*;
pub use nat::*;
pub use tagless::*;
