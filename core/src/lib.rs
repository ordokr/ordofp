#![no_std]
#![doc(html_playground_url = "https://play.rust-lang.org/")]
// =============================================================================
// Opt-in nightly acceleration (`nightly` cargo feature)
// =============================================================================
// The crate builds on stable Rust by default. The `nightly` feature enables
// unstable-Rust codegen improvements with identical semantics:
//
// - likely_unlikely: branch prediction hints (core::hint::likely/unlikely)
//   used by `hints`; 10-15% improvement in hot paths per hashbrown benchmarks.
//   On stable the hints are identity functions.
// - portable_simd (with `par`): core::simd vectorized f32 kernels in
//   `par::simd`. On stable the same API is backed by scalar loops.
#![cfg_attr(feature = "nightly", feature(likely_unlikely))]
#![cfg_attr(all(feature = "nightly", feature = "par"), feature(portable_simd))]
//! `OrdoFP` Core
//!
//! This library forms the core of `OrdoFP`. It should ideally be minimalistic,
//! containing only the fundamental building blocks of Universalis programming.
//!
//! # Examples
//!
//! ```
//! # use ordofp_core::hlist::*;
//! # use ordofp_core::{hlist, HList};
//! # fn main() {
//!
//! let h = hlist![1, false, 42f32];
//! let folded = h.foldr(hlist![|acc, i| i + acc,
//!     |acc, _| if acc > 42f32 { 9000 } else { 0 },
//!     |acc, f| f + acc],
//!     1f32);
//! assert_eq!(folded, 9001);
//!
//! // Reverse
//! let h1 = hlist![true, "hi"];
//! assert_eq!(h1.into_reverse(), hlist!["hi", true]);
//!
//! // foldr (foldl also available)
//! let h2 = hlist![1, false, 42f32];
//! let folded = h2.foldr(
//!             hlist![|acc, i| i + acc,
//!                    |acc, _| if acc > 42f32 { 9000 } else { 0 },
//!                    |acc, f| f + acc],
//!             1f32
//!     );
//! assert_eq!(folded, 9001);
//!
//! // Mapping over an HList
//! let h3 = hlist![9000, "joe", 41f32];
//! let mapped = h3.to_ref().map(hlist![|&n| n + 1,
//!                               |&s| s,
//!                               |&f| f + 1f32]);
//! assert_eq!(mapped, hlist![9001, "joe", 42f32]);
//!
//! // Plucking a value out by type
//! let h4 = hlist![1, "hello", true, 42f32];
//! let (t, remainder): (bool, _) = h4.pluck();
//! assert!(t);
//! assert_eq!(remainder, hlist![1, "hello", 42f32]);
//!
//! // Resculpting an HList
//! let h5 = hlist![9000, "joe", 41f32, true];
//! let (reshaped, remainder2): (HList![f32, i32, &str], _) = h5.sculpt();
//! assert_eq!(reshaped, hlist![41f32, 9000, "joe"]);
//! assert_eq!(remainder2, hlist![true]);
//! # }
//! ```
//!
//! Links:
//!   1. [Source on Github](https://github.com/ordokr/ordofp)
//!   2. [Crates.io page](https://crates.io/crates/ordofp)

#[cfg(any(test, feature = "std"))]
extern crate std;

// NOTE: `alloc` is linked unconditionally; the `alloc` FEATURE gates API surface only. A true no-alloc core build is not currently supported.
extern crate alloc;

// Exported macros (e.g. `nonempty!`) must not assume the consumer crate
// declares `extern crate alloc`; this hidden re-export gives their expansions
// a `$crate`-anchored path to `alloc` items.
#[doc(hidden)]
pub extern crate alloc as __alloc;

#[macro_use]
mod macros;

#[cfg(feature = "alloc")]
pub mod alternative;

// Async module (OrdoFP 2.0)
// Requires the "async" feature flag
#[cfg(feature = "async")]
pub mod async_core;

// Effects module (OrdoFP 2.0)
// Requires the "async" feature flag
#[cfg(feature = "async")]
pub mod effects;

// ParFlumen (data-parallel execution)
// Requires the "par" feature flag
#[cfg(feature = "par")]
pub mod par;

// Nexus phase
// Requires the "nexus" feature flag (which implies "alloc")
#[cfg(feature = "nexus")]
pub mod nexus;

// Linear types module (OrdoFP 3.0)
// Requires the "linear" feature flag
#[cfg(feature = "linear")]
pub mod linear;

// Dependent types module (OrdoFP 3.0)
// Requires the "dependent" feature flag (implies "alloc")
#[cfg(feature = "dependent")]
pub mod dependent;

// Recursion schemes module (OrdoFP 4.0)
// Requires the "alloc" feature flag
#[cfg(feature = "alloc")]
pub mod recursion;

// Quantitative types module (OrdoFP 4.0 Phase 4)
// Requires the "quantitative" feature flag (implies "alloc"); pulled in by "async"
#[cfg(feature = "quantitative")]
pub mod quantitative;

// Row polymorphism module (OrdoFP 4.0 Phase 5)
// Requires the "rows" feature flag (implies "alloc")
#[cfg(feature = "rows")]
pub mod rows;

// Free monads and Tagless Final module (OrdoFP 4.0 Phase 6)
// Requires the "alloc" feature flag
#[cfg(feature = "alloc")]
pub mod free;

// Category theory foundations module (OrdoFP 4.0 Phase 7)
// Requires the "alloc" feature flag
#[cfg(feature = "alloc")]
pub mod category;

// Multiplicity / row-effect research modules
#[cfg(feature = "alloc")]
pub mod arena;
pub mod diagnostics;
#[cfg(feature = "distributed")]
pub mod distributed;
#[cfg(feature = "alloc")]
pub mod easy;
#[cfg(feature = "alloc")]
pub mod metrics;
pub mod prelude;
#[cfg(feature = "alloc")]
pub mod refined;
pub mod specialization;
#[cfg(feature = "supervision")]
pub mod supervision;
#[cfg(feature = "alloc")]
pub mod tracing;
#[cfg(feature = "Probatum")]
pub mod validated;
pub mod vernacular;

pub mod bifunctor;
pub mod comonad;
pub mod datatypes;
pub mod disiunctio;
// FFI helper scaffolding — no callers in a pure-FP library,
// so gated off-by-default. Enable with the `ffi` feature.
#[cfg(feature = "ffi")]
pub mod ffi_bedrock;
#[cfg(feature = "alloc")]
pub mod fix;
pub mod foldable;
pub mod foldl;
pub mod gat;
pub mod hints;
pub mod hlist;
pub mod indices;
pub mod labelled;
#[cfg(feature = "alloc")]
pub mod nonempty;
pub mod optics;
pub mod path;
#[cfg(feature = "alloc")]
pub mod pfds;
pub mod tailrec;
pub mod traits;
pub mod transformers;
#[cfg(feature = "alloc")]
pub mod traversable;
mod tuples;
pub mod typeclasses;
pub mod universalis;
pub mod wrappers;
pub mod zipper;

#[cfg(test)]
mod test_structs;
