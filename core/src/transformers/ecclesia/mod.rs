//! CPS (Church-encoded) transformer variants.
//!
//! > *"Ecclesia Transformata"*
//! > — Transformed Church. (Neo-Latin)
//!
//! These transformers use continuation-passing style to achieve O(1) bind
//! composition, avoiding quadratic overhead in left-associated bind chains.
//!
//! Adapted third-party patterns are inventoried in `ORIGINAL_SOURCE.md` in
//! this directory and the repo-root `THIRD_PARTY_NOTICES.md`.
//!
//! # Example
//!
//! There is no dedicated `purus`/`return` constructor -- build a value that
//! ignores the environment with `new` instead.
//!
//! ```rust
//! use ordofp_core::transformers::ecclesia::LectorEcclesiaT;
//!
//! // O(1) bind composition even for left-associated chains
//! let program: LectorEcclesiaT<(), i32> = LectorEcclesiaT::new(|_env| 42)
//!     .flat_map(|x| LectorEcclesiaT::new(move |_env| x + 1))
//!     .flat_map(|x| LectorEcclesiaT::new(move |_env| x * 2));
//!
//! assert_eq!(program.run(()), 86);
//! ```

#![cfg(feature = "transformers-cps")]

pub mod lector_ecclesia;

pub use lector_ecclesia::LectorEcclesiaT;
