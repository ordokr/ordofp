//! # Core Datatypes
//!
//! This module contains fundamental functional programming datatypes.
//!
//! ## Available Types
//!
//! - [`Identitas`] - The identity functor (wraps values without context)
//! - [`Aut`] - The disjunction type (Either L or R)
//! - [`Const`] - The constant functor (phantom type computations)
//! - [`Pigritia`] - Lazy evaluation with memoization
//! - [`Absurdum`] - The void/never type (uninhabited)
//! - [`Unitas`] - The unit type wrapper
//! - [`Phantasma`] - Zero-sized type marker for type-level programming

mod aut;
pub mod constant;
mod identitas;
mod lazy;
mod void;

pub use aut::{Aut, AutIterDexter, AutIterSinister};
pub use constant::Const;
pub use identitas::Identitas;
pub use lazy::Pigritia;
pub use void::{Absurdum, Phantasma, Unitas};
