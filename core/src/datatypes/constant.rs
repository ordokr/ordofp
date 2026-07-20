//! Const functor - phantom type computations.
//!
//! > *"Constans in mutabilitate."*
//! > — Constant in mutability. (Scholastic maxim)
//!
//! The `Const` type is a functor that ignores its second type parameter.
//! This is useful for collecting information during traversals, implementing
//! lenses, and other type-level programming patterns.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::datatypes::constant::{Const, get_const};
//!
//! // Const ignores the second type parameter
//! let c: Const<i32, String> = Const::new(42);
//! assert_eq!(c.get_const(), &42);
//!
//! // Map does nothing (phantom type changes, value stays the same)
//! let mapped: Const<i32, bool> = c.map(|_: String| true);
//! assert_eq!(mapped.get_const(), &42);
//! ```

use core::marker::PhantomData;

/// A type that holds a value of type `A` and ignores type `B`.
///
/// The `Const` functor is useful for:
/// - Collecting information during traversals
/// - Implementing getters in optics
/// - Type-level programming patterns
///
/// # Type Parameters
///
/// * `A` - The actual value type being held
/// * `B` - The phantom type (ignored)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Const<A, B> {
    value: A,
    _phantom: PhantomData<B>,
}

impl<A, B> Const<A, B> {
    /// Creates a new `Const` holding the given value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::constant::Const;
    ///
    /// let c: Const<i32, String> = Const::new(42);
    /// ```
    #[inline]
    pub const fn new(value: A) -> Self {
        Const {
            value,
            _phantom: PhantomData,
        }
    }

    /// Extracts the contained value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::constant::Const;
    ///
    /// let c: Const<i32, String> = Const::new(42);
    /// assert_eq!(c.get_const(), &42);
    /// ```
    #[inline]
    pub const fn get_const(&self) -> &A {
        &self.value
    }

    /// Consumes and returns the contained value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::constant::Const;
    ///
    /// let c: Const<i32, String> = Const::new(42);
    /// let value = c.into_const();
    /// assert_eq!(value, 42);
    /// ```
    #[inline]
    pub fn into_const(self) -> A {
        self.value
    }

    /// Maps the phantom type, leaving the value unchanged.
    ///
    /// This is the Functor instance for `Const A` - it ignores the function
    /// since `Const` doesn't actually contain a value of type `B`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::constant::Const;
    ///
    /// let c: Const<i32, String> = Const::new(42);
    /// let mapped: Const<i32, bool> = c.map(|_: String| true);
    /// assert_eq!(mapped.get_const(), &42);
    /// ```
    #[inline]
    pub fn map<C, F>(self, _f: F) -> Const<A, C>
    where
        F: FnOnce(B) -> C,
    {
        Const::new(self.value)
    }

    /// Reinterprets the phantom type.
    ///
    /// Since `Const` doesn't actually contain a value of type `B`,
    /// we can freely change it to any other type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::constant::Const;
    ///
    /// let c: Const<i32, String> = Const::new(42);
    /// let reinterpreted: Const<i32, Vec<u8>> = c.retag();
    /// assert_eq!(reinterpreted.get_const(), &42);
    /// ```
    #[inline]
    pub fn retag<C>(self) -> Const<A, C> {
        Const::new(self.value)
    }

    /// Maps over the constant value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::constant::Const;
    ///
    /// let c: Const<i32, String> = Const::new(42);
    /// let mapped: Const<i64, String> = c.map_const(|x| x as i64);
    /// assert_eq!(mapped.get_const(), &42i64);
    /// ```
    #[inline]
    pub fn map_const<C, F>(self, f: F) -> Const<C, B>
    where
        F: FnOnce(A) -> C,
    {
        Const::new(f(self.value))
    }
}

/// Extracts the constant value from a `Const`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::datatypes::constant::{Const, get_const};
///
/// let c: Const<i32, String> = Const::new(42);
/// assert_eq!(get_const(&c), &42);
/// ```
#[inline]
pub fn get_const<A, B>(c: &Const<A, B>) -> &A {
    c.get_const()
}

// ============================================================================
// Trait Implementations
// ============================================================================

impl<A, B> From<A> for Const<A, B> {
    #[inline]
    fn from(value: A) -> Self {
        Const::new(value)
    }
}

impl<A, B> AsRef<A> for Const<A, B> {
    #[inline]
    fn as_ref(&self) -> &A {
        &self.value
    }
}

// ============================================================================
// Applicative instance for Const when A is a Monoid
// ============================================================================

#[cfg(feature = "alloc")]
use crate::typeclasses::Unitas;

/// Combines two `Const` values using monoid combination.
///
/// This is the Applicative `ap` for `Const` - it combines the constant values
/// using the monoid operation.
#[cfg(feature = "alloc")]
impl<A: Unitas + Clone, B> Const<A, B> {
    /// The applicative pure for Const - returns the monoid identity.
    #[inline]
    pub fn pure_const<C>(_: C) -> Const<A, C> {
        Const::new(A::empty())
    }

    /// Combines two Const values using monoid combination.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::constant::Const;
    ///
    /// let c1: Const<i32, String> = Const::new(10);
    /// let c2: Const<i32, String> = Const::new(32);
    /// let combined: Const<i32, String> = c1.combine_const(&c2);
    /// assert_eq!(combined.get_const(), &42);
    /// ```
    #[inline]
    pub fn combine_const(&self, other: &Self) -> Self {
        Const::new(self.value.clone().combine(&other.value))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    #[test]
    fn test_const_new() {
        let c: Const<i32, String> = Const::new(42);
        assert_eq!(c.get_const(), &42);
    }

    #[test]
    fn test_const_into() {
        let c: Const<i32, String> = Const::new(42);
        assert_eq!(c.into_const(), 42);
    }

    #[test]
    fn test_const_map() {
        let c: Const<i32, String> = Const::new(42);
        let mapped: Const<i32, bool> = c.map(|_: String| true);
        assert_eq!(mapped.get_const(), &42);
    }

    #[test]
    fn test_const_retag() {
        let c: Const<i32, String> = Const::new(42);
        let retagged: Const<i32, Vec<u8>> = c.retag();
        assert_eq!(retagged.get_const(), &42);
    }

    #[test]
    fn test_const_map_const() {
        let c: Const<i32, String> = Const::new(42);
        let mapped: Const<i64, String> = c.map_const(i64::from);
        assert_eq!(mapped.get_const(), &42i64);
    }

    #[test]
    fn test_const_from() {
        let c: Const<i32, String> = 42.into();
        assert_eq!(c.get_const(), &42);
    }

    #[test]
    fn test_const_as_ref() {
        let c: Const<i32, String> = Const::new(42);
        let r: &i32 = c.as_ref();
        assert_eq!(r, &42);
    }

    #[test]
    fn test_const_eq() {
        let c1: Const<i32, String> = Const::new(42);
        let c2: Const<i32, String> = Const::new(42);
        let c3: Const<i32, String> = Const::new(0);
        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    #[test]
    fn test_const_clone() {
        let c1: Const<i32, String> = Const::new(42);
        let c2 = c1.clone();
        assert_eq!(c1, c2);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_const_combine() {
        let c1: Const<i32, String> = Const::new(10);
        let c2: Const<i32, String> = Const::new(32);
        let combined = c1.combine_const(&c2);
        assert_eq!(combined.get_const(), &42);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_const_pure() {
        let c: Const<i32, String> = Const::<i32, String>::pure_const("hello".to_string());
        assert_eq!(c.get_const(), &0); // i32 monoid empty is 0
    }

    // Functor Laws
    #[test]
    fn test_functor_identity_law() {
        // map(id) == id
        let c: Const<i32, String> = Const::new(42);
        let mapped: Const<i32, String> = c.clone().map(|x: String| x);
        assert_eq!(c, mapped);
    }

    #[test]
    fn test_functor_composition_law() {
        // map(f . g) == map(f) . map(g)
        let c: Const<i32, i32> = Const::new(42);

        let f = |x: i32| x.to_string();
        let g = |s: String| s.len();

        let left: Const<i32, usize> = c.map(|x| g(f(x)));
        let right: Const<i32, usize> = c.map(f).map(g);

        assert_eq!(left, right);
    }
}
