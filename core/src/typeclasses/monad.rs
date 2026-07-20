//! Monad type class - sequential computation in a context.
//!
//! A `Monad` is an `Applicatio` that also supports sequential chaining of
//! computations where each step can depend on the result of the previous step.
//!
//! # Laws
//!
//! 1. **Left Identity**: `pure(a).flat_map(f) == f(a)`
//! 2. **Right Identity**: `m.flat_map(pure) == m`
//! 3. **Associativity**: `m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))`
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::Monad;
//!
//! let result = Some(5)
//!     .flat_map(|x| Some(x * 2))
//!     .flat_map(|x| Some(x + 1));
//! assert_eq!(result, Some(11));
//!
//! let safe_div = |x: i32, y: i32| -> Option<i32> {
//!     if y == 0 { None } else { Some(x / y) }
//! };
//! let result = Some(10).flat_map(|x| safe_div(x, 2));
//! assert_eq!(result, Some(5));
//! ```

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use super::applicative::Applicatio;

/// A Monad is an Applicatio with sequential binding.
///
/// Monad provides `flat_map` (also known as `bind` or `>>=`), which allows
/// chaining computations where each step can depend on the result of the
/// previous step.
///
/// # Laws
///
/// 1. **Left Identity**: `pure(a).flat_map(f) == f(a)`
/// 2. **Right Identity**: `m.flat_map(pure) == m`
/// 3. **Associativity**: `m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))`
pub trait Monad: Applicatio {
    /// Chain a computation that returns a monad.
    ///
    /// Also known as `bind` or `>>=` in Haskell.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::Monad;
    ///
    /// let result = Some(5).flat_map(|x| Some(x * 2));
    /// assert_eq!(result, Some(10));
    /// ```
    fn flat_map<B, F>(self, f: F) -> Self::Target<B>
    where
        F: FnMut(Self::Inner) -> Self::Target<B>;

    /// Alias for `flat_map` using traditional Haskell naming.
    #[inline]
    fn bind<B, F>(self, f: F) -> Self::Target<B>
    where
        Self: Sized,
        F: FnMut(Self::Inner) -> Self::Target<B>,
    {
        self.flat_map(f)
    }

    /// Alias for `flat_map` using Scala naming.
    #[inline]
    fn and_then<B, F>(self, f: F) -> Self::Target<B>
    where
        Self: Sized,
        F: FnMut(Self::Inner) -> Self::Target<B>,
    {
        self.flat_map(f)
    }
}

// Note: The Universalis `flatten` and `join` functions are intentionally omitted
// because expressing their constraints properly requires features not yet
// stable in Rust. Use `.flat_map(|x| x)` directly for specific types instead.

// ============================================================================
// Implementation for Option
// ============================================================================

impl<A> Monad for Option<A> {
    #[inline]
    fn flat_map<B, F>(self, f: F) -> Option<B>
    where
        F: FnMut(A) -> Option<B>,
    {
        self.and_then(f)
    }
}

// ============================================================================
// Implementation for Result
// ============================================================================

impl<A, E> Monad for Result<A, E> {
    #[inline]
    fn flat_map<B, F>(self, f: F) -> Result<B, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.and_then(f)
    }
}

// ============================================================================
// Implementation for Vec (requires alloc)
// ============================================================================

#[cfg(feature = "alloc")]
impl<A: Clone> Monad for Vec<A> {
    #[inline]
    fn flat_map<B, F>(self, f: F) -> Vec<B>
    where
        F: FnMut(A) -> Vec<B>,
    {
        self.into_iter().flat_map(f).collect()
    }
}

// ============================================================================
// Kleisli composition helpers
// ============================================================================

/// Kleisli composition: compose two monadic functions.
///
/// Given `f: A -> M<B>` and `g: B -> M<C>`, returns a function `A -> M<C>`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::kleisli;
///
/// let f = |x: i32| Some(x * 2);
/// let g = |x: i32| Some(x + 1);
/// let mut fg = kleisli(f, g);
/// assert_eq!(fg(5), Some(11)); // (5 * 2) + 1 = 11
/// ```
#[inline]
pub fn kleisli<A, B, C, F, G, M>(mut f: F, mut g: G) -> impl FnMut(A) -> M::Target<C>
where
    F: FnMut(A) -> M,
    G: FnMut(B) -> M::Target<C>,
    M: Monad<Inner = B>,
{
    move |a| f(a).flat_map(&mut g)
}

/// Lift a pure function into a monadic function.
///
/// Given `f: A -> B`, returns a function `A -> M<B>`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::{lift_m, Applicatio};
///
/// let f = |x: i32| x * 2;
/// let mut lifted = lift_m::<Option<i32>, _, _>(f);
/// assert_eq!(lifted(5), Some(10));
/// ```
#[inline]
pub fn lift_m<M, A, B>(mut f: impl FnMut(A) -> B) -> impl FnMut(A) -> M
where
    M: Applicatio<Inner = B>,
{
    move |a| M::pure(f(a))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typeclasses::applicative::Applicatio;

    #[test]
    fn test_option_flat_map() {
        let result = Some(5).flat_map(|x| Some(x * 2));
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_option_flat_map_none() {
        let result = Some(5).flat_map(|_| None::<i32>);
        assert_eq!(result, None);
    }

    #[test]
    fn test_option_flat_map_from_none() {
        let result: Option<i32> = None.flat_map(|x: i32| Some(x * 2));
        assert_eq!(result, None);
    }

    #[test]
    fn test_option_chaining() {
        let result = Some(10).flat_map(|x| Some(x / 2)).flat_map(|x| Some(x + 1));
        assert_eq!(result, Some(6));
    }

    #[test]
    fn test_result_flat_map() {
        let result: Result<i32, ()> = Ok(5).flat_map(|x| Ok(x * 2));
        assert_eq!(result, Ok(10));
    }

    #[test]
    fn test_result_flat_map_err() {
        let result: Result<i32, &str> = Ok(5).flat_map(|_| Err("error"));
        assert_eq!(result, Err("error"));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_flat_map() {
        let result = alloc::vec![1, 2, 3].flat_map(|x| alloc::vec![x, x * 10]);
        assert_eq!(result, alloc::vec![1, 10, 2, 20, 3, 30]);
    }

    #[test]
    fn test_bind_alias() {
        let result = Some(5).bind(|x| Some(x * 2));
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_and_then_alias() {
        let result = Some(5).map(|x| x * 2);
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_flatten_via_flat_map() {
        // Use flat_map(|x| x) to flatten nested Option
        let nested = Some(Some(42));
        let result = nested.flat_map(|x| x);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_flatten_none_outer() {
        let nested: Option<Option<i32>> = None;
        let result = nested.flat_map(|x| x);
        assert_eq!(result, None);
    }

    #[test]
    fn test_flatten_none_inner() {
        let nested = Some(None::<i32>);
        let result = nested.flat_map(|x| x);
        assert_eq!(result, None);
    }

    #[test]
    fn test_kleisli_composition() {
        let f = |x: i32| Some(x * 2);
        let g = |x: i32| Some(x + 1);
        let mut fg = kleisli::<i32, i32, i32, _, _, Option<i32>>(f, g);
        assert_eq!(fg(5), Some(11));
    }

    // ========================================================================
    // Law tests
    // ========================================================================

    #[test]
    fn test_left_identity_law() {
        // pure(a).flat_map(f) == f(a)
        let a = 5;
        let f = |x: i32| Some(x * 2);

        let left = Option::pure(a).flat_map(f);
        let right = f(a);
        assert_eq!(left, right);
    }

    #[test]
    fn test_right_identity_law() {
        // m.flat_map(pure) == m
        let m = Some(42);

        let left = m.flat_map(Some);
        let right = m;
        assert_eq!(left, right);
    }

    #[test]
    fn test_associativity_law() {
        // m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
        let m = Some(5);
        let f = |x: i32| Some(x * 2);
        let g = |x: i32| Some(x + 1);

        let left = m.flat_map(f).flat_map(g);
        let right = m.flat_map(|x| f(x).flat_map(g));
        assert_eq!(left, right);
    }
}
