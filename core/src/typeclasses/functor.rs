//! Functor type class - types that can be mapped over.
//!
//! A `Functor` is a type constructor that supports mapping a function over its contents
//! while preserving the structure.
//!
//! # Laws
//!
//! 1. **Identity**: `fa.map(|x| x) == fa`
//! 2. **Composition**: `fa.map(f).map(g) == fa.map(|x| g(f(x)))`
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::Functor;
//!
//! let opt = Some(5);
//! let doubled = opt.map(|x| x * 2);
//! assert_eq!(doubled, Some(10));
//!
//! let vec = vec![1, 2, 3];
//! let squared: Vec<i32> = vec.map(|x| x * x);
//! assert_eq!(squared, vec![1, 4, 9]);
//! ```

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A Functor is a type constructor that can be mapped over.
///
/// Functor provides a way to apply a function to values inside a context
/// without leaving that context.
///
/// # Laws
///
/// 1. **Identity**: `fa.map(|x| x) == fa`
/// 2. **Composition**: `fa.map(f).map(g) == fa.map(|x| g(f(x)))`
pub trait Functor {
    /// The inner value type.
    type Inner;

    /// The target type after mapping (type constructor applied to a new type).
    type Target<T>;

    /// Maps a function over the inner value(s).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::Functor;
    ///
    /// let opt = Some(5);
    /// let doubled = opt.map(|x| x * 2);
    /// assert_eq!(doubled, Some(10));
    /// ```
    fn map<B, F>(self, f: F) -> Self::Target<B>
    where
        F: FnMut(Self::Inner) -> B;

    /// Alias for `map` using more traditional FP naming.
    #[inline]
    fn fmap<B, F>(self, f: F) -> Self::Target<B>
    where
        Self: Sized,
        F: FnMut(Self::Inner) -> B,
    {
        self.map(f)
    }

    /// Replace all values with a constant.
    ///
    /// Equivalent to `self.map(|_| b)`.
    #[inline]
    fn map_const<B>(self, b: B) -> Self::Target<B>
    where
        Self: Sized,
        B: Clone,
    {
        self.map(|_| b.clone())
    }

    /// Map and then discard the result, keeping the original structure.
    ///
    /// Equivalent to `self.map(|_| ())`.
    #[inline]
    fn void(self) -> Self::Target<()>
    where
        Self: Sized,
    {
        self.map(|_| ())
    }
}

// ============================================================================
// Implementation for Option
// ============================================================================

impl<A> Functor for Option<A> {
    type Inner = A;
    type Target<T> = Option<T>;

    #[inline]
    fn map<B, F>(self, f: F) -> Option<B>
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

// ============================================================================
// Implementation for Result
// ============================================================================

impl<A, E> Functor for Result<A, E> {
    type Inner = A;
    type Target<T> = Result<T, E>;

    #[inline]
    fn map<B, F>(self, f: F) -> Result<B, E>
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

// ============================================================================
// Implementation for Vec (requires alloc)
// ============================================================================

#[cfg(feature = "alloc")]
impl<A> Functor for Vec<A> {
    type Inner = A;
    type Target<T> = Vec<T>;

    #[inline]
    fn map<B, F>(self, f: F) -> Vec<B>
    where
        F: FnMut(A) -> B,
    {
        self.into_iter().map(f).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_map() {
        let opt = Some(5);
        let result = opt.map(|x| x * 2);
        assert_eq!(result, Some(10));

        let none: Option<i32> = None;
        let result = none.map(|x| x * 2);
        assert_eq!(result, None);
    }

    #[test]
    fn test_result_map() {
        let ok: Result<i32, &str> = Ok(5);
        let result = ok.map(|x| x * 2);
        assert_eq!(result, Ok(10));

        let err: Result<i32, &str> = Err("error");
        let result = err.map(|x| x * 2);
        assert_eq!(result, Err("error"));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_map() {
        let vec = alloc::vec![1, 2, 3];
        let result: Vec<i32> = vec.map(|x| x * 2);
        assert_eq!(result, alloc::vec![2, 4, 6]);
    }

    #[test]
    fn test_fmap_alias() {
        let opt = Some(5);
        let result = opt.fmap(|x| x * 2);
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_map_const() {
        let opt = Some(5);
        let result = opt.map_const("constant");
        assert_eq!(result, Some("constant"));
    }

    #[test]
    fn test_void() {
        let opt = Some(5);
        let result = opt.void();
        assert_eq!(result, Some(()));
    }

    #[test]
    #[allow(clippy::map_identity)] // map(id) == id is the law under test
    fn test_identity_law() {
        let opt = Some(42);
        let result = opt.map(|x| x);
        assert_eq!(result, opt);
    }

    #[test]
    fn test_composition_law() {
        let opt = Some(5);
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        let left = opt.map(f).map(g);
        let right = opt.map(|x| g(f(x)));
        assert_eq!(left, right);
    }
}
