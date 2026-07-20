//! # Identitas (Identity Functor)
//!
//! The `Identitas` type represents the identity functor from category theory - the simplest
//! possible functor that wraps a value without adding any context or effects.
//!
//! ## Quick Start
//!
//! Simple wrapper with full monadic interface:
//!
//! ```rust
//! use ordofp_core::datatypes::Identitas;
//!
//! // Create identity values
//! let id = Identitas::new(42);
//! assert_eq!(id.unwrap(), 42);
//!
//! // Transform with map
//! let doubled = id.map(|x| x * 2);
//! assert_eq!(doubled.unwrap(), 84);
//!
//! // Chain with flat_map
//! let result = Identitas::new(10)
//!     .flat_map(|x| Identitas::new(x + 5))
//!     .flat_map(|x| Identitas::new(x * 2));
//! assert_eq!(result.unwrap(), 30);
//! ```
//!
//! ## Use Cases
//!
//! - Base case for monad transformer stacks
//! - Testing monadic code without effects
//! - Providing a consistent monadic interface for pure values
//! - Understanding monad laws and behavior
//!
//! ## Scholastic Naming
//!
//! Following `OrdoFP`'s Scholastic naming convention:
//! - `Identitas` - Latin for "identity"

use core::hash::Hash;

/// The identity functor - wraps a value without adding context or effects.
///
/// # Type Parameters
///
/// * `A` - The type of the wrapped value
///
/// # Examples
///
/// ```rust
/// use ordofp_core::datatypes::Identitas;
///
/// let x = Identitas::new(42);
/// assert_eq!(x.unwrap(), 42);
///
/// let doubled = x.map(|n| n * 2);
/// assert_eq!(doubled.unwrap(), 84);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Identitas<A> {
    value: A,
}

impl<A> Identitas<A> {
    /// Creates a new `Identitas` wrapping the given value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Identitas;
    ///
    /// let x = Identitas::new(42);
    /// assert_eq!(x.unwrap(), 42);
    /// ```
    #[inline]
    pub const fn new(value: A) -> Self {
        Identitas { value }
    }

    /// Extracts the inner value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Identitas;
    ///
    /// let x = Identitas::new(42);
    /// assert_eq!(x.unwrap(), 42);
    /// ```
    #[inline]
    pub fn unwrap(self) -> A {
        self.value
    }

    /// Returns a reference to the inner value.
    #[inline]
    pub const fn value_ref(&self) -> &A {
        &self.value
    }

    /// Returns a mutable reference to the inner value.
    #[inline]
    pub fn value_mut(&mut self) -> &mut A {
        &mut self.value
    }

    /// Converts to the inner value.
    #[inline]
    pub fn into_inner(self) -> A {
        self.value
    }

    /// Lifts a value into the Identitas context.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Identitas;
    ///
    /// let x = Identitas::<i32>::pure(42);
    /// assert_eq!(x.unwrap(), 42);
    /// ```
    #[inline]
    pub fn pure(value: A) -> Self {
        Identitas::new(value)
    }

    /// Maps a function over the wrapped value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Identitas;
    ///
    /// let x = Identitas::new(21);
    /// let doubled = x.map(|n| n * 2);
    /// assert_eq!(doubled.unwrap(), 42);
    /// ```
    #[inline]
    pub fn map<B, F>(self, f: F) -> Identitas<B>
    where
        F: FnOnce(A) -> B,
    {
        Identitas::new(f(self.value))
    }

    /// Monadic bind operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Identitas;
    ///
    /// let result = Identitas::new(5)
    ///     .flat_map(|x| Identitas::new(x + 3))
    ///     .flat_map(|x| Identitas::new(x * 2));
    /// assert_eq!(result.unwrap(), 16);
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> Identitas<B>
    where
        F: FnOnce(A) -> Identitas<B>,
    {
        f(self.value)
    }

    /// Alias for `flat_map`.
    #[inline]
    pub fn bind<B, F>(self, f: F) -> Identitas<B>
    where
        F: FnOnce(A) -> Identitas<B>,
    {
        self.flat_map(f)
    }

    /// Applicative apply operation.
    #[inline]
    pub fn apply<B, F>(self, mf: Identitas<F>) -> Identitas<B>
    where
        F: FnOnce(A) -> B,
    {
        Identitas::new((mf.value)(self.value))
    }

    /// Comonadic extract operation.
    #[inline]
    pub fn extract(self) -> A {
        self.value
    }

    /// Comonadic duplicate operation.
    #[inline]
    pub fn duplicate(self) -> Identitas<Identitas<A>> {
        Identitas::new(self)
    }

    /// Comonadic extend operation.
    #[inline]
    pub fn extend<B, F>(self, f: F) -> Identitas<B>
    where
        F: FnOnce(Identitas<A>) -> B,
    {
        Identitas::new(f(self))
    }

    /// Sequences two Identitas operations, discarding the first result.
    #[inline]
    pub fn then<B>(self, next: Identitas<B>) -> Identitas<B> {
        next
    }
}

impl<A> From<A> for Identitas<A> {
    #[inline]
    fn from(value: A) -> Self {
        Identitas::new(value)
    }
}

impl<A> AsRef<A> for Identitas<A> {
    #[inline]
    fn as_ref(&self) -> &A {
        &self.value
    }
}

impl<A> AsMut<A> for Identitas<A> {
    #[inline]
    fn as_mut(&mut self) -> &mut A {
        &mut self.value
    }
}

impl<A> IntoIterator for Identitas<A> {
    type Item = A;
    type IntoIter = core::iter::Once<A>;

    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_unwrap() {
        let x = Identitas::new(42);
        assert_eq!(x.unwrap(), 42);
    }

    #[test]
    fn test_map() {
        let x = Identitas::new(21);
        let doubled = x.map(|n| n * 2);
        assert_eq!(doubled.unwrap(), 42);
    }

    #[test]
    fn test_flat_map_chain() {
        let result = Identitas::new(5)
            .flat_map(|x| Identitas::new(x + 3))
            .flat_map(|x| Identitas::new(x * 2));
        assert_eq!(result.unwrap(), 16);
    }

    #[test]
    fn test_left_identity() {
        // pure(a).bind(f) == f(a)
        let a = 5;
        let f = |x: i32| Identitas::new(x * 2);

        let left = Identitas::pure(a).bind(f);
        let right = f(a);

        assert_eq!(left, right);
    }

    #[test]
    fn test_right_identity() {
        // m.bind(pure) == m
        let m = Identitas::new(42);
        let bound = m.bind(Identitas::pure);

        assert_eq!(Identitas::new(42), bound);
    }

    #[test]
    fn test_associativity() {
        // (m.bind(f)).bind(g) == m.bind(|x| f(x).bind(g))
        let m = Identitas::new(5);

        let left = m
            .bind(|x| Identitas::new(x + 1))
            .bind(|x| Identitas::new(x * 2));

        let right =
            Identitas::new(5).bind(|x| Identitas::new(x + 1).bind(|y| Identitas::new(y * 2)));

        assert_eq!(left, right);
    }

    #[test]
    fn test_comonad_extract() {
        let x = Identitas::new(42);
        assert_eq!(x.extract(), 42);
    }

    #[test]
    fn test_comonad_duplicate() {
        let x = Identitas::new(42);
        let dup = x.duplicate();
        assert_eq!(dup.unwrap().unwrap(), 42);
    }

    #[test]
    fn test_comonad_extend() {
        let x = Identitas::new(5);
        let squared = x.extend(|id| id.unwrap() * id.unwrap());
        assert_eq!(squared.unwrap(), 25);
    }
}
