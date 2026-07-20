//! Linearis - The core linear wrapper type
//!
//! > *"Usus unicus, valor perfectus"*
//! > — Single use, perfect value. (Neo-Latin)

use core::fmt;
use core::ops::Deref;

/// A linear wrapper that enforces single-use semantics.
///
/// `Linearis<T>` wraps a value and provides operations that consume the
/// wrapper, ensuring the inner value is used exactly once. This is useful
/// for resource management where you want to guarantee cleanup.
///
/// # Ownership and Linearity
///
/// Rust's ownership system already provides linear semantics through move
/// semantics. `Linearis` makes this explicit and provides additional
/// combinators for working with linear values ergonomically.
///
/// # Escape hatches (single-use is advisory)
///
/// Like `peek`, several impls deliberately weaken strict linearity:
///
/// - `Linearis<T>` derives **`Clone`/`Copy`** (for `T: Clone`/`T: Copy`),
///   so a copy can be consumed independently of the original;
/// - it implements **`Deref`**, so the inner value can be observed (and,
///   for interior-mutable `T`, mutated) without consuming the wrapper.
///
/// `Linearis` is therefore a *discipline aid*, not an enforcement
/// mechanism: treat clones, copies, and `Deref` access as escape hatches
/// you opt into knowingly.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::Linearis;
///
/// // Create a linear value
/// let linear_val = Linearis::new(42);
///
/// // Consume it with a function
/// let result = linear_val.consume_with(|x| x * 2);
/// assert_eq!(result, 84);
///
/// // linear_val is now consumed and cannot be used!
/// ```
///
/// # Mapping
///
/// ```rust
/// use ordofp_core::linear::Linearis;
///
/// let linear_val = Linearis::new(10);
/// let doubled = linear_val.fmap_linear(|x| x * 2);
/// let result = doubled.consume();
/// assert_eq!(result, 20);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Linearis<T> {
    value: T,
}

impl<T> Linearis<T> {
    /// Create a new linear value.
    ///
    /// The returned `Linearis<T>` must be consumed exactly once.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x = Linearis::new(42);
    /// ```
    #[inline]
    pub const fn new(value: T) -> Self {
        Linearis { value }
    }

    /// Consume the linear value, returning the inner value.
    ///
    /// This is the primary way to extract the value from a `Linearis`.
    /// After calling this method, the `Linearis` is consumed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x = Linearis::new(42);
    /// let inner = x.consume();
    /// assert_eq!(inner, 42);
    /// ```
    #[inline]
    pub fn consume(self) -> T {
        self.value
    }

    /// Consume the linear value by applying a function.
    ///
    /// This is equivalent to `f(self.consume())` but more ergonomic
    /// for chaining operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x = Linearis::new(10);
    /// let result = x.consume_with(|n| n * 2);
    /// assert_eq!(result, 20);
    /// ```
    #[inline]
    pub fn consume_with<B, F>(self, f: F) -> B
    where
        F: FnOnce(T) -> B,
    {
        f(self.value)
    }

    /// Map a function over the linear value, preserving linearity.
    ///
    /// This consumes the original `Linearis` and produces a new one
    /// containing the transformed value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x = Linearis::new(5);
    /// let y = x.fmap_linear(|n| n.to_string());
    /// assert_eq!(y.consume(), "5");
    /// ```
    #[inline]
    pub fn fmap_linear<B, F>(self, f: F) -> Linearis<B>
    where
        F: FnOnce(T) -> B,
    {
        Linearis::new(f(self.value))
    }

    /// Chain linear computations (monadic bind).
    ///
    /// Applies a function that produces a new `Linearis` to the inner
    /// value, flattening the result.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x = Linearis::new(5);
    /// let y = x.flat_map_linear(|n| Linearis::new(n * 2));
    /// assert_eq!(y.consume(), 10);
    /// ```
    #[inline]
    pub fn flat_map_linear<B, F>(self, f: F) -> Linearis<B>
    where
        F: FnOnce(T) -> Linearis<B>,
    {
        f(self.value)
    }

    /// Zip two linear values together.
    ///
    /// Consumes both values and produces a linear pair.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x = Linearis::new(1);
    /// let y = Linearis::new("hello");
    /// let pair = x.zip_linear(y);
    /// assert_eq!(pair.consume(), (1, "hello"));
    /// ```
    #[inline]
    pub fn zip_linear<B>(self, other: Linearis<B>) -> Linearis<(T, B)> {
        Linearis::new((self.value, other.value))
    }

    /// Apply a linear function to this linear value.
    ///
    /// This is the applicative `ap` operation for linear types.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let f = Linearis::new(|x: i32| x + 1);
    /// let x = Linearis::new(5);
    /// let result = x.ap_linear(f);
    /// assert_eq!(result.consume(), 6);
    /// ```
    #[inline]
    pub fn ap_linear<B, F>(self, f: Linearis<F>) -> Linearis<B>
    where
        F: FnOnce(T) -> B,
    {
        let func = f.consume();
        Linearis::new(func(self.value))
    }

    /// Replace the inner value, discarding the original.
    ///
    /// This is useful when you need to consume a linear value but
    /// don't care about its contents.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x = Linearis::new(42);
    /// let y = x.replace_linear("replaced");
    /// assert_eq!(y.consume(), "replaced");
    /// ```
    #[inline]
    pub fn replace_linear<B>(self, value: B) -> Linearis<B> {
        let _ = self.value;
        Linearis::new(value)
    }

    /// Void the inner value, consuming it.
    ///
    /// Returns a `Linearis<()>`, effectively discarding the value
    /// while maintaining the linear wrapper.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x = Linearis::new(42);
    /// let voided = x.void_linear();
    /// assert_eq!(voided.consume(), ());
    /// ```
    #[inline]
    pub fn void_linear(self) -> Linearis<()> {
        let _ = self.value;
        Linearis::new(())
    }

    /// Get a reference to the inner value without consuming.
    ///
    /// # Safety Note
    ///
    /// This breaks strict linearity by allowing observation without
    /// consumption. Use sparingly and prefer `consume_with` when possible.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x = Linearis::new(42);
    /// assert_eq!(*x.peek(), 42);
    /// let _ = x.consume(); // Still must consume
    /// ```
    #[inline]
    pub fn peek(&self) -> &T {
        &self.value
    }

    /// Convert to an Option, consuming the linear value.
    ///
    /// This always returns `Some(value)`, but is useful for
    /// interoperability with Option-based APIs.
    #[inline]
    pub fn into_option(self) -> Option<T> {
        Some(self.value)
    }

    /// Convert to a Result, consuming the linear value.
    ///
    /// This always returns `Ok(value)`, but is useful for
    /// interoperability with Result-based APIs.
    ///
    /// # Errors
    ///
    /// Never returns `Err`; the error type parameter exists only to satisfy
    /// Result-based call sites.
    #[inline]
    pub fn into_result<E>(self) -> Result<T, E> {
        Ok(self.value)
    }
}

impl<T> Linearis<Option<T>> {
    /// Transpose a `Linearis<Option<T>>` to `Option<Linearis<T>>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x = Linearis::new(Some(42));
    /// let transposed = x.transpose();
    /// assert_eq!(transposed.map(|l| l.consume()), Some(42));
    /// ```
    #[inline]
    pub fn transpose(self) -> Option<Linearis<T>> {
        self.value.map(Linearis::new)
    }
}

impl<T, E> Linearis<Result<T, E>> {
    /// Transpose a `Linearis<Result<T, E>>` to `Result<Linearis<T>, E>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::Linearis;
    ///
    /// let x: Linearis<Result<i32, &str>> = Linearis::new(Ok(42));
    /// let transposed = x.transpose_result();
    /// assert_eq!(transposed.map(|l| l.consume()), Ok(42));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err(e)` exactly when the wrapped value is `Err(e)`; the
    /// linear obligation transfers to the `Ok` payload only.
    #[inline]
    pub fn transpose_result(self) -> Result<Linearis<T>, E> {
        self.value.map(Linearis::new)
    }
}

impl<T: Default> Default for Linearis<T> {
    #[inline]
    fn default() -> Self {
        Linearis::new(T::default())
    }
}

impl<T: fmt::Debug> fmt::Debug for Linearis<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Linearis")
            .field("value", &self.value)
            .finish()
    }
}

impl<T: fmt::Display> fmt::Display for Linearis<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Linearis({})", self.value)
    }
}

impl<T> From<T> for Linearis<T> {
    #[inline]
    fn from(value: T) -> Self {
        Linearis::new(value)
    }
}

impl<T> Deref for Linearis<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Extension trait for converting values into linear wrappers.
pub trait LinearisExt: Sized {
    /// Wrap this value in a `Linearis`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::linear::LinearisExt;
    ///
    /// let x = 42.into_linearis();
    /// assert_eq!(x.consume(), 42);
    /// ```
    #[inline]
    fn into_linearis(self) -> Linearis<Self> {
        Linearis::new(self)
    }
}

impl<T> LinearisExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::string::ToString;

    #[test]
    fn test_new_and_consume() {
        let x = Linearis::new(42);
        assert_eq!(x.consume(), 42);
    }

    #[test]
    fn test_consume_with() {
        let x = Linearis::new(10);
        let result = x.consume_with(|n| n * 2);
        assert_eq!(result, 20);
    }

    #[test]
    fn test_fmap_linear() {
        let x = Linearis::new(5);
        let y = x.fmap_linear(|n| n * 3);
        assert_eq!(y.consume(), 15);
    }

    #[test]
    fn test_flat_map_linear() {
        let x = Linearis::new(5);
        let y = x.flat_map_linear(|n| Linearis::new(n + 10));
        assert_eq!(y.consume(), 15);
    }

    #[test]
    fn test_zip_linear() {
        let x = Linearis::new(1);
        let y = Linearis::new("hello");
        let pair = x.zip_linear(y);
        assert_eq!(pair.consume(), (1, "hello"));
    }

    #[test]
    fn test_ap_linear() {
        let f = Linearis::new(|x: i32| x + 1);
        let x = Linearis::new(5);
        let result = x.ap_linear(f);
        assert_eq!(result.consume(), 6);
    }

    #[test]
    fn test_replace_linear() {
        let x = Linearis::new(42);
        let y = x.replace_linear("replaced");
        assert_eq!(y.consume(), "replaced");
    }

    #[test]
    fn test_void_linear() {
        let x = Linearis::new(42);
        let voided = x.void_linear();
        assert_eq!(voided.consume(), ());
    }

    #[test]
    fn test_peek() {
        let x = Linearis::new(42);
        assert_eq!(*x.peek(), 42);
        assert_eq!(x.consume(), 42);
    }

    #[test]
    fn test_into_option() {
        let x = Linearis::new(42);
        assert_eq!(x.into_option(), Some(42));
    }

    #[test]
    fn test_into_result() {
        let x = Linearis::new(42);
        let result: Result<i32, ()> = x.into_result();
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_transpose_option() {
        let x = Linearis::new(Some(42));
        let transposed = x.transpose();
        assert_eq!(transposed.map(super::Linearis::consume), Some(42));

        let none: Linearis<Option<i32>> = Linearis::new(None);
        assert!(none.transpose().is_none());
    }

    #[test]
    fn test_transpose_result() {
        let x: Linearis<Result<i32, &str>> = Linearis::new(Ok(42));
        let transposed = x.transpose_result();
        assert_eq!(transposed.map(super::Linearis::consume), Ok(42));

        let err: Linearis<Result<i32, &str>> = Linearis::new(Err("error"));
        assert_eq!(err.transpose_result(), Err("error"));
    }

    #[test]
    fn test_from_trait() {
        let x: Linearis<i32> = 42.into();
        assert_eq!(x.consume(), 42);
    }

    #[test]
    fn test_linearis_ext() {
        let x = 42.into_linearis();
        assert_eq!(x.consume(), 42);
    }

    #[test]
    fn test_default() {
        let x: Linearis<i32> = Linearis::default();
        assert_eq!(x.consume(), 0);
    }

    #[test]
    fn test_debug() {
        let x = Linearis::new(42);
        let debug_str = format!("{x:?}");
        assert!(debug_str.contains("Linearis"));
        assert!(debug_str.contains("42"));
    }

    #[test]
    fn test_display() {
        let x = Linearis::new(42);
        let display_str = format!("{x}");
        assert_eq!(display_str, "Linearis(42)");
    }

    #[test]
    fn test_deref() {
        let x = Linearis::new(42);
        assert_eq!(*x, 42);
    }

    #[test]
    fn test_eq_ord() {
        let x = Linearis::new(42);
        let y = Linearis::new(42);
        let z = Linearis::new(43);

        assert_eq!(x, y);
        assert!(x < z);
    }

    #[test]
    fn test_chaining() {
        let result = Linearis::new(5i32)
            .fmap_linear(|x| x * 2)
            .fmap_linear(|x| x + 1)
            .flat_map_linear(|x: i32| Linearis::new(x.to_string()))
            .consume();

        assert_eq!(result, "11");
    }
}
