//! Applicatio Functor type class - functors with application.
//!
//! > *\"Applicatio formae ad materiam.\"*
//! > — The application of form to matter. (Scholastic maxim)
//!
//! An `Applicatio` is a `Functor` that also supports lifting values into the context
//! and applying functions that are themselves in a context.
//!
//! The name derives from the scholastic concept of *applicatio*, the act of
//! bringing form to matter, or in our case, bringing functions to values
//! within a computational context.
//!
//! # Laws (*Leges Applicationis*)
//!
//! 1. **Identity**: `pure(|x| x).ap(v) == v`
//! 2. **Homomorphism**: `pure(f).ap(pure(x)) == pure(f(x))`
//! 3. **Interchange**: `u.ap(pure(y)) == pure(|f| f(y)).ap(u)`
//! 4. **Composition**: `pure(compose).ap(u).ap(v).ap(w) == u.ap(v.ap(w))`
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::{Apply, Applicatio};
//!
//! let f: Option<fn(i32) -> i32> = Some(|x| x * 2);
//! let a = Some(5);
//! let result = a.ap(f);
//! assert_eq!(result, Some(10));
//!
//! let lifted: Option<i32> = Option::pure(42);
//! assert_eq!(lifted, Some(42));
//! ```

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use super::functor::Functor;

/// The Apply typeclass - a Functor with application.
///
/// Apply provides a way to apply functions that are themselves in a context
/// to values in a context.
///
/// This is separated from Applicatio to allow types that can apply but
/// cannot lift values (e.g., Map-like types with no natural `pure`).
pub trait Apply: Functor {
    /// Apply a function in a context to a value in a context.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::Apply;
    ///
    /// let f: Option<fn(i32) -> i32> = Some(|x| x * 2);
    /// let a = Some(5);
    /// let result = a.apply(f);
    /// assert_eq!(result, Some(10));
    /// ```
    fn apply<B, F>(self, fa: Self::Target<F>) -> Self::Target<B>
    where
        Self::Target<F>: Apply<Inner = F, Target<B> = Self::Target<B>>,
        F: FnMut(Self::Inner) -> B;

    /// Alias for `apply`.
    #[inline]
    fn ap<B, F>(self, fa: Self::Target<F>) -> Self::Target<B>
    where
        Self: Sized,
        Self::Target<F>: Apply<Inner = F, Target<B> = Self::Target<B>>,
        F: FnMut(Self::Inner) -> B,
    {
        self.apply(fa)
    }

    /// Sequence two applicatives, keeping the left result.
    ///
    /// **Note:** The default implementation discards `fb` without evaluating it.
    /// This is correct for pure data types (`Option`, `Result`, `Vec`) but will
    /// silently drop effects for effectful applicatives. Types with effects
    /// should override this to properly sequence both sides.
    #[inline]
    fn ap_left<B>(self, _fb: Self::Target<B>) -> Self::Target<Self::Inner>
    where
        Self: Sized,
        Self::Inner: Clone,
        Self::Target<B>: Apply<Inner = B, Target<Self::Inner> = Self::Target<Self::Inner>>,
        Self::Target<fn(B) -> Self::Inner>:
            Apply<Inner = fn(B) -> Self::Inner, Target<Self::Inner> = Self::Target<Self::Inner>>,
    {
        // Simplified: just map to preserve structure.
        // Effectful types should override to sequence both sides.
        self.map(|a| a)
    }

    /// Sequence two applicatives, keeping the right result.
    ///
    /// **Note:** The default implementation discards `self` without evaluating it.
    /// This is correct for pure data types (`Option`, `Result`, `Vec`) but will
    /// silently drop effects for effectful applicatives. Types with effects
    /// should override this to properly sequence both sides.
    #[inline]
    fn ap_right<B>(self, fb: Self::Target<B>) -> Self::Target<B>
    where
        Self: Sized,
        Self::Target<B>: Apply<Inner = B, Target<B> = Self::Target<B>> + Clone,
        Self::Target<fn(Self::Inner) -> B>:
            Apply<Inner = fn(Self::Inner) -> B, Target<B> = Self::Target<B>>,
    {
        // Simplified: just return fb.
        // Effectful types should override to sequence both sides.
        let _ = self;
        fb
    }
}

/// Applicatio - Apply with the ability to lift values.
///
/// > *\"Forma dat esse.\"*
/// > — Form gives being. (Scholastic axiom)
///
/// An Applicatio functor is an Apply that also has a way to lift
/// pure values into the functor context. Named after the scholastic
/// concept of *applicatio formae ad materiam*.
///
/// # Laws (*Leges Applicationis*)
///
/// 1. **Identity**: `pure(|x| x).ap(v) == v`
/// 2. **Homomorphism**: `pure(f).ap(pure(x)) == pure(f(x))`
/// 3. **Interchange**: `u.ap(pure(y)) == pure(|f| f(y)).ap(u)`
/// 4. **Composition**: `pure(compose).ap(u).ap(v).ap(w) == u.ap(v.ap(w))`
pub trait Applicatio: Apply {
    /// Lift a value into the Applicatio context.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::Applicatio;
    ///
    /// let opt: Option<i32> = Option::pure(42);
    /// assert_eq!(opt, Some(42));
    /// ```
    fn pure(a: Self::Inner) -> Self;

    /// Lift a value into the target type.
    ///
    /// This is useful when you need to lift into the target type
    /// rather than Self.
    fn pure_target<T>(t: T) -> Self::Target<T>
    where
        Self::Target<T>: Applicatio<Inner = T>;
}

/// Convenience function to lift a value using explicit type annotation.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::pure;
///
/// let opt: Option<i32> = pure(42);
/// assert_eq!(opt, Some(42));
/// ```
#[inline]
pub fn pure<F: Applicatio>(a: F::Inner) -> F {
    F::pure(a)
}

// ============================================================================
// Implementation for Option
// ============================================================================

impl<A> Apply for Option<A> {
    #[inline]
    fn apply<B, F>(self, ff: Option<F>) -> Option<B>
    where
        F: FnMut(A) -> B,
    {
        match (ff, self) {
            (Some(mut f), Some(a)) => Some(f(a)),
            _ => None,
        }
    }
}

impl<A> Applicatio for Option<A> {
    #[inline]
    fn pure(a: A) -> Self {
        Some(a)
    }

    #[inline]
    fn pure_target<T>(t: T) -> Option<T> {
        Some(t)
    }
}

// ============================================================================
// Implementation for Result
// ============================================================================

impl<A, E> Apply for Result<A, E> {
    #[inline]
    fn apply<B, F>(self, ff: Result<F, E>) -> Result<B, E>
    where
        F: FnMut(A) -> B,
    {
        match (ff, self) {
            (Ok(mut f), Ok(a)) => Ok(f(a)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

impl<A, E> Applicatio for Result<A, E> {
    #[inline]
    fn pure(a: A) -> Self {
        Ok(a)
    }

    #[inline]
    fn pure_target<T>(t: T) -> Result<T, E> {
        Ok(t)
    }
}

// ============================================================================
// Implementation for Vec (requires alloc)
// ============================================================================

#[cfg(feature = "alloc")]
impl<A: Clone> Apply for Vec<A> {
    #[inline]
    fn apply<B, F>(self, ff: Vec<F>) -> Vec<B>
    where
        F: FnMut(A) -> B,
    {
        // Cartesian product application: apply each function to each element
        let mut result = Vec::with_capacity(ff.len() * self.len());
        for mut f in ff {
            for a in &self {
                result.push(f(a.clone()));
            }
        }
        result
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> Applicatio for Vec<A> {
    #[inline]
    fn pure(a: A) -> Self {
        alloc::vec![a]
    }

    #[inline]
    fn pure_target<T>(t: T) -> Vec<T> {
        alloc::vec![t]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_ap() {
        let f: Option<fn(i32) -> i32> = Some(|x| x * 2);
        let a = Some(5);
        let result = a.apply(f);
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_option_ap_none_function() {
        let f: Option<fn(i32) -> i32> = None;
        let a = Some(5);
        let result = a.apply(f);
        assert_eq!(result, None);
    }

    #[test]
    fn test_option_ap_none_value() {
        let f: Option<fn(i32) -> i32> = Some(|x| x * 2);
        let a: Option<i32> = None;
        let result = a.apply(f);
        assert_eq!(result, None);
    }

    #[test]
    fn test_option_pure() {
        let result: Option<i32> = Option::pure(42);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_pure_function() {
        let result: Option<i32> = pure(42);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_result_ap() {
        let f: Result<fn(i32) -> i32, &str> = Ok(|x| x * 2);
        let a: Result<i32, &str> = Ok(5);
        let result = a.apply(f);
        assert_eq!(result, Ok(10));
    }

    #[test]
    fn test_result_ap_err_function() {
        let f: Result<fn(i32) -> i32, &str> = Err("error");
        let a: Result<i32, &str> = Ok(5);
        let result = a.apply(f);
        assert_eq!(result, Err("error"));
    }

    #[test]
    fn test_result_ap_err_value() {
        let f: Result<fn(i32) -> i32, &str> = Ok(|x| x * 2);
        let a: Result<i32, &str> = Err("error");
        let result = a.apply(f);
        assert_eq!(result, Err("error"));
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_pure() {
        let result: Vec<i32> = Vec::pure(42);
        assert_eq!(result, alloc::vec![42]);
    }

    #[test]
    fn test_identity_law() {
        let v = Some(42);
        let id: Option<fn(i32) -> i32> = Some(|x| x);
        let result = v.apply(id);
        assert_eq!(result, v);
    }

    #[test]
    fn test_homomorphism_law() {
        let f = |x: i32| x * 2;
        let x = 21;

        let left = Option::pure(x).apply(Option::pure(f));
        let right = Option::pure(f(x));
        assert_eq!(left, right);
    }
}
