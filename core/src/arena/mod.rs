//! Arena-Based Effect Handlers
//!
//! > *"In arena celeriter allocamus, simul liberamus."*
//! > — In the arena we allocate quickly, we free together. (Performance maxim)
//!
//! This module provides arena-based allocation for effect handlers to reduce
//! allocation overhead and improve cache locality.
//!
//! # Overview
//!
//! Traditional effect handlers allocate continuations on the heap, which can
//! cause performance overhead due to:
//! - Frequent small allocations
//! - Cache misses from scattered memory
//! - Deallocation overhead
//!
//! Arena allocation solves these problems by:
//! - Allocating from a contiguous memory block
//! - Freeing all allocations at once when the arena is dropped
//! - Improving cache locality
//!
//! # Usage
//!
//! ```rust
//! use ordofp_core::arena::with_arena;
//!
//! // Run a computation whose intermediate values live in the arena.
//! // (Drop of allocated values never runs — see `Arena` docs.)
//! let result = with_arena(|arena| {
//!     let x = arena.alloc(40);
//!     let y = arena.alloc(2);
//!     *x + *y
//! });
//! ```
//!
//! # Performance
//!
//! Arena allocation is particularly beneficial for:
//! - Short-lived effect handlers
//! - Deep effect stacks
//! - High-frequency effect operations

mod allocator;
mod pool;

pub use allocator::*;
pub use pool::*;
