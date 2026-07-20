//! Alternative type class for applicative functors with choice.
//!
//! `Alternative` extends `Applicative` with operations for representing
//! failure and choice between computations.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::alternative::Alternative;
//!
//! // Option: None represents failure, Some represents success
//! let a: Option<i32> = Some(1);
//! let b: Option<i32> = Some(2);
//! let c: Option<i32> = None;
//!
//! // alt chooses the first success
//! assert_eq!(a.alt(&b), Some(1));
//! assert_eq!(c.alt(&b), Some(2));
//! assert_eq!(c.alt(&c), None);
//!
//! // guard for conditional computation
//! assert_eq!(<Option<()> as Alternative>::guard(true), Some(()));
//! assert_eq!(<Option<()> as Alternative>::guard(false), None);
//! ```

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Alternative functor - an applicative with monoid structure.
///
/// Provides operations for choice (`alt`) and failure (`empty`).
///
/// # Laws
///
/// 1. **Left identity**: `empty().alt(x) == x`
/// 2. **Right identity**: `x.alt(empty()) == x`
/// 3. **Associativity**: `a.alt(b).alt(c) == a.alt(b.alt(c))`
pub trait Alternative: Sized + Clone {
    /// The empty/failure value (identity for `alt`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::alternative::Alternative;
    ///
    /// assert_eq!(Option::<i32>::empty(), None);
    /// assert_eq!(Vec::<i32>::empty(), Vec::new());
    /// ```
    fn empty() -> Self;

    /// Choose between two alternatives.
    ///
    /// Returns `self` if it represents success, otherwise returns `other`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::alternative::Alternative;
    ///
    /// let a = Some(1);
    /// let b = Some(2);
    /// let c: Option<i32> = None;
    ///
    /// assert_eq!(a.alt(&b), Some(1));
    /// assert_eq!(c.alt(&b), Some(2));
    /// ```
    fn alt(&self, other: &Self) -> Self;

    /// Conditional failure.
    ///
    /// Returns a success value if the condition is true,
    /// otherwise returns the empty/failure value.
    fn guard(condition: bool) -> Self;

    /// Check if this is the empty/failure value.
    fn is_empty(&self) -> bool;

    /// Combine with a default value.
    ///
    /// If `self` is the empty value, return `default`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::alternative::Alternative;
    ///
    /// let none: Option<i32> = None;
    /// let some = Some(5);
    ///
    /// assert_eq!(none.or_else_alt(|| Some(10)), Some(10));
    /// assert_eq!(some.or_else_alt(|| Some(10)), Some(5));
    /// ```
    #[inline]
    fn or_else_alt<F>(&self, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        if self.is_empty() { f() } else { self.clone() }
    }

    /// Filter based on a predicate, returning empty if false.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::alternative::Alternative;
    ///
    /// let x = Some(5);
    /// assert_eq!(x.filter_alt(|opt| opt.is_some()), Some(5));
    ///
    /// let none: Option<i32> = None;
    /// assert_eq!(none.filter_alt(|opt| opt.is_some()), None);
    /// ```
    #[inline]
    fn filter_alt<F>(&self, pred: F) -> Self
    where
        F: FnOnce(&Self) -> bool,
    {
        if pred(self) {
            self.clone()
        } else {
            Self::empty()
        }
    }
}

// Implementation for Option<T> where T: Clone + Default
impl<T: Clone + Default> Alternative for Option<T> {
    #[inline]
    fn empty() -> Self {
        None
    }

    #[inline]
    fn alt(&self, other: &Self) -> Self {
        self.clone().or_else(|| other.clone())
    }

    #[inline]
    fn guard(condition: bool) -> Self {
        if condition { Some(T::default()) } else { None }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_none()
    }
}

// Implementation for Vec
#[cfg(feature = "alloc")]
impl<T: Clone + Default> Alternative for Vec<T> {
    #[inline]
    fn empty() -> Self {
        Vec::new()
    }

    #[inline]
    fn alt(&self, other: &Self) -> Self {
        if self.is_empty() {
            other.clone()
        } else {
            self.clone()
        }
    }

    #[inline]
    fn guard(condition: bool) -> Self {
        if condition {
            alloc::vec![T::default()]
        } else {
            Vec::new()
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }
}

/// Extension trait for Result to work with Alternative-like patterns.
pub trait ResultAlt<T, E> {
    /// Convert Result to Option, discarding the error.
    fn ok_alt(self) -> Option<T>;

    /// Choose between two Results.
    fn alt_result(self, other: Self) -> Self;
}

impl<T, E> ResultAlt<T, E> for Result<T, E> {
    #[inline]
    fn ok_alt(self) -> Option<T> {
        self.ok()
    }

    #[inline]
    fn alt_result(self, other: Self) -> Self {
        self.or(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_empty() {
        assert_eq!(Option::<i32>::empty(), None);
    }

    #[test]
    fn test_option_alt_both_some() {
        let a = Some(1);
        let b = Some(2);
        assert_eq!(a.alt(&b), Some(1));
    }

    #[test]
    fn test_option_alt_first_none() {
        let a: Option<i32> = None;
        let b = Some(2);
        assert_eq!(a.alt(&b), Some(2));
    }

    #[test]
    fn test_option_alt_both_none() {
        let a: Option<i32> = None;
        let b: Option<i32> = None;
        assert_eq!(a.alt(&b), None);
    }

    #[test]
    fn test_option_guard_true() {
        assert_eq!(Option::<()>::guard(true), Some(()));
    }

    #[test]
    fn test_option_guard_false() {
        assert_eq!(Option::<()>::guard(false), None);
    }

    #[test]
    fn test_option_is_empty() {
        assert!(Option::<i32>::empty().is_empty());
        assert!(!Some(1).is_empty());
    }

    #[test]
    fn test_vec_empty() {
        assert_eq!(Vec::<i32>::empty(), Vec::<i32>::new());
    }

    #[test]
    fn test_vec_alt_first_non_empty() {
        let a = alloc::vec![1, 2];
        let b = alloc::vec![3, 4];
        assert_eq!(a.alt(&b), alloc::vec![1, 2]);
    }

    #[test]
    fn test_vec_alt_first_empty() {
        let a: Vec<i32> = alloc::vec![];
        let b = alloc::vec![3, 4];
        assert_eq!(a.alt(&b), alloc::vec![3, 4]);
    }

    #[test]
    fn test_vec_is_empty() {
        assert!(Vec::<i32>::empty().is_empty());
        assert!(!alloc::vec![1].is_empty());
    }

    #[test]
    fn test_left_identity_law() {
        // empty().alt(x) == x
        let x = Some(42);
        assert_eq!(Option::<i32>::empty().alt(&x), x);

        let v = alloc::vec![1, 2, 3];
        assert_eq!(Vec::<i32>::empty().alt(&v), v);
    }

    #[test]
    fn test_right_identity_law() {
        // x.alt(empty()) == x
        let x = Some(42);
        assert_eq!(x.alt(&Option::empty()), x);

        let v = alloc::vec![1, 2, 3];
        assert_eq!(v.alt(&Vec::empty()), v);
    }

    #[test]
    fn test_associativity_law() {
        // a.alt(b).alt(c) == a.alt(b.alt(c))
        let a = Some(1);
        let b = Some(2);
        let c = Some(3);
        assert_eq!(a.alt(&b).alt(&c), a.alt(&b.alt(&c)));

        let none: Option<i32> = None;
        assert_eq!(none.alt(&b).alt(&c), none.alt(&b.alt(&c)));
    }

    #[test]
    fn test_filter_alt() {
        let x = Some(5);
        assert_eq!(x.filter_alt(core::option::Option::is_some), Some(5));

        let none: Option<i32> = None;
        assert_eq!(none.filter_alt(core::option::Option::is_some), None);
    }

    #[test]
    fn test_result_alt() {
        let ok: Result<i32, &str> = Ok(1);
        let err: Result<i32, &str> = Err("error");
        let ok2: Result<i32, &str> = Ok(2);

        assert_eq!(ok.alt_result(ok2), Ok(1));
        assert_eq!(err.alt_result(ok2), Ok(2));
    }

    #[test]
    fn test_result_ok_alt() {
        let ok: Result<i32, &str> = Ok(42);
        let err: Result<i32, &str> = Err("error");

        assert_eq!(ok.ok_alt(), Some(42));
        assert_eq!(err.ok_alt(), None);
    }
}
