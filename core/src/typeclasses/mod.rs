//! Type classes for algebraic structures and higher-kinded type simulation.
//!
//! > *\"Ordo est parium dispariumque rerum sua cuique loca tribuens dispositio.\"*
//! > — Order is the disposition of equal and unequal things, assigning each to its proper place. (Augustine)
//!
//! This module provides the core type classes used in functional programming,
//! following scholastic naming conventions:
//!
//! - [`Compositio`] (Semigroup) - Types with an associative binary operation
//! - [`Unitas`] (Monoid) - Compositio with an identity element
//! - [`Functor`] - Types that can be mapped over (Latin: *one who performs*)
//! - [`Applicatio`] (Applicative) - Functors with pure and apply
//! - [`Monad`] - Applicatio with `flat_map/bind` (Leibnizian heritage)
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::{Compositio, Unitas, Functor, Monad};
//!
//! // Compositio: combine values
//! let combined = Some(3).combine(&Some(4));
//! assert_eq!(combined, Some(7));
//!
//! // Unitas: combine with identity
//! use ordofp_core::typeclasses::combine_all;
//! let sum = combine_all(&[1, 2, 3, 4]);
//! assert_eq!(sum, 10);
//!
//! // Monad: chain computations
//! let result = Some(5)
//!     .map(|x| x * 2)
//!     .flat_map(|x| if x > 5 { Some(x) } else { None });
//! assert_eq!(result, Some(10));
//! ```

mod applicative;
pub mod arrow;
pub mod category;
#[cfg(feature = "alloc")]
pub mod contravariant;
mod functor;
#[cfg(feature = "alloc")]
pub mod genus;
pub mod hkt;
#[cfg(feature = "alloc")]
pub mod invariant;
pub mod map_n;
mod monad;
pub mod monad_error;
pub mod monad_plus;
mod monoid;
#[cfg(feature = "alloc")]
pub mod natural_transformation;
#[cfg(feature = "alloc")]
pub mod profunctor;
pub mod semigroup;
#[cfg(feature = "alloc")]
pub mod semigroupal;
pub mod tap;

// Re-export scholastic names
pub use semigroup::{Compositio, combine_all_option};

// Re-export all of monoid (its combine_n handles n=0 properly, so prefer it over semigroup's)
pub use monoid::*;

pub use applicative::*;
pub use functor::*;

pub use arrow::*;
pub use category::*;
pub use hkt::*;
pub use monad::*;
pub use monad_error::*;
pub use monad_plus::*;
pub use tap::*;

#[cfg(feature = "alloc")]
pub use contravariant::*;
#[cfg(feature = "alloc")]
pub use genus::*;
#[cfg(feature = "alloc")]
pub use invariant::*;
#[cfg(feature = "alloc")]
pub use natural_transformation::*;
#[cfg(feature = "alloc")]
pub use profunctor::*;
#[cfg(feature = "alloc")]
pub use semigroupal::*;
