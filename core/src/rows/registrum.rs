//! Registrum - Extensible Record Type
//!
//! > *"Registrum est liber in quo res gestae describuntur."*
//! > — A register is a book in which deeds are recorded.
//!
//! This module provides the `Registrum` type - an extensible record that
//! supports row polymorphism.

use core::fmt;

use crate::hlist::{Coniunctio, HList, Nihil};
use crate::labelled::Field;

use super::campus::{Confluo, Extendo, HabetCampum, Muto, Restricto};

// =============================================================================
// Registrum - Extensible Record
// =============================================================================

/// An extensible record type with row polymorphism support.
///
/// `Registrum` wraps an `HList` of labelled fields and provides a clean API
/// for record operations while supporting row-polymorphic functions.
///
/// # Latin Etymology
///
/// *Registrum* = register, record book
///
/// # Type Parameters
///
/// * `R` - The row type (`HList` of Field types)
///
/// # Example
///
/// ```rust
/// use ordofp_core::rows::Registrum;
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::indices::{Here, There};
///
/// type Name = (Ln, La, Lm, Le);
/// type Age = (La, Lg, Le);
///
/// let person = Registrum::new()
///     .extend_field::<Name, _>("name", "Alice")
///     .extend_field::<Age, _>("age", 30);
///
/// assert_eq!(*person.get::<Name, &str, There<Here>>(), "Alice");
/// assert_eq!(*person.get::<Age, i32, Here>(), 30);
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Registrum<R> {
    pub(crate) fields: R,
}

impl Registrum<Nihil> {
    /// Create a new empty record.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::rows::Registrum;
    ///
    /// let empty = Registrum::new();
    /// assert!(empty.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Registrum { fields: Nihil }
    }
}

impl Default for Registrum<Nihil> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<R: HList> Registrum<R> {
    /// Create a record from an `HList` of fields.
    ///
    /// This is primarily for internal use or advanced scenarios.
    #[inline]
    pub fn from_hlist(fields: R) -> Self {
        Registrum { fields }
    }

    /// Get the underlying `HList` of fields.
    #[inline]
    pub fn into_hlist(self) -> R {
        self.fields
    }

    /// Get a reference to the underlying `HList`.
    #[inline]
    pub fn as_hlist(&self) -> &R {
        &self.fields
    }

    /// Get the number of fields in this record.
    #[inline]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Check if the record has no fields.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl<R> Registrum<R> {
    /// Extend this record with a new field.
    ///
    /// # Type Parameters
    ///
    /// * `Label` - The type-level name for the new field
    /// * `Value` - The value type
    ///
    /// # Arguments
    ///
    /// * `name` - The runtime field name
    /// * `value` - The field value
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::rows::Registrum;
    /// use ordofp_core::labelled::chars::*;
    ///
    /// type Name = (Ln, La, Lm, Le);
    ///
    /// let record = Registrum::new().extend_field::<Name, _>("name", "Alice");
    /// assert_eq!(record.len(), 1);
    /// ```
    #[inline]
    pub fn extend_field<Label, Value>(
        self,
        name: &'static str,
        value: Value,
    ) -> Registrum<R::Output>
    where
        R: Extendo<Label, Value>,
    {
        Registrum {
            fields: self.fields.extend(name, value),
        }
    }

    /// Get a reference to a field by label.
    ///
    /// # Type Parameters
    ///
    /// * `Label` - The type-level field name
    /// * `Value` - The value type
    /// * `Index` - The index type (usually inferred)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::rows::Registrum;
    /// use ordofp_core::labelled::chars::*;
    /// use ordofp_core::indices::Here;
    ///
    /// type Age = (La, Lg, Le);
    ///
    /// let record = Registrum::new().extend_field::<Age, _>("age", 30i32);
    /// let age: &i32 = record.get::<Age, i32, Here>();
    /// assert_eq!(*age, 30);
    /// ```
    #[inline]
    pub fn get<Label, Value, Index>(&self) -> &Value
    where
        R: HabetCampum<Label, Value, Index>,
    {
        self.fields.get_field()
    }

    /// Get a mutable reference to a field by label.
    #[inline]
    pub fn get_mut<Label, Value, Index>(&mut self) -> &mut Value
    where
        R: HabetCampum<Label, Value, Index>,
    {
        self.fields.get_field_mut()
    }

    /// Remove a field from the record and return it.
    ///
    /// Returns a tuple of (`field_value`, `remaining_record`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::rows::Registrum;
    /// use ordofp_core::labelled::chars::*;
    /// use ordofp_core::indices::Here;
    ///
    /// type Age = (La, Lg, Le);
    ///
    /// let record = Registrum::new().extend_field::<Age, _>("age", 30i32);
    /// let (age, rest): (i32, _) = record.restrict::<Age, Here>();
    /// assert_eq!(age, 30);
    /// assert!(rest.is_empty());
    /// ```
    #[inline]
    pub fn restrict<Label, Index>(self) -> (R::Value, Registrum<R::Remainder>)
    where
        R: Restricto<Label, Index>,
    {
        let (value, remainder) = self.fields.restrict();
        (value, Registrum { fields: remainder })
    }

    /// Merge this record with another.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::rows::Registrum;
    /// use ordofp_core::labelled::chars::*;
    ///
    /// type Name = (Ln, La, Lm, Le);
    /// type Age = (La, Lg, Le);
    ///
    /// let name_record = Registrum::new().extend_field::<Name, _>("name", "Alice");
    /// let age_record = Registrum::new().extend_field::<Age, _>("age", 30i32);
    ///
    /// let person = name_record.merge(age_record);
    /// assert_eq!(person.len(), 2);
    /// ```
    #[inline]
    pub fn merge<Other>(self, other: Registrum<Other>) -> Registrum<R::Output>
    where
        R: Confluo<Other>,
    {
        Registrum {
            fields: self.fields.merge(other.fields),
        }
    }

    /// Modify a field with a function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::rows::Registrum;
    /// use ordofp_core::labelled::chars::*;
    /// use ordofp_core::indices::Here;
    ///
    /// type Age = (La, Lg, Le);
    ///
    /// let person = Registrum::new().extend_field::<Age, _>("age", 30i32);
    /// let older = person.modify::<Age, i32, Here, _>(|age| age + 1);
    /// let age: &i32 = older.get::<Age, i32, Here>();
    /// assert_eq!(*age, 31);
    /// ```
    #[inline]
    pub fn modify<Label, NewValue, Index, F>(self, f: F) -> Registrum<R::Output>
    where
        R: Muto<Label, NewValue, Index>,
        F: FnOnce(R::OldValue) -> NewValue,
    {
        Registrum {
            fields: self.fields.modify(f),
        }
    }
}

// =============================================================================
// Debug Implementation
// =============================================================================

impl fmt::Debug for Registrum<Nihil> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Registrum {{}}")
    }
}

impl<Label, Value: fmt::Debug, Tail: HList> fmt::Debug
    for Registrum<Coniunctio<Field<Label, Value>, Tail>>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Registrum {{ {}: {:?}, ... }}",
            self.fields.head.name, self.fields.head.value
        )
    }
}

// =============================================================================
// Row-Polymorphic Function Support
// =============================================================================

/// Trait for functions that work on records with specific fields.
///
/// This enables row-polymorphic functions that can accept any record
/// containing the required fields.
///
/// # Example
///
/// ```rust
/// use ordofp_core::rows::{Registrum, HabetCampum};
/// use ordofp_core::labelled::chars::*;
///
/// type Name = (Ln, La, Lm, Le);
///
/// fn greet<R, I>(record: &Registrum<R>) -> String
/// where
///     R: HabetCampum<Name, String, I>,
/// {
///     format!("Hello, {}!", record.get::<Name, String, I>())
/// }
///
/// let record = Registrum::new().extend_field::<Name, _>("name", String::from("Alice"));
/// assert_eq!(greet(&record), "Hello, Alice!");
/// ```
pub trait RegistrumExt<R> {
    /// Apply a function to the record.
    fn with<F, T>(self, f: F) -> T
    where
        F: FnOnce(Registrum<R>) -> T;
}

impl<R> RegistrumExt<R> for Registrum<R> {
    #[inline]
    fn with<F, T>(self, f: F) -> T
    where
        F: FnOnce(Registrum<R>) -> T,
    {
        f(self)
    }
}

// =============================================================================
// Conversion Traits
// =============================================================================

impl<R: HList> From<R> for Registrum<R> {
    #[inline]
    fn from(fields: R) -> Self {
        Registrum { fields }
    }
}

// Note: We can't implement From<Registrum<R>> for R due to orphan rules
// Users can use into_hlist() instead.

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indices::Here;
    use crate::labelled::chars::*;

    type Name = (Ln, La, Lm, Le);
    type Age = (La, Lg, Le);

    #[test]
    fn test_registrum_new() {
        let record = Registrum::new();
        assert!(record.is_empty());
        assert_eq!(record.len(), 0);
    }

    #[test]
    fn test_registrum_from_hlist() {
        let hlist = hlist![field!(Name, "Alice")];
        let record = Registrum::from_hlist(hlist);
        assert!(!record.is_empty());
        assert_eq!(record.len(), 1);
    }

    #[test]
    fn test_registrum_into_hlist() {
        let hlist = hlist![field!(Name, "Alice")];
        let record = Registrum::from_hlist(hlist);
        let back = record.into_hlist();
        assert_eq!(back, hlist);
    }

    #[test]
    fn test_registrum_extend() {
        let record = Registrum::new()
            .extend_field::<Name, _>("name", "Alice")
            .extend_field::<Age, _>("age", 30i32);

        assert_eq!(record.len(), 2);
    }

    #[test]
    fn test_registrum_get() {
        let record = Registrum::from_hlist(hlist![field!(Name, "Alice"), field!(Age, 30i32)]);

        let name: &&str = record.get::<Name, &str, Here>();
        assert_eq!(*name, "Alice");
    }

    #[test]
    fn test_registrum_restrict() {
        let record = Registrum::from_hlist(hlist![field!(Name, "Alice"), field!(Age, 30i32)]);

        let (name, rest): (&str, _) = record.restrict::<Name, Here>();
        assert_eq!(name, "Alice");
        assert_eq!(rest.len(), 1);
    }

    #[test]
    fn test_registrum_merge() {
        let r1 = Registrum::from_hlist(hlist![field!(Name, "Alice")]);
        let r2 = Registrum::from_hlist(hlist![field!(Age, 30i32)]);

        let merged = r1.merge(r2);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_registrum_modify() {
        let record = Registrum::from_hlist(hlist![field!(Age, 30i32)]);
        let modified = record.modify::<Age, i32, Here, _>(|age| age + 1);

        let age: &i32 = modified.get::<Age, i32, Here>();
        assert_eq!(*age, 31);
    }

    #[test]
    fn test_registrum_debug() {
        let record = Registrum::new();
        let debug = alloc::format!("{record:?}");
        assert!(debug.contains("Registrum"));
    }
}
