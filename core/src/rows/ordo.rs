//! Ordo - Row types for type-level field tracking
//!
//! > *"Ordo est recta ratio rerum ad finem."*
//! > — Order is the right arrangement of things toward an end. (Scholastic definition)
//!
//! This module defines the core row type abstractions for row polymorphism.

use core::marker::PhantomData;

use crate::hlist::{Coniunctio, HList, Nihil};

// =============================================================================
// Ordo - Row Kind
// =============================================================================

/// Trait representing a row type (type-level list of field labels).
///
/// A row tracks which fields are present in a record or variant at the type level.
/// This enables row-polymorphic functions that can work with any record
/// containing certain fields.
///
/// # Latin Etymology
///
/// *Ordo* = row, rank, order - the type-level representation of record structure.
pub trait Ordo {
    /// The field list type
    type Fields: HList;
}

/// Empty row - a record with no fields.
///
/// > *"Ex nihilo nihil fit."*
/// > — From nothing, nothing comes.
impl Ordo for Nihil {
    type Fields = Nihil;
}

/// Non-empty row - a record with at least one field.
impl<H, T: HList> Ordo for Coniunctio<H, T> {
    type Fields = Coniunctio<H, T>;
}

// =============================================================================
// Disjoint Rows
// =============================================================================

/// Marker trait indicating two rows have no overlapping field labels.
///
/// This is used to ensure type safety when merging records.
///
/// # Latin Etymology
///
/// *Disiunctus* = separated, disjoint
pub trait Disiunctus<Other: Ordo>: Ordo {}

/// Two empty rows are trivially disjoint.
impl Disiunctus<Nihil> for Nihil {}

/// An empty row is disjoint from any row.
impl<H, T: HList> Disiunctus<Coniunctio<H, T>> for Nihil {}

/// Any row is disjoint from an empty row.
impl<H, T: HList> Disiunctus<Nihil> for Coniunctio<H, T> {}

// =============================================================================
// Row Extension
// =============================================================================

/// Result of extending a row with a new field.
///
/// # Latin Etymology
///
/// *Extensio* = extension, stretching out
pub struct Extensio<Label, Row: Ordo> {
    _label: PhantomData<Label>,
    _row: PhantomData<Row>,
}

impl<Label, Row: Ordo> Ordo for Extensio<Label, Row> {
    type Fields = Coniunctio<Label, Row::Fields>;
}

// =============================================================================
// Row Restriction
// =============================================================================

/// Result of removing a field from a row.
///
/// # Latin Etymology
///
/// *Restrictio* = restriction, limitation
pub struct Restrictio<Label, Row: Ordo> {
    _label: PhantomData<Label>,
    _row: PhantomData<Row>,
}

// =============================================================================
// Row Union
// =============================================================================

/// The union of two disjoint rows.
///
/// # Latin Etymology
///
/// *Unio* = union, oneness
pub struct Unio<R1: Ordo, R2: Ordo> {
    _r1: PhantomData<R1>,
    _r2: PhantomData<R2>,
}

// =============================================================================
// Lack - Field Absence Constraint
// =============================================================================

/// Constraint indicating a row lacks a specific field label.
///
/// This is the dual of `HasField` - it asserts that a label is NOT present.
///
/// # Latin Etymology
///
/// *Carentia* = lack, absence
pub trait Carentia<Label>: Ordo {}

/// Empty row lacks all labels.
impl<Label> Carentia<Label> for Nihil {}

// =============================================================================
// Row Cons
// =============================================================================

/// Type-level cons for rows.
///
/// Adds a new label to a row, producing an extended row type.
pub type OrdineConiunctio<Label, Tail> = Coniunctio<Label, Tail>;

// =============================================================================
// Row Operations Helpers
// =============================================================================

/// Helper trait for computing row operations at the type level.
///
/// There is deliberately no closed type-level membership predicate here:
/// deciding "does this row contain `Label`?" as a type-level `Bool` needs
/// type-level label equality, which stable Rust cannot express. Membership
/// is instead *evidence-based*, in the qualified-types tradition: bound a
/// generic function by [`HabetCampum<Label, Value, Index>`](crate::rows::HabetCampum)
/// and the `Here`/`There` index the compiler infers is the membership proof.
pub trait OrdoOps: Ordo {
    /// Get the number of fields in this row.
    const NUMERUS: usize;
}

impl OrdoOps for Nihil {
    const NUMERUS: usize = 0;
}

impl<H, T: OrdoOps + HList> OrdoOps for Coniunctio<H, T> {
    const NUMERUS: usize = 1 + T::NUMERUS;
}

// =============================================================================
// Type-Level Booleans
// =============================================================================

/// Type-level boolean trait.
pub trait Bool {
    /// The runtime value this type-level boolean reifies to: `true` for
    /// [`Verum`], `false` for [`Falsum`].
    const VALUE: bool;
}

/// Type-level true.
///
/// # Latin Etymology
///
/// *Verum* = true, truth
pub struct Verum;

impl Bool for Verum {
    const VALUE: bool = true;
}

/// Type-level false.
///
/// # Latin Etymology
///
/// *Falsum* = false, falsehood
pub struct Falsum;

impl Bool for Falsum {
    const VALUE: bool = false;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nihil_is_ordo() {
        fn assert_ordo<T: Ordo>() {}
        assert_ordo::<Nihil>();
    }

    #[test]
    fn test_coniunctio_is_ordo() {
        fn assert_ordo<T: Ordo>() {}
        assert_ordo::<Coniunctio<(), Nihil>>();
    }

    #[test]
    fn test_ordo_ops_numerus() {
        assert_eq!(<Nihil as OrdoOps>::NUMERUS, 0);
        assert_eq!(<Coniunctio<(), Nihil> as OrdoOps>::NUMERUS, 1);
        assert_eq!(
            <Coniunctio<(), Coniunctio<(), Nihil>> as OrdoOps>::NUMERUS,
            2
        );
    }

    #[test]
    fn test_bool_values() {
        assert!(Verum::VALUE);
        assert!(!Falsum::VALUE);
    }
}
