//! Typed Optimization Combinators
//!
//! This module provides **explicit, opt-in** optimization combinators that
//! leverage effect type information for safety guarantees.
//!
//! # Design Philosophy
//!
//! Nexus does NOT automatically optimize your code. Instead, it provides
//! typed combinators that:
//!
//! 1. **Require explicit use** — You choose when to apply optimizations
//! 2. **Enforce preconditions** — The type system prevents unsafe usage
//! 3. **Document trade-offs** — Each combinator states its requirements
//!
//! # Available Optimizations
//!
//! | Combinator | Requirement | What It Does |
//! |------------|-------------|--------------|
//! | `par_map` | Pure effects only | Parallel mapping (type-safe) |
//! | `memoize` | Idempotent effects | Caching with configurable strategy |
//! | `speculative` | Total effects | Run multiple branches, take first success |
//! | `fuse` | Commutative effects | Combine operations |
//!
//! # Type Safety
//!
//! The type system enforces that optimizations are only applied when safe:
//!
//! ```rust
//! use ordofp_core::nexus::pure;
//! use ordofp_core::nexus::optim::par_map;
//!
//! // This compiles - Pure effects can be parallelized
//! let items = vec![1, 2, 3];
//! let results = par_map(&items, |x| pure(x * 2));
//! assert_eq!(results, vec![2, 4, 6]);
//!
//! // This does NOT compile - State effects cannot be parallelized
//! // let results = par_map(&items, |x| state_modify(|s| s + x));
//! //              ^^^^^^^ Error: Row<STATE_BIT> does not implement ParallelSafe
//! ```
//!
//! # Limitations
//!
//! - **No automatic insertion** — You must call combinators explicitly
//! - **No cross-function analysis** — Optimization scope is local
//! - **No compiler integration** — These are library-level abstractions
//! - **Parallel execution** — Currently sequential; requires rayon integration

pub mod commutativity;
pub mod fusion;
pub mod io_fast;
pub mod memoize;
pub mod parallel;
pub mod purity;
pub mod reader_fast;
pub mod speculative;
pub mod state_fast;
pub mod writer_fast;

pub use commutativity::{Commutative, EffectCommutes, NonCommutative};
pub use fusion::{CanFuse, FusionRule, fuse};
pub use io_fast::{FastIO, IoOp, IoOpExt, perform_io, pure_io};
pub use memoize::{CacheStrategy, Memoized, memoize};
pub use parallel::{par_map, par_sequence, par_traverse};
pub use purity::{EffectProperties, IsIdempotent, IsPure, IsTotal};
pub use reader_fast::{ReaderOp, ReaderOpExt, ask_reader, asks_reader, pure_reader};
pub use speculative::{first_success, race, speculative};
pub use state_fast::{FastState, StateOp, StateOpExt, get_op, modify_op, pure_op, put_op};
pub use writer_fast::{FastMonoid, WriterOp, WriterOpExt, pure_writer, tell_writer};
