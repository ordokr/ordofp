//! Unrestricted - Explicitly non-linear values
//!
//! > *"Liber et absolutus"*
//! > — Free and unrestricted. (Latin)

use core::fmt;
use core::ops::{Deref, DerefMut};

/// A wrapper that explicitly marks a value as unrestricted (non-linear).
///
/// In a linear type system, `Unrestricted<T>` (or `Liber<T>`) marks values
/// that can be used multiple times. This is the opposite of [`Linearis`],
/// which enforces single-use semantics.
///
/// # When to Use
///
/// Use `Unrestricted` when:
/// - You need to explicitly document that a value can be freely copied/cloned
/// - Working with APIs that mix linear and non-linear types
/// - You want to "escape" from a linear context
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, Unrestricted};
///
/// fn process_linear(x: Linearis<i32>) -> i32 {
///     x.consume()
/// }
///
/// fn process_unrestricted(x: Unrestricted<i32>) -> (i32, i32) {
///     // Can use x multiple times because it's unrestricted
///     (*x, *x * 2)
/// }
///
/// assert_eq!(process_linear(Linearis::new(21)), 21);
/// assert_eq!(process_unrestricted(Unrestricted::new(21)), (21, 42));
/// ```
///
/// [`Linearis`]: crate::linear::Linearis
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Unrestricted<T> {
    value: T,
}

/// Type alias using Latin naming convention.
///
/// `Liber` means "free" in Latin, representing values free from linear constraints.
pub type Liber<T> = Unrestricted<T>;

impl<T> Unrestricted<T> {
    /// Create a new unrestricted value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Unrestricted;
    ///
    /// let x = Unrestricted::new(42);
    /// ```
    #[inline]
    pub const fn new(value: T) -> Self {
        Unrestricted { value }
    }

    /// Extract the inner value.
    ///
    /// Unlike `Linearis::consume`, this does not prevent further use
    /// of the wrapper (if it implements Clone).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Unrestricted;
    ///
    /// let x = Unrestricted::new(42);
    /// let inner = x.extract();
    /// assert_eq!(inner, 42);
    /// ```
    #[inline]
    pub fn extract(self) -> T {
        self.value
    }

    /// Get a reference to the inner value.
    #[inline]
    pub fn inner_ref(&self) -> &T {
        &self.value
    }

    /// Get a mutable reference to the inner value.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Map a function over the unrestricted value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Unrestricted;
    ///
    /// let x = Unrestricted::new(5);
    /// let y = x.fmap(|n| n * 2);
    /// assert_eq!(y.extract(), 10);
    /// ```
    #[inline]
    pub fn fmap<B, F>(self, f: F) -> Unrestricted<B>
    where
        F: FnOnce(T) -> B,
    {
        Unrestricted::new(f(self.value))
    }

    /// Chain computations on unrestricted values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Unrestricted;
    ///
    /// let x = Unrestricted::new(5);
    /// let y = x.flat_map(|n| Unrestricted::new(n + 10));
    /// assert_eq!(y.extract(), 15);
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> Unrestricted<B>
    where
        F: FnOnce(T) -> Unrestricted<B>,
    {
        f(self.value)
    }

    /// Duplicate an unrestricted value (requires Clone).
    ///
    /// This makes the "unrestricted" nature explicit - we can create
    /// as many copies as we want.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Unrestricted;
    ///
    /// let x = Unrestricted::new(42);
    /// let (a, b) = x.duplicate();
    /// assert_eq!(a.extract(), 42);
    /// assert_eq!(b.extract(), 42);
    /// ```
    #[inline]
    pub fn duplicate(&self) -> (Unrestricted<T>, Unrestricted<T>)
    where
        T: Clone,
    {
        (
            Unrestricted::new(self.value.clone()),
            Unrestricted::new(self.value.clone()),
        )
    }

    /// Zip two unrestricted values together.
    #[inline]
    pub fn zip<B>(self, other: Unrestricted<B>) -> Unrestricted<(T, B)> {
        Unrestricted::new((self.value, other.value))
    }
}

impl<T: Clone> Unrestricted<T> {
    /// Use the value without consuming the wrapper.
    ///
    /// This clones the inner value, demonstrating unrestricted use.
    #[inline]
    pub fn use_value(&self) -> T {
        self.value.clone()
    }
}

impl<T> From<T> for Unrestricted<T> {
    fn from(value: T) -> Self {
        Unrestricted::new(value)
    }
}

impl<T> Deref for Unrestricted<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Unrestricted<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: fmt::Debug> fmt::Debug for Unrestricted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Unrestricted")
            .field("value", &self.value)
            .finish()
    }
}

impl<T: fmt::Display> fmt::Display for Unrestricted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unrestricted({})", self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_extract() {
        let x = Unrestricted::new(42);
        assert_eq!(x.extract(), 42);
    }

    #[test]
    fn test_fmap() {
        let x = Unrestricted::new(5);
        let y = x.fmap(|n| n * 2);
        assert_eq!(y.extract(), 10);
    }

    #[test]
    fn test_flat_map() {
        let x = Unrestricted::new(5);
        let y = x.flat_map(|n| Unrestricted::new(n + 10));
        assert_eq!(y.extract(), 15);
    }

    #[test]
    fn test_duplicate() {
        let x = Unrestricted::new(42);
        let (a, b) = x.duplicate();
        assert_eq!(a.extract(), 42);
        assert_eq!(b.extract(), 42);
    }

    #[test]
    fn test_use_value() {
        let x = Unrestricted::new(42);
        assert_eq!(x.use_value(), 42);
        assert_eq!(x.use_value(), 42); // Can use multiple times
    }

    #[test]
    fn test_zip() {
        let x = Unrestricted::new(1);
        let y = Unrestricted::new("hello");
        let zipped = x.zip(y);
        assert_eq!(zipped.extract(), (1, "hello"));
    }

    #[test]
    fn test_deref() {
        let x = Unrestricted::new(42);
        assert_eq!(*x, 42);
    }

    #[test]
    fn test_deref_mut() {
        let mut x = Unrestricted::new(42);
        *x = 100;
        assert_eq!(*x, 100);
    }

    #[test]
    fn test_clone() {
        let x = Unrestricted::new(42);
        let y = x;
        assert_eq!(x.extract(), y.extract());
    }

    #[test]
    fn test_liber_alias() {
        let x: Liber<i32> = Liber::new(42);
        assert_eq!(x.extract(), 42);
    }
}
