//! # Absurdum — The Void Type
//!
//! > *"Ex nihilo nihil fit."*
//! > — Nothing comes from nothing. (Parmenides)
//!
//! The `Absurdum` type represents logical impossibility - a type with no values.
//! It is the terminal absurdity, from which anything can be derived
//! (*ex falso quodlibet*).
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::datatypes::Absurdum;
//!
//! // A function that can never be called (no Absurdum values exist)
//! fn impossible(x: Absurdum) -> String {
//!     x.absurd() // Can return any type!
//! }
//! ```

use core::convert::Infallible;
use core::marker::PhantomData;

/// A type with no inhabitants - logical impossibility.
///
/// `Absurdum` (Latin: "the absurd") represents a type that cannot be
/// instantiated. It is isomorphic to `std::convert::Infallible` and
/// can be used to indicate that a branch of code is unreachable.
///
/// # Theory
///
/// In type theory, `Absurdum` is the *initial object* (or *bottom type*).
/// It has a unique morphism to any other type, expressed by the
/// `absurd` method. This corresponds to the logical principle:
///
/// > *Ex falso quodlibet* — From falsehood, anything follows.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::datatypes::Absurdum;
///
/// fn never_called(x: Absurdum) -> i32 {
///     x.absurd() // Type-safe unreachable code
/// }
///
/// // Absurdum can be used to make Result infallible:
/// fn always_succeeds() -> Result<i32, Absurdum> {
///     Ok(42)
/// }
///
/// // The Err case can never be constructed
/// let result = always_succeeds();
/// let value = match result {
///     Ok(v) => v,
///     Err(absurd) => absurd.absurd(),
/// };
/// assert_eq!(value, 42);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Absurdum {}

impl Absurdum {
    /// Derive any type from the absurd.
    ///
    /// Since `Absurdum` has no values, this method can never actually be called.
    /// However, it allows the type system to express that code after an
    /// `Absurdum` value is unreachable.
    ///
    /// # Latin Etymology
    ///
    /// *absurdum*: "the absurd, that which is out of tune"
    /// From *ab* (away from) + *surdus* (deaf, mute, unheard)
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Absurdum;
    ///
    /// fn demonstrate(x: Absurdum) -> String {
    ///     x.absurd() // Never called, but type-checks!
    /// }
    /// ```
    #[inline]
    pub fn absurd<T>(self) -> T {
        match self {}
    }
}

impl From<Infallible> for Absurdum {
    #[inline]
    fn from(x: Infallible) -> Self {
        match x {}
    }
}

impl From<Absurdum> for Infallible {
    #[inline]
    fn from(x: Absurdum) -> Self {
        match x {}
    }
}

// ============================================================================
// Unit — The Trivial Type
// ============================================================================

/// A wrapper around `()` with additional utility methods.
///
/// `Unitas` (Latin: "unity, oneness") is the terminal object in the
/// category of types - a type with exactly one value.
///
/// # Theory
///
/// While `()` (unit) is Rust's built-in unit type, `Unitas` provides
/// a named wrapper with explicit methods, useful when you want to
/// be clear about intent.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::datatypes::Unitas;
///
/// let unit = Unitas::unit();
/// assert_eq!(unit.into_inner(), ());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Unitas(());

impl Unitas {
    /// The single value of the unit type.
    #[inline]
    pub const fn unit() -> Self {
        Unitas(())
    }

    /// Extract the inner `()`.
    #[inline]
    pub const fn into_inner(self) {
        // Returns unit implicitly
    }

    /// Create from any value, discarding it.
    ///
    /// This is the unique morphism from any type to the terminal object.
    #[inline]
    pub fn from_any<T>(_: T) -> Self {
        Unitas(())
    }
}

impl From<()> for Unitas {
    #[inline]
    fn from((): ()) -> Self {
        Unitas(())
    }
}

impl From<Unitas> for () {
    #[inline]
    fn from(_: Unitas) -> Self {}
}

// ============================================================================
// PhantomType — Type-Level Marker
// ============================================================================

/// A zero-sized marker type for type-level programming.
///
/// `Phantasma` (Latin: "apparition, phantom") carries type information
/// without any runtime representation. It is useful for type-level
/// programming patterns.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::datatypes::Phantasma;
///
/// struct TypedId<T> {
///     id: u64,
///     _marker: Phantasma<T>,
/// }
///
/// impl<T> TypedId<T> {
///     fn new(id: u64) -> Self {
///         TypedId { id, _marker: Phantasma::new() }
///     }
/// }
///
/// // Different types, same runtime representation
/// let user_id: TypedId<String> = TypedId::new(1);
/// let post_id: TypedId<i32> = TypedId::new(1);
/// // user_id and post_id have different types but same size
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Phantasma<T>(PhantomData<T>);

impl<T> Phantasma<T> {
    /// Create a new phantom marker.
    #[inline]
    pub const fn new() -> Self {
        Phantasma(PhantomData)
    }

    /// Convert to a different phantom type.
    ///
    /// This is safe because phantom types have no runtime representation.
    #[inline]
    pub const fn transmute<U>(self) -> Phantasma<U> {
        Phantasma(PhantomData)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unitas() {
        let unit = Unitas::unit();
        unit.into_inner();
    }

    #[test]
    fn test_unitas_from_any() {
        let unit = Unitas::from_any(42);
        unit.into_inner();

        let unit = Unitas::from_any("hello");
        unit.into_inner();
    }

    #[test]
    fn test_unitas_conversions() {
        let unit: Unitas = ().into();
        let _inner: () = unit.into();
    }

    #[test]
    fn test_phantasma() {
        let p1: Phantasma<i32> = Phantasma::new();
        let _p2: Phantasma<u64> = p1.transmute();

        // Both have zero size
        assert_eq!(core::mem::size_of::<Phantasma<i32>>(), 0);
        assert_eq!(core::mem::size_of::<Phantasma<u64>>(), 0);
    }

    #[test]
    fn test_absurdum_size() {
        // Absurdum should be zero-sized (uninhabited types have no representation)
        assert_eq!(core::mem::size_of::<Absurdum>(), 0);
    }

    #[test]
    fn test_infallible_conversion() {
        // We can't actually test calling these functions, but we can verify they compile
        fn _from_infallible(x: Infallible) -> Absurdum {
            x.into()
        }

        fn _to_infallible(x: Absurdum) -> Infallible {
            x.into()
        }
    }
}
