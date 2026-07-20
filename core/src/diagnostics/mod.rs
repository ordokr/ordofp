//! Diagnostics Module for `OrdoFP`
//!
//! Provides diagnostic *vocabulary* for tooling:
//! - Effect type pretty-printing (suitable for hover information)
//! - Effect mismatch diagnostics with clear error messages
//! - Type display formatting for complex effect rows
//!
//! # Overview
//!
//! This module helps developers understand complex effect types through:
//!
//! 1. **Effect Type Formatting** - Human-readable effect row display
//! 2. **Error Message Enhancement** - Clear diagnostics for effect mismatches
//! 3. **Type Simplification** - Reducing complex types to understandable forms
//!
//! # Integration status
//!
//! No tool consumes this module today — rust-analyzer and other IDE tooling
//! have no OrdoFP-specific hooks. The types here are a consumer-less
//! vocabulary intended for *future* tooling integration; within the crate
//! they are exercised only by their own tests.

mod display;
mod error_messages;
mod type_hints;

pub use display::*;
pub use error_messages::*;
pub use type_hints::*;
