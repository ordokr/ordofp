//! Dependent Type Foundations - Typus Dependens
//!
//! > *"Numerus est multitudo ex unitatibus conflata."*
//! > — A number is a multitude composed of units. (Euclid)
//!
//! This module provides dependent-type-style programming patterns where Rust
//! allows, inspired by Idris 2's Quantitative Type Theory and Haskell's
//! type-level programming.
//!
//! # Features
//!
//! - **Type-Level Naturals** ([`peano`]): Compile-time natural numbers (`Zero`, `Succ<N>`)
//! - **Length-Indexed Vectors** ([`vect`]): Vectors with compile-time length tracking
//! - **Proof Objects** ([`proof`]): Type-level witnesses and proofs
//! - **Refinement Types** ([`refinement`]): Value constraints encoded in types
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Zero | Nihil | *nihil* = nothing |
//! | Successor | Successor | *successor* = one who follows |
//! | Vector | Vectum | *vectum* = that which carries |
//! | Proof | Testimonium | *testimonium* = evidence, witness |
//! | Equal | Aequus | *aequus* = equal, level |
//! | Less Than | Minor | *minor* = smaller |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::dependent::peano::{Zero, Succ, N1, N2, N3};
//! use ordofp_core::dependent::vect::Vectum;
//! use ordofp_core::dependent::FromArray;
//!
//! // Create a length-indexed vector
//! let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
//!
//! // Head is guaranteed to exist (no Option needed)
//! let head: &i32 = v.caput();
//! assert_eq!(*head, 1);
//!
//! // Concatenation updates the type-level length
//! let v2: Vectum<i32, N2> = Vectum::from_array([4, 5]);
//! let combined: Vectum<i32, Succ<Succ<Succ<Succ<Succ<Zero>>>>>> = v.concatenare(v2);
//! assert_eq!(combined.len(), 5);
//! ```

pub mod peano;
pub mod proof;
pub mod refinement;
pub mod vect;

pub use peano::*;
pub use proof::*;
pub use refinement::*;
pub use vect::*;
