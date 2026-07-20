//! Campus - Field access traits for row polymorphism
//!
//! > *"Campus est locus apertus."*
//! > — A field is an open place.
//!
//! This module provides traits for accessing, extending, and restricting
//! fields in row-polymorphic records.

use crate::hlist::{Coniunctio, HList, Nihil};
use crate::indices::{Here, There};
use crate::labelled::{Field, field_with_name};

// =============================================================================
// HasField - Field Presence Constraint
// =============================================================================

/// Trait indicating a row contains a field with a specific label and type.
///
/// This is the core trait for row polymorphism - it allows functions to
/// require that a record contains certain fields without specifying the
/// complete record type.
///
/// # Latin Etymology
///
/// *Habet* = has, possesses
/// *Campus* = field
///
/// # Example
///
/// ```rust
/// use ordofp_core::rows::HabetCampum;
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::indices::Here;
/// use ordofp_core::{hlist, field};
///
/// type Name = (Ln, La, Lm, Le);
///
/// fn get_name<R, I>(record: &R) -> &str
/// where
///     R: HabetCampum<Name, String, I>,
/// {
///     record.get_field()
/// }
///
/// let record = hlist![field!(Name, String::from("Alice"))];
/// assert_eq!(get_name::<_, Here>(&record), "Alice");
/// ```
pub trait HabetCampum<Label, Value, Index> {
    /// Get a reference to the field value.
    fn get_field(&self) -> &Value;

    /// Get a mutable reference to the field value.
    fn get_field_mut(&mut self) -> &mut Value;
}

/// Implementation when the field is at the head.
impl<Label, Value, Tail: HList> HabetCampum<Label, Value, Here>
    for Coniunctio<Field<Label, Value>, Tail>
{
    #[inline]
    fn get_field(&self) -> &Value {
        &self.head.value
    }

    #[inline]
    fn get_field_mut(&mut self) -> &mut Value {
        &mut self.head.value
    }
}

/// Implementation when the field is in the tail.
impl<Label, Value, Head, Tail, TailIndex> HabetCampum<Label, Value, There<TailIndex>>
    for Coniunctio<Head, Tail>
where
    Tail: HabetCampum<Label, Value, TailIndex>,
{
    #[inline]
    fn get_field(&self) -> &Value {
        self.tail.get_field()
    }

    #[inline]
    fn get_field_mut(&mut self) -> &mut Value {
        self.tail.get_field_mut()
    }
}

// =============================================================================
// Extendo - Record Extension
// =============================================================================

/// Trait for extending a record with a new field.
///
/// # Latin Etymology
///
/// *Extendo* = to extend, stretch out
///
/// # Example
///
/// ```rust
/// use ordofp_core::rows::Extendo;
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::hlist::Nihil;
///
/// type Name = (Ln, La, Lm, Le);
/// type Age = (La, Lg, Le);
///
/// let record = Extendo::<Age, i32>::extend(
///     Extendo::<Name, &str>::extend(Nihil, "name", "Alice"),
///     "age",
///     30,
/// );
/// assert_eq!(record.head.value, 30);
/// assert_eq!(record.tail.head.value, "Alice");
/// ```
pub trait Extendo<Label, Value>: Sized {
    /// The result type after extending.
    type Output;

    /// Extend the record with a new field.
    fn extend(self, name: &'static str, value: Value) -> Self::Output;
}

/// Extending Nihil creates a single-field record.
impl<Label, Value> Extendo<Label, Value> for Nihil {
    type Output = Coniunctio<Field<Label, Value>, Nihil>;

    #[inline]
    fn extend(self, name: &'static str, value: Value) -> Self::Output {
        Coniunctio {
            head: field_with_name(name, value),
            tail: Nihil,
        }
    }
}

/// Extending a non-empty record prepends the new field.
impl<Label, Value, Head, Tail: HList> Extendo<Label, Value> for Coniunctio<Head, Tail> {
    type Output = Coniunctio<Field<Label, Value>, Coniunctio<Head, Tail>>;

    #[inline]
    fn extend(self, name: &'static str, value: Value) -> Self::Output {
        Coniunctio {
            head: field_with_name(name, value),
            tail: self,
        }
    }
}

// =============================================================================
// Restricto - Field Removal
// =============================================================================

/// Trait for removing a field from a record.
///
/// Returns the field value and the record with the field removed.
///
/// # Latin Etymology
///
/// *Restricto* = to restrict, confine, remove
///
/// # Example
///
/// ```rust
/// use ordofp_core::rows::Restricto;
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::indices::Here;
/// use ordofp_core::{hlist, field};
///
/// type Age = (La, Lg, Le);
///
/// let record = hlist![field!(Age, 30i32)];
/// let (age, rest) = Restricto::<Age, Here>::restrict(record);
/// assert_eq!(age, 30);
/// assert!(rest.is_empty());
/// ```
pub trait Restricto<Label, Index>: Sized {
    /// The type of the removed value.
    type Value;

    /// The record type after removal.
    type Remainder;

    /// Remove a field by label and return it along with the remainder.
    fn restrict(self) -> (Self::Value, Self::Remainder);
}

/// Restricting when the field is at the head.
impl<Label, Value, Tail: HList> Restricto<Label, Here> for Coniunctio<Field<Label, Value>, Tail> {
    type Value = Value;
    type Remainder = Tail;

    #[inline]
    fn restrict(self) -> (Self::Value, Self::Remainder) {
        (self.head.value, self.tail)
    }
}

/// Restricting when the field is in the tail.
impl<Label, Head, Tail, TailIndex> Restricto<Label, There<TailIndex>> for Coniunctio<Head, Tail>
where
    Tail: Restricto<Label, TailIndex>,
{
    type Value = Tail::Value;
    type Remainder = Coniunctio<Head, Tail::Remainder>;

    #[inline]
    fn restrict(self) -> (Self::Value, Self::Remainder) {
        let (value, tail_remainder) = self.tail.restrict();
        (
            value,
            Coniunctio {
                head: self.head,
                tail: tail_remainder,
            },
        )
    }
}

// =============================================================================
// Merge - Record Merging
// =============================================================================

/// Trait for merging two records together.
///
/// The resulting record contains all fields from both records.
///
/// # Latin Etymology
///
/// *Confluo* = to flow together, merge
///
/// # Example
///
/// ```rust
/// use ordofp_core::rows::Confluo;
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::{hlist, field};
///
/// type Name = (Ln, La, Lm, Le);
/// type Age = (La, Lg, Le);
///
/// let name_record = hlist![field!(Name, "Alice")];
/// let age_record = hlist![field!(Age, 30i32)];
///
/// let person = name_record.merge(age_record);
/// assert_eq!(person.head.value, "Alice");
/// assert_eq!(person.tail.head.value, 30);
/// ```
pub trait Confluo<Other>: Sized {
    /// The merged record type.
    type Output;

    /// Merge this record with another.
    fn merge(self, other: Other) -> Self::Output;
}

/// Merging with empty record returns self.
impl<T: HList> Confluo<Nihil> for T {
    type Output = T;

    #[inline]
    fn merge(self, _: Nihil) -> Self::Output {
        self
    }
}

/// Merging empty record with any record returns the other.
impl<Head, Tail: HList> Confluo<Coniunctio<Head, Tail>> for Nihil {
    type Output = Coniunctio<Head, Tail>;

    #[inline]
    fn merge(self, other: Coniunctio<Head, Tail>) -> Self::Output {
        other
    }
}

/// Merging two non-empty records.
impl<H1, T1: HList + Confluo<Coniunctio<H2, T2>>, H2, T2: HList> Confluo<Coniunctio<H2, T2>>
    for Coniunctio<H1, T1>
{
    type Output = Coniunctio<H1, T1::Output>;

    #[inline]
    fn merge(self, other: Coniunctio<H2, T2>) -> Self::Output {
        Coniunctio {
            head: self.head,
            tail: self.tail.merge(other),
        }
    }
}

// =============================================================================
// Rename - Field Renaming
// =============================================================================

/// Trait for renaming a field in a record.
///
/// # Latin Etymology
///
/// *Renomino* = to rename
pub trait Renomino<OldLabel, NewLabel, Index>: Sized {
    /// The record type after renaming.
    type Output;

    /// Rename a field.
    fn rename(self, new_name: &'static str) -> Self::Output;
}

/// Renaming when the field is at the head.
impl<OldLabel, NewLabel, Value, Tail: HList> Renomino<OldLabel, NewLabel, Here>
    for Coniunctio<Field<OldLabel, Value>, Tail>
{
    type Output = Coniunctio<Field<NewLabel, Value>, Tail>;

    #[inline]
    fn rename(self, new_name: &'static str) -> Self::Output {
        Coniunctio {
            head: field_with_name(new_name, self.head.value),
            tail: self.tail,
        }
    }
}

/// Renaming when the field is in the tail.
impl<OldLabel, NewLabel, Head, Tail, TailIndex> Renomino<OldLabel, NewLabel, There<TailIndex>>
    for Coniunctio<Head, Tail>
where
    Tail: Renomino<OldLabel, NewLabel, TailIndex>,
{
    type Output = Coniunctio<Head, Tail::Output>;

    #[inline]
    fn rename(self, new_name: &'static str) -> Self::Output {
        Coniunctio {
            head: self.head,
            tail: self.tail.rename(new_name),
        }
    }
}

// =============================================================================
// Modify - Field Modification
// =============================================================================

/// Trait for modifying a field value in a record.
///
/// # Latin Etymology
///
/// *Muto* = to change, modify
pub trait Muto<Label, NewValue, Index>: Sized {
    /// The original value type.
    type OldValue;

    /// The record type after modification.
    type Output;

    /// Modify a field with a function.
    fn modify<F>(self, f: F) -> Self::Output
    where
        F: FnOnce(Self::OldValue) -> NewValue;
}

/// Modifying when the field is at the head.
impl<Label, OldValue, NewValue, Tail: HList> Muto<Label, NewValue, Here>
    for Coniunctio<Field<Label, OldValue>, Tail>
{
    type OldValue = OldValue;
    type Output = Coniunctio<Field<Label, NewValue>, Tail>;

    #[inline]
    fn modify<F>(self, f: F) -> Self::Output
    where
        F: FnOnce(Self::OldValue) -> NewValue,
    {
        Coniunctio {
            head: field_with_name(self.head.name, f(self.head.value)),
            tail: self.tail,
        }
    }
}

/// Modifying when the field is in the tail.
impl<Label, NewValue, Head, Tail, TailIndex> Muto<Label, NewValue, There<TailIndex>>
    for Coniunctio<Head, Tail>
where
    Tail: Muto<Label, NewValue, TailIndex>,
{
    type OldValue = Tail::OldValue;
    type Output = Coniunctio<Head, Tail::Output>;

    #[inline]
    fn modify<F>(self, f: F) -> Self::Output
    where
        F: FnOnce(Self::OldValue) -> NewValue,
    {
        Coniunctio {
            head: self.head,
            tail: self.tail.modify(f),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::labelled::chars::*;

    type Name = (Ln, La, Lm, Le);
    type Age = (La, Lg, Le);

    #[test]
    fn test_habet_campum_head() {
        let record = hlist![field!(Name, "Alice")];
        let name: &&str = HabetCampum::<Name, &str, Here>::get_field(&record);
        assert_eq!(*name, "Alice");
    }

    #[test]
    fn test_habet_campum_tail() {
        let record = hlist![field!(Age, 30i32), field!(Name, "Alice")];
        let name: &&str = HabetCampum::<Name, &str, There<Here>>::get_field(&record);
        assert_eq!(*name, "Alice");
    }

    #[test]
    fn test_extendo_nihil() {
        let record: Coniunctio<Field<Name, &str>, Nihil> =
            Extendo::<Name, &str>::extend(Nihil, "name", "Alice");
        assert_eq!(record.head.value, "Alice");
    }

    #[test]
    fn test_extendo_non_empty() {
        let record = hlist![field!(Name, "Alice")];
        let extended = Extendo::<Age, i32>::extend(record, "age", 30i32);
        assert_eq!(extended.head.value, 30);
        assert_eq!(extended.tail.head.value, "Alice");
    }

    #[test]
    fn test_restricto_head() {
        let record = hlist![field!(Name, "Alice"), field!(Age, 30i32)];
        let (name, rest): (&str, _) = Restricto::<Name, Here>::restrict(record);
        assert_eq!(name, "Alice");
        assert_eq!(rest.head.value, 30);
    }

    #[test]
    fn test_restricto_tail() {
        let record = hlist![field!(Age, 30i32), field!(Name, "Alice")];
        let (name, rest): (&str, _) = Restricto::<Name, There<Here>>::restrict(record);
        assert_eq!(name, "Alice");
        assert_eq!(rest.head.value, 30);
    }

    #[test]
    fn test_confluo_empty() {
        let record = hlist![field!(Name, "Alice")];
        let merged = record.merge(Nihil);
        assert_eq!(merged.head.value, "Alice");
    }

    #[test]
    fn test_confluo_two_records() {
        let r1 = hlist![field!(Name, "Alice")];
        let r2 = hlist![field!(Age, 30i32)];
        let merged = r1.merge(r2);
        assert_eq!(merged.head.value, "Alice");
        assert_eq!(merged.tail.head.value, 30);
    }

    #[test]
    fn test_muto() {
        let record = hlist![field!(Age, 30i32)];
        let modified = Muto::<Age, i32, Here>::modify(record, |age| age + 1);
        assert_eq!(modified.head.value, 31);
    }

    #[test]
    fn test_renomino() {
        type NewName = (Ln, Le, Lw, DoubleUnderscore, Ln, La, Lm, Le);
        let record = hlist![field!(Name, "Alice")];
        let renamed: Coniunctio<Field<NewName, &str>, Nihil> =
            Renomino::<Name, NewName, Here>::rename(record, "new_name");
        assert_eq!(renamed.head.name, "new_name");
        assert_eq!(renamed.head.value, "Alice");
    }
}
