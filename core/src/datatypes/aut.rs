//! # Aut (Either) - The Scholastic Disjunction Type
//!
//! > *"Aut Caesar aut nihil."* — Either Caesar or nothing. (Latin proverb)
//!
//! The `Aut` type represents values with two possibilities: a value of type `L` (Sinister/Left)
//! or a value of type `R` (Dexter/Right). Unlike `Result`, it carries no semantic meaning of
//! success or failure - both sides are equally valid.
//!
//! ## Quick Start
//!
//! ```rust
//! use ordofp_core::datatypes::Aut;
//!
//! // Create Aut values
//! let left: Aut<i32, String> = Aut::sinister(42);
//! let right: Aut<i32, String> = Aut::dexter("hello".to_string());
//!
//! // Pattern match
//! match left {
//!     Aut::Sinister(n) => println!("Got left: {}", n),
//!     Aut::Dexter(s) => println!("Got right: {}", s),
//! }
//!
//! // Map over the right value (functor-biased)
//! let mapped = Aut::<&str, i32>::dexter(10).map(|x| x * 2);
//! assert_eq!(mapped, Aut::dexter(20));
//! ```
//!
//! ## Scholastic Naming
//!
//! - `Aut` — Latin "or" (exclusive disjunction)
//! - `Sinister` — Left (Latin, originally meaning "left side")
//! - `Dexter` — Right (Latin, meaning "right side" or "skillful")
//!
//! ## Relationship to Result
//!
//! `Aut` can be converted to/from `Result`:
//!
//! ```rust
//! use ordofp_core::datatypes::Aut;
//!
//! let result: Result<i32, &str> = Ok(42);
//! let aut = Aut::from_result(result);
//! assert_eq!(aut, Aut::dexter(42));
//!
//! let back: Result<i32, &str> = aut.into_result();
//! assert_eq!(back, Ok(42));
//! ```

use core::fmt::{self, Debug};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A value that is either `Sinister(L)` (left) or `Dexter(R)` (right).
///
/// This is the scholastic equivalent of `Either` from Haskell or other FP languages.
/// Unlike `Result`, both variants are considered equally valid - there's no semantic
/// meaning of "success" or "failure".
///
/// # Functor Bias
///
/// Like Haskell's `Either`, `Aut` is right-biased for Functor/Monad operations.
/// The `map` and `flat_map` methods operate on the `Dexter` (right) variant,
/// passing `Sinister` (left) values through unchanged.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::datatypes::Aut;
///
/// let right: Aut<&str, i32> = Aut::dexter(42);
/// let left: Aut<&str, i32> = Aut::sinister("error");
///
/// // Map only affects the right value
/// assert_eq!(right.map(|x| x * 2), Aut::dexter(84));
/// assert_eq!(left.map(|x| x * 2), Aut::sinister("error"));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Aut<L, R> {
    /// Left value (sinister)
    Sinister(L),
    /// Right value (dexter)
    Dexter(R),
}

impl<L: Debug, R: Debug> Debug for Aut<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Aut::Sinister(l) => f.debug_tuple("Sinister").field(l).finish(),
            Aut::Dexter(r) => f.debug_tuple("Dexter").field(r).finish(),
        }
    }
}

impl<L, R> Aut<L, R> {
    // ========== Constructors ==========

    /// Creates a `Sinister` (left) value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, String> = Aut::sinister(42);
    /// assert!(left.is_sinister());
    /// ```
    #[inline]
    pub const fn sinister(l: L) -> Self {
        Aut::Sinister(l)
    }

    /// Creates a `Dexter` (right) value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let right: Aut<i32, String> = Aut::dexter("hello".to_string());
    /// assert!(right.is_dexter());
    /// ```
    #[inline]
    pub const fn dexter(r: R) -> Self {
        Aut::Dexter(r)
    }

    /// Alias for `sinister` - creates a left value.
    #[inline]
    pub const fn left(l: L) -> Self {
        Aut::Sinister(l)
    }

    /// Alias for `dexter` - creates a right value.
    #[inline]
    pub const fn right(r: R) -> Self {
        Aut::Dexter(r)
    }

    // ========== Predicates ==========

    /// Returns `true` if this is a `Sinister` (left) value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, &str> = Aut::sinister(42);
    /// let right: Aut<i32, &str> = Aut::dexter("hello");
    ///
    /// assert!(left.is_sinister());
    /// assert!(!right.is_sinister());
    /// ```
    #[inline]
    pub const fn is_sinister(&self) -> bool {
        matches!(self, Aut::Sinister(_))
    }

    /// Returns `true` if this is a `Dexter` (right) value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, &str> = Aut::sinister(42);
    /// let right: Aut<i32, &str> = Aut::dexter("hello");
    ///
    /// assert!(!left.is_dexter());
    /// assert!(right.is_dexter());
    /// ```
    #[inline]
    pub const fn is_dexter(&self) -> bool {
        matches!(self, Aut::Dexter(_))
    }

    /// Alias for `is_sinister`.
    #[inline]
    pub const fn is_left(&self) -> bool {
        self.is_sinister()
    }

    /// Alias for `is_dexter`.
    #[inline]
    pub const fn is_right(&self) -> bool {
        self.is_dexter()
    }

    // ========== Accessors ==========

    /// Returns a reference to the left value, if present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, &str> = Aut::sinister(42);
    /// assert_eq!(left.sinister_ref(), Some(&42));
    ///
    /// let right: Aut<i32, &str> = Aut::dexter("hello");
    /// assert_eq!(right.sinister_ref(), None);
    /// ```
    #[inline]
    pub const fn sinister_ref(&self) -> Option<&L> {
        match self {
            Aut::Sinister(l) => Some(l),
            Aut::Dexter(_) => None,
        }
    }

    /// Returns a reference to the right value, if present.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, &str> = Aut::sinister(42);
    /// assert_eq!(left.dexter_ref(), None);
    ///
    /// let right: Aut<i32, &str> = Aut::dexter("hello");
    /// assert_eq!(right.dexter_ref(), Some(&"hello"));
    /// ```
    #[inline]
    pub const fn dexter_ref(&self) -> Option<&R> {
        match self {
            Aut::Sinister(_) => None,
            Aut::Dexter(r) => Some(r),
        }
    }

    /// Alias for `sinister_ref`.
    #[inline]
    pub const fn left_ref(&self) -> Option<&L> {
        self.sinister_ref()
    }

    /// Alias for `dexter_ref`.
    #[inline]
    pub const fn right_ref(&self) -> Option<&R> {
        self.dexter_ref()
    }

    /// Converts into an `Option<L>`, discarding the right value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, &str> = Aut::sinister(42);
    /// assert_eq!(left.sinister_option(), Some(42));
    ///
    /// let right: Aut<i32, &str> = Aut::dexter("hello");
    /// assert_eq!(right.sinister_option(), None);
    /// ```
    #[inline]
    pub fn sinister_option(self) -> Option<L> {
        match self {
            Aut::Sinister(l) => Some(l),
            Aut::Dexter(_) => None,
        }
    }

    /// Converts into an `Option<R>`, discarding the left value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, &str> = Aut::sinister(42);
    /// assert_eq!(left.dexter_option(), None);
    ///
    /// let right: Aut<i32, &str> = Aut::dexter("hello");
    /// assert_eq!(right.dexter_option(), Some("hello"));
    /// ```
    #[inline]
    pub fn dexter_option(self) -> Option<R> {
        match self {
            Aut::Sinister(_) => None,
            Aut::Dexter(r) => Some(r),
        }
    }

    /// Alias for `sinister_option`.
    #[inline]
    pub fn left_option(self) -> Option<L> {
        self.sinister_option()
    }

    /// Alias for `right_option`.
    #[inline]
    pub fn right_option(self) -> Option<R> {
        self.dexter_option()
    }

    // ========== Unwrapping ==========

    /// Unwraps the left value, panicking if this is a right.
    ///
    /// # Panics
    ///
    /// Panics if called on a `Dexter` value.
    #[inline]
    pub fn unwrap_sinister(self) -> L {
        match self {
            Aut::Sinister(l) => l,
            Aut::Dexter(_) => panic!("called `unwrap_sinister` on a `Dexter` value"),
        }
    }

    /// Unwraps the right value, panicking if this is a left.
    ///
    /// # Panics
    ///
    /// Panics if called on a `Sinister` value.
    #[inline]
    pub fn unwrap_dexter(self) -> R {
        match self {
            Aut::Sinister(_) => panic!("called `unwrap_dexter` on a `Sinister` value"),
            Aut::Dexter(r) => r,
        }
    }

    /// Alias for `unwrap_sinister`.
    #[inline]
    pub fn unwrap_left(self) -> L {
        self.unwrap_sinister()
    }

    /// Alias for `unwrap_dexter`.
    #[inline]
    pub fn unwrap_right(self) -> R {
        self.unwrap_dexter()
    }

    /// Returns the left value or a default.
    #[inline]
    pub fn sinister_or(self, default: L) -> L {
        match self {
            Aut::Sinister(l) => l,
            Aut::Dexter(_) => default,
        }
    }

    /// Returns the right value or a default.
    #[inline]
    pub fn dexter_or(self, default: R) -> R {
        match self {
            Aut::Sinister(_) => default,
            Aut::Dexter(r) => r,
        }
    }

    /// Returns the left value or computes it from a closure.
    #[inline]
    pub fn sinister_or_else<F>(self, f: F) -> L
    where
        F: FnOnce(R) -> L,
    {
        match self {
            Aut::Sinister(l) => l,
            Aut::Dexter(r) => f(r),
        }
    }

    /// Returns the right value or computes it from a closure.
    #[inline]
    pub fn dexter_or_else<F>(self, f: F) -> R
    where
        F: FnOnce(L) -> R,
    {
        match self {
            Aut::Sinister(l) => f(l),
            Aut::Dexter(r) => r,
        }
    }

    // ========== Transformations ==========

    /// Maps a function over the right value (functor operation).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let right: Aut<&str, i32> = Aut::dexter(10);
    /// assert_eq!(right.map(|x| x * 2), Aut::dexter(20));
    ///
    /// let left: Aut<&str, i32> = Aut::sinister("error");
    /// assert_eq!(left.map(|x| x * 2), Aut::sinister("error"));
    /// ```
    #[inline]
    pub fn map<U, F>(self, f: F) -> Aut<L, U>
    where
        F: FnOnce(R) -> U,
    {
        match self {
            Aut::Sinister(l) => Aut::Sinister(l),
            Aut::Dexter(r) => Aut::Dexter(f(r)),
        }
    }

    /// Maps a function over the left value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, &str> = Aut::sinister(10);
    /// assert_eq!(left.map_sinister(|x| x * 2), Aut::sinister(20));
    ///
    /// let right: Aut<i32, &str> = Aut::dexter("hello");
    /// assert_eq!(right.map_sinister(|x| x * 2), Aut::dexter("hello"));
    /// ```
    #[inline]
    pub fn map_sinister<U, F>(self, f: F) -> Aut<U, R>
    where
        F: FnOnce(L) -> U,
    {
        match self {
            Aut::Sinister(l) => Aut::Sinister(f(l)),
            Aut::Dexter(r) => Aut::Dexter(r),
        }
    }

    /// Alias for `map_sinister`.
    #[inline]
    pub fn map_left<U, F>(self, f: F) -> Aut<U, R>
    where
        F: FnOnce(L) -> U,
    {
        self.map_sinister(f)
    }

    /// Maps functions over both sides (bifunctor operation).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, &str> = Aut::sinister(10);
    /// assert_eq!(left.bimap(|x| x * 2, |s| s.len()), Aut::sinister(20));
    ///
    /// let right: Aut<i32, &str> = Aut::dexter("hello");
    /// assert_eq!(right.bimap(|x| x * 2, |s| s.len()), Aut::dexter(5));
    /// ```
    #[inline]
    pub fn bimap<U, V, F, G>(self, f: F, g: G) -> Aut<U, V>
    where
        F: FnOnce(L) -> U,
        G: FnOnce(R) -> V,
    {
        match self {
            Aut::Sinister(l) => Aut::Sinister(f(l)),
            Aut::Dexter(r) => Aut::Dexter(g(r)),
        }
    }

    /// Chains a computation on the right value (monad operation).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// fn safe_div(a: i32, b: i32) -> Aut<&'static str, i32> {
    ///     if b == 0 {
    ///         Aut::sinister("division by zero")
    ///     } else {
    ///         Aut::dexter(a / b)
    ///     }
    /// }
    ///
    /// let result = Aut::dexter(10).flat_map(|x| safe_div(x, 2));
    /// assert_eq!(result, Aut::dexter(5));
    ///
    /// let error = Aut::dexter(10).flat_map(|x| safe_div(x, 0));
    /// assert_eq!(error, Aut::sinister("division by zero"));
    ///
    /// let left: Aut<&str, i32> = Aut::sinister("initial error");
    /// let still_left = left.flat_map(|x| safe_div(x, 2));
    /// assert_eq!(still_left, Aut::sinister("initial error"));
    /// ```
    #[inline]
    pub fn flat_map<U, F>(self, f: F) -> Aut<L, U>
    where
        F: FnOnce(R) -> Aut<L, U>,
    {
        match self {
            Aut::Sinister(l) => Aut::Sinister(l),
            Aut::Dexter(r) => f(r),
        }
    }

    /// Alias for `flat_map`.
    #[inline]
    pub fn and_then<U, F>(self, f: F) -> Aut<L, U>
    where
        F: FnOnce(R) -> Aut<L, U>,
    {
        self.flat_map(f)
    }

    /// Swaps left and right.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, &str> = Aut::sinister(42);
    /// assert_eq!(left.swap(), Aut::dexter(42));
    ///
    /// let right: Aut<i32, &str> = Aut::dexter("hello");
    /// assert_eq!(right.swap(), Aut::sinister("hello"));
    /// ```
    #[inline]
    pub fn swap(self) -> Aut<R, L> {
        match self {
            Aut::Sinister(l) => Aut::Dexter(l),
            Aut::Dexter(r) => Aut::Sinister(r),
        }
    }

    /// Flattens a nested `Aut<L, Aut<L, R>>` into `Aut<L, R>`.
    #[inline]
    pub fn flatten(self) -> Aut<L, R>
    where
        R: Into<Aut<L, R>>,
    {
        match self {
            Aut::Sinister(l) => Aut::Sinister(l),
            Aut::Dexter(r) => r.into(),
        }
    }

    // ========== Result Conversion ==========

    /// Converts from `Result<R, L>` to `Aut<L, R>`.
    ///
    /// Note: `Ok` becomes `Dexter`, `Err` becomes `Sinister`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let ok: Result<i32, &str> = Ok(42);
    /// assert_eq!(Aut::from_result(ok), Aut::dexter(42));
    ///
    /// let err: Result<i32, &str> = Err("error");
    /// assert_eq!(Aut::from_result(err), Aut::sinister("error"));
    /// ```
    #[inline]
    pub fn from_result(result: Result<R, L>) -> Self {
        match result {
            Ok(r) => Aut::Dexter(r),
            Err(l) => Aut::Sinister(l),
        }
    }

    /// Converts to `Result<R, L>`.
    ///
    /// Note: `Dexter` becomes `Ok`, `Sinister` becomes `Err`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let right: Aut<&str, i32> = Aut::dexter(42);
    /// assert_eq!(right.into_result(), Ok(42));
    ///
    /// let left: Aut<&str, i32> = Aut::sinister("error");
    /// assert_eq!(left.into_result(), Err("error"));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` carrying the left value exactly when `self` is
    /// [`Aut::Sinister`]; [`Aut::Dexter`] becomes `Ok`. This is a pure
    /// conversion — no new error condition is introduced.
    #[inline]
    pub fn into_result(self) -> Result<R, L> {
        match self {
            Aut::Sinister(l) => Err(l),
            Aut::Dexter(r) => Ok(r),
        }
    }

    // ========== Folding ==========

    /// Folds the `Aut` into a single value by providing functions for each case.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, &str> = Aut::sinister(42);
    /// let result = left.fold(|l| l.to_string(), |r| r.to_string());
    /// assert_eq!(result, "42");
    ///
    /// let right: Aut<i32, &str> = Aut::dexter("hello");
    /// let result = right.fold(|l| l.to_string(), |r| r.to_string());
    /// assert_eq!(result, "hello");
    /// ```
    #[inline]
    pub fn fold<U, F, G>(self, on_left: F, on_right: G) -> U
    where
        F: FnOnce(L) -> U,
        G: FnOnce(R) -> U,
    {
        match self {
            Aut::Sinister(l) => on_left(l),
            Aut::Dexter(r) => on_right(r),
        }
    }

    /// Merges both sides into a single type when they're the same.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Aut;
    ///
    /// let left: Aut<i32, i32> = Aut::sinister(42);
    /// assert_eq!(left.merge(), 42);
    ///
    /// let right: Aut<i32, i32> = Aut::dexter(100);
    /// assert_eq!(right.merge(), 100);
    /// ```
    #[inline]
    pub fn merge(self) -> L
    where
        L: From<R>,
    {
        match self {
            Aut::Sinister(l) => l,
            Aut::Dexter(r) => L::from(r),
        }
    }
}

impl<L, R> From<Result<R, L>> for Aut<L, R> {
    #[inline]
    fn from(result: Result<R, L>) -> Self {
        Aut::from_result(result)
    }
}

impl<L, R> From<Aut<L, R>> for Result<R, L> {
    #[inline]
    fn from(aut: Aut<L, R>) -> Self {
        aut.into_result()
    }
}

impl<L: Default, R> Default for Aut<L, R> {
    /// Default is a `Sinister` with the default left value.
    #[inline]
    fn default() -> Self {
        Aut::Sinister(L::default())
    }
}

// ========== Iterator Support ==========

/// An iterator that yields the right value of an `Aut`, if present.
pub struct AutIterDexter<R> {
    inner: Option<R>,
}

impl<R> Iterator for AutIterDexter<R> {
    type Item = R;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = usize::from(self.inner.is_some());
        (n, Some(n))
    }
}

impl<R> ExactSizeIterator for AutIterDexter<R> {}

/// An iterator that yields the left value of an `Aut`, if present.
pub struct AutIterSinister<L> {
    inner: Option<L>,
}

impl<L> Iterator for AutIterSinister<L> {
    type Item = L;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.take()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = usize::from(self.inner.is_some());
        (n, Some(n))
    }
}

impl<L> ExactSizeIterator for AutIterSinister<L> {}

impl<L, R> Aut<L, R> {
    /// Returns an iterator over the right value.
    #[inline]
    pub fn iter_dexter(self) -> AutIterDexter<R> {
        AutIterDexter {
            inner: self.dexter_option(),
        }
    }

    /// Returns an iterator over the left value.
    #[inline]
    pub fn iter_sinister(self) -> AutIterSinister<L> {
        AutIterSinister {
            inner: self.sinister_option(),
        }
    }
}

impl<L, R> IntoIterator for Aut<L, R> {
    type Item = R;
    type IntoIter = AutIterDexter<R>;

    /// By default, iterating over an `Aut` yields the right value (if present).
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_dexter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::format;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn test_constructors() {
        let left: Aut<i32, &str> = Aut::sinister(42);
        let right: Aut<i32, &str> = Aut::dexter("hello");

        assert!(left.is_sinister());
        assert!(!left.is_dexter());
        assert!(!right.is_sinister());
        assert!(right.is_dexter());
    }

    #[test]
    fn test_accessors() {
        let left: Aut<i32, &str> = Aut::sinister(42);
        let right: Aut<i32, &str> = Aut::dexter("hello");

        assert_eq!(left.sinister_ref(), Some(&42));
        assert_eq!(left.dexter_ref(), None);
        assert_eq!(right.sinister_ref(), None);
        assert_eq!(right.dexter_ref(), Some(&"hello"));
    }

    #[test]
    fn test_map() {
        let right: Aut<&str, i32> = Aut::dexter(10);
        let left: Aut<&str, i32> = Aut::sinister("error");

        assert_eq!(right.map(|x| x * 2), Aut::dexter(20));
        assert_eq!(left.map(|x| x * 2), Aut::sinister("error"));
    }

    #[test]
    fn test_map_sinister() {
        let left: Aut<i32, &str> = Aut::sinister(10);
        let right: Aut<i32, &str> = Aut::dexter("hello");

        assert_eq!(left.map_sinister(|x| x * 2), Aut::sinister(20));
        assert_eq!(right.map_sinister(|x| x * 2), Aut::dexter("hello"));
    }

    #[test]
    fn test_bimap() {
        let left: Aut<i32, &str> = Aut::sinister(10);
        let right: Aut<i32, &str> = Aut::dexter("hello");

        assert_eq!(left.bimap(|x| x * 2, str::len), Aut::sinister(20));
        assert_eq!(right.bimap(|x| x * 2, str::len), Aut::dexter(5));
    }

    #[test]
    fn test_flat_map() {
        fn safe_div(a: i32, b: i32) -> Aut<&'static str, i32> {
            if b == 0 {
                Aut::sinister("division by zero")
            } else {
                Aut::dexter(a / b)
            }
        }

        let result = Aut::dexter(10).flat_map(|x| safe_div(x, 2));
        assert_eq!(result, Aut::dexter(5));

        let error = Aut::dexter(10).flat_map(|x| safe_div(x, 0));
        assert_eq!(error, Aut::sinister("division by zero"));

        let left: Aut<&str, i32> = Aut::sinister("initial error");
        let still_left = left.flat_map(|x| safe_div(x, 2));
        assert_eq!(still_left, Aut::sinister("initial error"));
    }

    #[test]
    fn test_swap() {
        let left: Aut<i32, &str> = Aut::sinister(42);
        let right: Aut<i32, &str> = Aut::dexter("hello");

        assert_eq!(left.swap(), Aut::dexter(42));
        assert_eq!(right.swap(), Aut::sinister("hello"));
    }

    #[test]
    fn test_result_conversion() {
        let ok: Result<i32, &str> = Ok(42);
        let err: Result<i32, &str> = Err("error");

        assert_eq!(Aut::from_result(ok), Aut::dexter(42));
        assert_eq!(Aut::from_result(err), Aut::sinister("error"));

        let right: Aut<&str, i32> = Aut::dexter(42);
        let left: Aut<&str, i32> = Aut::sinister("error");

        assert_eq!(right.into_result(), Ok(42));
        assert_eq!(left.into_result(), Err("error"));
    }

    #[test]
    fn test_fold() {
        let left: Aut<i32, &str> = Aut::sinister(42);
        let right: Aut<i32, &str> = Aut::dexter("hello");

        assert_eq!(
            left.fold(|l| format!("left: {l}"), |r| format!("right: {r}")),
            "left: 42"
        );
        assert_eq!(
            right.fold(|l| format!("left: {l}"), |r| format!("right: {r}")),
            "right: hello"
        );
    }

    #[test]
    fn test_merge() {
        let left: Aut<i32, i32> = Aut::sinister(42);
        let right: Aut<i32, i32> = Aut::dexter(100);

        assert_eq!(left.merge(), 42);
        assert_eq!(right.merge(), 100);
    }

    #[test]
    fn test_unwrap_or() {
        let left: Aut<i32, &str> = Aut::sinister(42);
        let right: Aut<i32, &str> = Aut::dexter("hello");

        assert_eq!(left.sinister_or(0), 42);
        assert_eq!(right.sinister_or(0), 0);
        assert_eq!(left.dexter_or("default"), "default");
        assert_eq!(right.dexter_or("default"), "hello");
    }

    #[test]
    fn test_iterator() {
        let right: Aut<&str, i32> = Aut::dexter(42);
        let left: Aut<&str, i32> = Aut::sinister("error");

        let right_vals: Vec<i32> = right.into_iter().collect();
        let left_vals: Vec<i32> = left.into_iter().collect();

        assert_eq!(right_vals, vec![42]);
        assert!(left_vals.is_empty());
    }

    #[test]
    fn test_functor_identity_law() {
        // fmap id == id
        let right: Aut<&str, i32> = Aut::dexter(42);
        let left: Aut<&str, i32> = Aut::sinister("error");

        assert_eq!(right.map(|x| x), right);
        assert_eq!(left.map(|x| x), left);
    }

    #[test]
    fn test_functor_composition_law() {
        // fmap (f . g) == fmap f . fmap g
        let right: Aut<&str, i32> = Aut::dexter(10);
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        let composed = right.map(|x| g(f(x)));
        let sequential = right.map(f).map(g);

        assert_eq!(composed, sequential);
    }

    #[test]
    fn test_monad_left_identity() {
        // return a >>= f == f a
        let a = 10;
        let f = |x: i32| Aut::<&str, i32>::dexter(x * 2);

        let left_side: Aut<&str, i32> = Aut::dexter(a).flat_map(f);
        let right_side = f(a);

        assert_eq!(left_side, right_side);
    }

    #[test]
    fn test_monad_right_identity() {
        // m >>= return == m
        let m: Aut<&str, i32> = Aut::dexter(42);

        let result = m.flat_map(Aut::dexter);

        assert_eq!(result, m);
    }

    #[test]
    fn test_monad_associativity() {
        // (m >>= f) >>= g == m >>= (\x -> f x >>= g)
        let m: Aut<&str, i32> = Aut::dexter(5);
        let f = |x: i32| Aut::<&str, i32>::dexter(x + 1);
        let g = |x: i32| Aut::<&str, i32>::dexter(x * 2);

        let left_side = m.flat_map(f).flat_map(g);
        let right_side = m.flat_map(|x| f(x).flat_map(g));

        assert_eq!(left_side, right_side);
    }

    /// `dexter_or_else` on a `Dexter` must return the inner value without
    /// invoking the closure.  On a `Sinister` it must apply the closure to
    /// the error and return the result — the primary recovery edge case.
    ///
    /// `sinister_or_else` mirrors the same contract on the other variant.
    #[test]
    fn test_or_else_recovery_from_error() {
        // dexter_or_else: Dexter branch returns the value, closure is NOT called.
        let right: Aut<&str, usize> = Aut::dexter(99);
        let result = right.dexter_or_else(|_e| panic!("closure must not be called for Dexter"));
        assert_eq!(
            result, 99,
            "Dexter.dexter_or_else must return the inner value"
        );

        // dexter_or_else: Sinister branch applies the closure to compute a fallback.
        // This is the key edge case: recovering from an error by deriving a
        // default from the error value itself (e.g. converting error length → 0).
        let left: Aut<&str, usize> = Aut::sinister("err");
        let recovered = left.dexter_or_else(str::len);
        assert_eq!(
            recovered, 3,
            "Sinister.dexter_or_else must apply the closure to the error"
        );

        // sinister_or_else: Sinister branch returns the left value, closure is NOT called.
        let left2: Aut<i32, &str> = Aut::sinister(7);
        let s = left2.sinister_or_else(|_r| panic!("closure must not be called for Sinister"));
        assert_eq!(
            s, 7,
            "Sinister.sinister_or_else must return the inner value"
        );

        // sinister_or_else: Dexter branch applies the closure to the right value.
        let right2: Aut<i32, &str> = Aut::dexter("hello");
        let s2 = right2.sinister_or_else(|r| r.len() as i32);
        assert_eq!(
            s2, 5,
            "Dexter.sinister_or_else must apply the closure to the right value"
        );
    }
}
