//! Row Polymorphism - Extensible Records and Variants
//!
//! > *"Ordo est parium dispariumque rerum sua cuique loca tribuens dispositio."*
//! > — Order is the disposition of equal and unequal things, assigning each to its proper place. (St. Augustine)
//!
//! This module provides row polymorphism support inspired by PureScript and Elm,
//! enabling extensible records and variants with type-safe field operations.
//!
//! # Overview
//!
//! Row polymorphism allows functions to operate on records with "at least" certain
//! fields, without specifying the complete record type. This enables:
//!
//! - **Extensible Records**: Add fields to records while preserving type safety
//! - **Row-Polymorphic Functions**: Functions that work on any record with required fields
//! - **Extensible Variants**: Open sum types that can be extended with new cases
//!
//! # Latin Etymology
//!
//! - *Ordo* = row, rank, order (the type-level row)
//! - *Registrum* = record, register
//! - *Campus* = field
//! - *Variatio* = variant, variation
//! - *Extendo* = to extend
//! - *Restricto* = to restrict, remove
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::rows::{Registrum, HabetCampum, Extendo};
//! use ordofp_core::labelled::chars::*;
//! use ordofp_core::indices::{Here, There};
//!
//! type Name = (Ln, La, Lm, Le);
//! type Age = (La, Lg, Le);
//!
//! // Create a record with name and age (label type + runtime name + value)
//! let person = Registrum::new()
//!     .extend_field::<Name, _>("name", "Alice")
//!     .extend_field::<Age, _>("age", 30);
//!
//! // Access fields (value type and index must be given explicitly here,
//! // since nothing downstream constrains them)
//! let name: &&str = person.get::<Name, &str, There<Here>>();
//! assert_eq!(*name, "Alice");
//! ```

extern crate alloc;

mod campus;
mod ordo;
mod registrum;
mod variatio;

pub use campus::*;
pub use ordo::*;
pub use registrum::*;
pub use variatio::*;
