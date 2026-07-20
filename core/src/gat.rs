//! GAT-based Type Classes for Higher Kinded Types simulation.
//!
//! Rust does not currently support true Higher Kinded Types (HKT).
//! However, with Generalized Associated Types (GATs), we can simulate them effectively.
//!
//! This module provides the `Functor`, `Apply`, `Applicative`, and `Monad` traits.
//!
//! # Supported Types
//!
//! - `Option<T>` - Optional values
//! - `Result<T, E>` - Error handling
//! - `Vec<T>` - Collections (requires `alloc` feature)
//!
//! # Example
//!
//! ```
//! use ordofp_core::gat::{Functor, Monad, Applicative};
//!
//! // Option as Monad
//! let result = Some(5)
//!     .map(|x| x * 2)
//!     .flat_map(|x| if x > 5 { Some(x) } else { None });
//! assert_eq!(result, Some(10));
//!
//! // Result as Monad
//! let ok_result: Result<i32, &str> = Ok(10);
//! let chained = ok_result.flat_map(|x| Ok(x + 1));
//! assert_eq!(chained, Ok(11));
//! ```

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A Functor is a type constructor that supports mapping a function over its inner value.
pub trait Functor {
    /// The inner value type
    type Inner;

    /// The target type after mapping (the "Kind" applied to a new type)
    type Target<T>;

    /// Maps a function over the inner value.
    fn map<B, F>(self, f: F) -> Self::Target<B>
    where
        F: FnMut(Self::Inner) -> B;
}

/// Apply extends Functor with the ability to apply a wrapped function to a wrapped value.
pub trait Apply: Functor {
    /// Applies a wrapped function to this value.
    ///
    /// The function is wrapped in the same context as the value.
    fn apply<B, F>(self, f: Self::Target<F>) -> Self::Target<B>
    where
        F: FnMut(Self::Inner) -> B;
}

/// Applicative extends Apply with the ability to wrap a pure value.
pub trait Applicative: Apply {
    /// Wraps a value into the context.
    ///
    /// Note: This is `pure` or `return`.
    /// Since traits in Rust are implemented on the concrete type (e.g., `Option<A>`),
    /// this function acts as a factory for the context `Self::Target<T>`.
    fn pure_target<T>(t: T) -> Self::Target<T>;
}

/// Monad extends Applicative with the ability to flatten nested contexts (chaining).
pub trait Monad: Applicative {
    /// Chains an operation that returns a value in the context.
    ///
    /// Also known as `bind` or `>>=`.
    fn flat_map<B, F>(self, f: F) -> Self::Target<B>
    where
        F: FnMut(Self::Inner) -> Self::Target<B>;
}

// Implementations for Option

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

impl<A> Apply for Option<A> {
    #[inline]
    fn apply<B, F>(self, f: Option<F>) -> Option<B>
    where
        F: FnMut(A) -> B,
    {
        match (self, f) {
            (Some(a), Some(mut func)) => Some(func(a)),
            _ => None,
        }
    }
}

impl<A> Applicative for Option<A> {
    #[inline]
    fn pure_target<T>(t: T) -> Option<T> {
        Some(t)
    }
}

impl<A> Monad for Option<A> {
    #[inline]
    fn flat_map<B, F>(self, f: F) -> Option<B>
    where
        F: FnMut(A) -> Option<B>,
    {
        self.and_then(f)
    }
}

// Implementations for Result

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

impl<A, E> Apply for Result<A, E> {
    #[inline]
    fn apply<B, F>(self, f: Result<F, E>) -> Result<B, E>
    where
        F: FnMut(A) -> B,
    {
        match (self, f) {
            (Ok(a), Ok(mut func)) => Ok(func(a)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

impl<A, E> Applicative for Result<A, E> {
    #[inline]
    fn pure_target<T>(t: T) -> Result<T, E> {
        Ok(t)
    }
}

impl<A, E> Monad for Result<A, E> {
    #[inline]
    fn flat_map<B, F>(self, f: F) -> Result<B, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.and_then(f)
    }
}

// Implementations for Vec (requires alloc)

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

#[cfg(feature = "alloc")]
impl<A: Clone> Apply for Vec<A> {
    #[inline]
    fn apply<B, F>(self, f: Vec<F>) -> Vec<B>
    where
        F: FnMut(A) -> B,
    {
        // Cartesian product: apply each function to each value
        let mut result = Vec::with_capacity(self.len() * f.len());
        for mut func in f {
            for a in self.iter().cloned() {
                result.push(func(a));
            }
        }
        result
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> Applicative for Vec<A> {
    #[inline]
    fn pure_target<T>(t: T) -> Vec<T> {
        alloc::vec![t]
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_monad() {
        let val = Some(5);
        let res = val.map(|x| x * 2).flat_map(|x| Some(x + 1));
        assert_eq!(res, Some(11));

        let none: Option<i32> = None;
        let res_none = none.map(|x| x * 2).flat_map(|x| Some(x + 1));
        assert_eq!(res_none, None);
    }

    #[test]
    fn test_result_monad() {
        let val: Result<i32, &str> = Ok(5);
        let res = val.map(|x| x * 2).flat_map(|x| Ok(x + 1));
        assert_eq!(res, Ok(11));

        let err: Result<i32, &str> = Err("oops");
        let res_err = err.map(|x| x * 2).flat_map(|x| Ok(x + 1));
        assert_eq!(res_err, Err("oops"));
    }

    #[test]
    fn test_applicative() {
        let f = Some(|x: i32| x * 2);
        let val = Some(10);
        let res = val.apply(f);
        assert_eq!(res, Some(20));

        let val2: Option<i32> = <Option<i32>>::pure_target(42);
        assert_eq!(val2, Some(42));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_vec_functor() {
        let vec = alloc::vec![1, 2, 3];
        let mapped: Vec<i32> = vec.map(|x| x * 2);
        assert_eq!(mapped, alloc::vec![2, 4, 6]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_vec_applicative() {
        let vec = alloc::vec![1, 2];
        let funcs: Vec<fn(i32) -> i32> = alloc::vec![|x| x + 1, |x| x * 10];
        let result = vec.apply(funcs);
        // Cartesian product: [f1(1), f1(2), f2(1), f2(2)] = [2, 3, 10, 20]
        assert_eq!(result, alloc::vec![2, 3, 10, 20]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_vec_monad() {
        let vec = alloc::vec![1, 2, 3];
        let result = vec.flat_map(|x| alloc::vec![x, x * 10]);
        assert_eq!(result, alloc::vec![1, 10, 2, 20, 3, 30]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_vec_pure() {
        let pure_vec: Vec<i32> = <Vec<i32>>::pure_target(42);
        assert_eq!(pure_vec, alloc::vec![42]);
    }

    // Monad law tests
    #[test]
    fn test_option_monad_left_identity() {
        // Left identity: pure(a).flat_map(f) == f(a)
        let a = 5;
        let f = |x: i32| Some(x * 2);

        let left = <Option<i32>>::pure_target(a).flat_map(f);
        let right = f(a);
        assert_eq!(left, right);
    }

    #[test]
    fn test_option_monad_right_identity() {
        // Right identity: m.flat_map(pure) == m
        let m = Some(42);
        let result = m.flat_map(<Option<i32>>::pure_target);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_option_monad_associativity() {
        // Associativity: m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
        let m = Some(5);
        let f = |x: i32| Some(x + 1);
        let g = |x: i32| Some(x * 2);

        let left = m.flat_map(f).flat_map(g);
        let right = Some(5).flat_map(|x| f(x).flat_map(g));
        assert_eq!(left, right);
    }

    #[test]
    fn test_result_monad_laws() {
        // Left identity
        let a = 5;
        let f = |x: i32| Ok::<i32, &str>(x * 2);
        let left: Result<i32, &str> = <Result<i32, &str>>::pure_target(a).flat_map(f);
        let right = f(a);
        assert_eq!(left, right);

        // Right identity
        let m: Result<i32, &str> = Ok(42);
        let result = m.flat_map(<Result<i32, &str>>::pure_target);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_option_apply_none_propagation() {
        // The wildcard arm `_ => None` in Option's Apply impl covers two cases that
        // are not exercised by the happy-path test: a None function container and a
        // None value container. Both must short-circuit to None without panicking.
        let val = Some(42i32);
        let no_func: Option<fn(i32) -> i32> = None;
        assert_eq!(
            val.apply(no_func),
            None,
            "apply with a None function container must produce None"
        );

        let val_none: Option<i32> = None;
        let func = Some(|x: i32| x * 2);
        assert_eq!(
            val_none.apply(func),
            None,
            "apply with a None value container must produce None"
        );
    }
}
