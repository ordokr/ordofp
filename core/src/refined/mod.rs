//! Refinement Types
//!
//! > *"Veritas in definitione est"*
//! > — Truth is in the definition. (Latin)
//!
//! This module provides refinement types for expressing type-level constraints
//! on values. A refinement type `Refined<T, P>` wraps a value of type `T` that
//! is guaranteed to satisfy predicate `P`.
//!
//! # Overview
//!
//! Refinement types allow expressing invariants that values must satisfy:
//!
//! - `Refined<i32, Positive>` - A positive integer
//! - `Refined<String, NonEmpty>` - A non-empty string
//! - `Refined<i64, InRange<0, 100>>` - An integer in range [0, 100]
//!
//! # Design Philosophy
//!
//! The refinement type system follows the principle of "making illegal states
//! unrepresentable" by encoding constraints at the type level. Values are
//! validated at construction time, and the refined wrapper provides compile-time
//! evidence that the constraint holds.
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Refined | Refinatus | *refinare* = to refine, purify |
//! | Predicate | Praedicatum | *praedicare* = to declare, assert |
//! | Constraint | Constrictio | *constringere* = to bind together |
//! | Evidence | Evidentia | *evidens* = visible, apparent |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::refined::{Refined, Positive, NonEmpty};
//!
//! // Create a positive integer
//! let pos: Refined<i32, Positive> = Refined::new(42).expect("42 is positive");
//!
//! // Create a non-empty string
//! let name: Refined<String, NonEmpty> = Refined::new("Alice".to_string()).unwrap();
//!
//! // Access the inner value
//! let val: i32 = pos.into_inner();
//! assert_eq!(val, 42);
//! ```

mod combinators;
mod common;
mod predicate;
mod wrapper;

pub use combinators::*;
pub use common::*;
pub use predicate::*;
pub use wrapper::*;
