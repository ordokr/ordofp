#![doc(html_playground_url = "https://play.rust-lang.org/")]
// Law-check functions take their inputs by value by design: they are
// quickcheck-style value properties whose arguments are consumed test data,
// and the by-value signatures keep call sites free of reference noise.
#![allow(clippy::needless_pass_by_value)]
//! # `OrdoFP` Laws
//!
//! This library contains property-based laws for testing implementations of
//! type classes declared in `OrdoFP`. Laws help ensure that your implementations
//! are mathematically correct and composable.
//!
//! ## Available Law Modules
//!
//! ### Core Algebraic Laws
//! - [`semigroup_laws`] - Associativity law for Semigroup
//! - [`monoid_laws`] - Identity laws for Monoid
//! - [`optics_laws`] - Lens (GetPut/PutGet/PutPut) and prism laws
//!
//! ### Functor Hierarchy Laws
//! - [`functor_laws`] - Identity and composition laws for GAT Functor
//! - [`applicative_laws`] - Laws for Applicative functors
//! - [`monad_laws`] - Left/right identity and associativity for Monad
//! - [`alternative_laws`] - Choice/failure laws for Alternative
//!
//! ### Async Type Class Laws (requires `async` feature)
//! - [`async_functor_laws`] - Identity and composition laws for `FunctorAsync`
//! - [`async_monad_laws`] - Left/right identity and associativity for `MonadAsync`
//!
//! ### Other Type Class Laws
//! - [`foldable_laws`] - Consistency laws for Foldable
//! - [`traversable_laws`] - Identity, naturality laws for Traversable
//! - [`comonad_laws`] - Extract/extend laws for Comonad
//! - [`bifunctor_laws`] - Identity and composition for Bifunctor
//! - [`category_laws`] - Identity and associativity laws for Category
//! - [`fixpoint_laws`] - Laws relating `Fix`, `cata`, and `ana`
//!
//! ## Usage with `QuickCheck`
//!
//! ```ignore
//! use ordofp_laws::functor_laws;
//! use quickcheck::quickcheck;
//!
//! quickcheck(functor_laws::identity::<Option<i32>> as fn(Option<i32>) -> bool);
//! ```

extern crate ordofp;
extern crate quickcheck;

pub mod is_eq;
pub mod wrapper;

// Core algebraic laws
pub mod monoid_laws;
pub mod optics_laws;
pub mod semigroup_laws;

// Functor hierarchy laws
pub mod alternative_laws;
pub mod applicative_laws;
pub mod functor_laws;
pub mod monad_laws;

// Async type class laws
#[cfg(feature = "async")]
pub mod async_functor_laws;
#[cfg(feature = "async")]
pub mod async_monad_laws;

// Other type class laws
pub mod bifunctor_laws;
pub mod category_laws;
pub mod comonad_laws;
pub mod fixpoint_laws;
pub mod foldable_laws;
pub mod traversable_laws;
