//! Recursion Schemes - Structured recursion combinators.
//!
//! > *"Recursio est iteratio in forma universali."*
//! > — Recursion is iteration in universal form.
//!
//! Recursion schemes (cata, ana, hylo, para, apo, zygo, histo, futu, chrono)
//! for structured recursion over data types described by a base functor.
//! Inspired by Haskell's `recursion-schemes` library.
//!
//! # Core Concepts
//!
//! ## Base Functor
//!
//! Every recursive type `T` has an associated **base functor** `Base<T>`.
//! For example, a list `[A]` has base functor `ListF<A, R>`:
//!
//! ```text
//! enum ListF<A, R> {
//!     Nil,
//!     Cons(A, R),
//! }
//! ```
//!
//! The base functor "unfolds" one layer of recursion.
//!
//! ## Morphisms
//!
//! | Morphism | Latin | Type | Description |
//! |----------|-------|------|-------------|
//! | `cata` | Catamorphismus | `(F<A> -> A) -> Fix<F> -> A` | Fold (tear down) |
//! | `ana` | Anamorphismus | `(A -> F<A>) -> A -> Fix<F>` | Unfold (build up) |
//! | `hylo` | Hylomorphismus | `(F<B> -> B) -> (A -> F<A>) -> A -> B` | Refold |
//! | `para` | Paramorphismus | `(F<(Fix<F>, A)> -> A) -> Fix<F> -> A` | Fold with subtree access |
//! | `apo` | Apomorphismus | `(A -> F<Either<Fix<F>, A>>) -> A -> Fix<F>` | Unfold with early termination |
//! | `histo` | Historiamorphismus | `(F<Cofree<F, A>> -> A) -> Fix<F> -> A` | Fold with history |
//! | `futu` | Futuramorphismus | `(A -> F<Free<F, A>>) -> A -> Fix<F>` | Unfold with future |
//! | `zygo` | Zygomorphismus | `(F<B> -> B) -> (F<(B, A)> -> A) -> Fix<F> -> A` | Fold with auxiliary |
//! | `chrono` | Chronomorphismus | `(F<Cofree<F, B>> -> B) -> (A -> F<Free<F, A>>) -> A -> B` | General refold |
//!
//! # Scholastic Naming
//!
//! - `Recursiva` - Trait for recursive structures
//! - `Corecursiva` - Trait for corecursive structures
//! - `FunctorBasis` - The base functor type family
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::recursion::{NatF, NatFWitness, cata};
//! use ordofp_core::fix::Fix;
//!
//! // `NatF` (the Peano natural number base functor) and its `Fix` witness
//! // already ship in this module.
//!
//! // Fold to convert a Peano natural to u64: the algebra sees one layer
//! // at a time, with recursive positions already folded to `u64`
//! fn to_u64(n: Fix<NatFWitness>) -> u64 {
//!     cata(|layer| match layer {
//!         NatF::ZeroF => 0,
//!         NatF::SuccF(prev) => prev + 1,
//!     }, n)
//! }
//!
//! // 3 = Succ(Succ(Succ(Zero)))
//! let three = Fix::new(NatF::SuccF(Fix::new(NatF::SuccF(Fix::new(NatF::SuccF(
//!     Fix::new(NatF::ZeroF),
//! ))))));
//! assert_eq!(to_u64(three), 3);
//! ```

mod base;
mod cofree;
mod free;
mod morphisms;
mod traits;

pub use base::*;
pub use cofree::*;
pub use free::*;
pub use morphisms::*;
pub use traits::*;
