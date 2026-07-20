//! `EitherT` Monad Transformer
//!
//! `EitherT` adds error handling with a specific error type to any base monad.
//! It wraps a computation that returns `M<Result<A, E>>` and provides a monadic
//! interface that handles the `Result` layer automatically.
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # fn main() {
//! use ordofp_core::transformers::EitherT;
//!
//! // EitherT over Option - combines error handling with optionality
//! let computation: EitherT<Option<Result<i32, &str>>> = EitherT::right(42);
//! let result = computation
//!     .map(|x| x * 2)
//!     .flat_map(|x| if x > 50 { EitherT::right(x) } else { EitherT::left("too small") });
//! assert_eq!(result.run(), Some(Ok(84)));
//! # }
//! # #[cfg(not(feature = "alloc"))]
//! # fn main() {}
//! ```

use super::MonadTransformer;

/// The `EitherT` monad transformer adds error handling to any base monad `M`.
///
/// `EitherT<M>` wraps `M<Result<A, E>>` and provides a unified interface for working
/// with fallible computations inside another monadic context.
///
/// # Type Parameters
///
/// - `M`: The base monad wrapping `Result<A, E>` (e.g., `Option<Result<A, E>>`)
///
/// # Terminology
///
/// - `right`: The success case (like `Ok`)
/// - `left`: The error case (like `Err`)
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # fn main() {
/// use ordofp_core::transformers::EitherT;
///
/// // Create a successful EitherT
/// let ok: EitherT<Option<Result<i32, &str>>> = EitherT::right(42);
/// assert_eq!(ok.run(), Some(Ok(42)));
///
/// // Create a failed EitherT
/// let err: EitherT<Option<Result<i32, &str>>> = EitherT::left("error");
/// assert_eq!(err.run(), Some(Err("error")));
/// # }
/// # #[cfg(not(feature = "alloc"))]
/// # fn main() {}
/// ```
///
/// ## Chaining Computations
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # fn main() {
/// use ordofp_core::transformers::EitherT;
///
/// fn safe_div(a: i32, b: i32) -> EitherT<Option<Result<i32, &'static str>>> {
///     if b == 0 {
///         EitherT::left("division by zero")
///     } else {
///         EitherT::right(a / b)
///     }
/// }
///
/// let result = EitherT::<Option<Result<i32, &str>>>::right(100)
///     .flat_map(|x| safe_div(x, 2))
///     .flat_map(|x| safe_div(x, 5));
/// assert_eq!(result.run(), Some(Ok(10)));
///
/// // Division by zero returns an error
/// let result2 = EitherT::<Option<Result<i32, &str>>>::right(100)
///     .flat_map(|x| safe_div(x, 0));
/// assert_eq!(result2.run(), Some(Err("division by zero")));
/// # }
/// # #[cfg(not(feature = "alloc"))]
/// # fn main() {}
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EitherT<M> {
    /// The wrapped computation
    inner: M,
}

impl<M> EitherT<M> {
    /// Creates a new `EitherT` from a wrapped computation.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let inner: Option<Result<i32, &str>> = Some(Ok(42));
    /// let either_t = EitherT::new(inner);
    /// assert_eq!(either_t.run(), Some(Ok(42)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn new(inner: M) -> Self {
        EitherT { inner }
    }

    /// Runs the transformer, extracting the inner computation.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let either_t: EitherT<Option<Result<i32, &str>>> = EitherT::right(42);
    /// let result: Option<Result<i32, &str>> = either_t.run();
    /// assert_eq!(result, Some(Ok(42)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn run(self) -> M {
        self.inner
    }

    /// Returns a reference to the inner computation.
    #[inline]
    pub fn inner_ref(&self) -> &M {
        &self.inner
    }
}

// ============================================================================
// EitherT over Option
// ============================================================================

impl<A, E> EitherT<Option<Result<A, E>>> {
    /// Creates an `EitherT` containing a success value over `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let ok: EitherT<Option<Result<i32, &str>>> = EitherT::right(42);
    /// assert_eq!(ok.run(), Some(Ok(42)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn right(value: A) -> Self {
        EitherT::new(Some(Ok(value)))
    }

    /// Creates an `EitherT` containing an error value over `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let err: EitherT<Option<Result<i32, &str>>> = EitherT::left("error");
    /// assert_eq!(err.run(), Some(Err("error")));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn left(error: E) -> Self {
        EitherT::new(Some(Err(error)))
    }

    /// Creates an `EitherT` representing an absent computation (None).
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let absent: EitherT<Option<Result<i32, &str>>> = EitherT::absent();
    /// assert_eq!(absent.run(), None);
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn absent() -> Self {
        EitherT::new(None)
    }

    /// Lifts an `Option` value into `EitherT`, treating `Some` as success.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let some: Option<i32> = Some(42);
    /// let lifted: EitherT<Option<Result<i32, &str>>> = EitherT::lift_m(some);
    /// assert_eq!(lifted.run(), Some(Ok(42)));
    ///
    /// let none: Option<i32> = None;
    /// let lifted_none: EitherT<Option<Result<i32, &str>>> = EitherT::lift_m(none);
    /// assert_eq!(lifted_none.run(), None);
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn lift_m(option: Option<A>) -> Self {
        EitherT::new(option.map(Ok))
    }

    /// Maps a function over the success value.
    ///
    /// If the computation is `Some(Ok(a))`, applies `f` to `a`.
    /// If the computation is `Some(Err(e))` or `None`, propagates unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let ok: EitherT<Option<Result<i32, &str>>> = EitherT::right(21);
    /// let doubled = ok.map(|x| x * 2);
    /// assert_eq!(doubled.run(), Some(Ok(42)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn map<B, F>(self, f: F) -> EitherT<Option<Result<B, E>>>
    where
        F: FnOnce(A) -> B,
    {
        EitherT::new(self.inner.map(|res| res.map(f)))
    }

    /// Maps a function over the error value.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let err: EitherT<Option<Result<i32, i32>>> = EitherT::left(1);
    /// let mapped = err.map_left(|e| e * 2);
    /// assert_eq!(mapped.run(), Some(Err(2)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn map_left<E2, F>(self, f: F) -> EitherT<Option<Result<A, E2>>>
    where
        F: FnOnce(E) -> E2,
    {
        EitherT::new(self.inner.map(|res| res.map_err(f)))
    }

    /// Chains a computation that returns an `EitherT`.
    ///
    /// Also known as `bind` or `>>=`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let ok: EitherT<Option<Result<i32, &str>>> = EitherT::right(10);
    /// let result = ok.flat_map(|x| {
    ///     if x > 5 { EitherT::right(x * 2) }
    ///     else { EitherT::left("too small") }
    /// });
    /// assert_eq!(result.run(), Some(Ok(20)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> EitherT<Option<Result<B, E>>>
    where
        F: FnOnce(A) -> EitherT<Option<Result<B, E>>>,
    {
        EitherT::new(match self.inner {
            Some(Ok(a)) => f(a).inner,
            Some(Err(e)) => Some(Err(e)),
            None => None,
        })
    }

    /// Applies a wrapped function to this value.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let val: EitherT<Option<Result<i32, &str>>> = EitherT::right(21);
    /// let func: EitherT<Option<Result<fn(i32) -> i32, &str>>> =
    ///     EitherT::right(|x: i32| x * 2);
    /// let result = val.apply(func);
    /// assert_eq!(result.run(), Some(Ok(42)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn apply<B, F>(self, f: EitherT<Option<Result<F, E>>>) -> EitherT<Option<Result<B, E>>>
    where
        F: FnOnce(A) -> B,
    {
        EitherT::new(match (self.inner, f.inner) {
            (Some(Ok(a)), Some(Ok(func))) => Some(Ok(func(a))),
            (Some(Err(e)), _) | (_, Some(Err(e))) => Some(Err(e)),
            (None, _) | (_, None) => None,
        })
    }

    /// Combines two `EitherT` values using a function.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let a: EitherT<Option<Result<i32, &str>>> = EitherT::right(10);
    /// let b: EitherT<Option<Result<i32, &str>>> = EitherT::right(20);
    /// let combined = a.map2(b, |x, y| x + y);
    /// assert_eq!(combined.run(), Some(Ok(30)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn map2<B, C, F>(
        self,
        other: EitherT<Option<Result<B, E>>>,
        f: F,
    ) -> EitherT<Option<Result<C, E>>>
    where
        F: FnOnce(A, B) -> C,
    {
        EitherT::new(match (self.inner, other.inner) {
            (Some(Ok(a)), Some(Ok(b))) => Some(Ok(f(a, b))),
            (Some(Err(e)), _) | (_, Some(Err(e))) => Some(Err(e)),
            (None, _) | (_, None) => None,
        })
    }

    /// Handles the error case, potentially recovering.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::EitherT;
    ///
    /// let err: EitherT<Option<Result<i32, &str>>> = EitherT::left("error");
    /// let recovered = err.handle_error(|_| EitherT::right(0));
    /// assert_eq!(recovered.run(), Some(Ok(0)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn handle_error<F>(self, f: F) -> Self
    where
        F: FnOnce(E) -> Self,
    {
        EitherT::new(match self.inner {
            Some(Ok(a)) => Some(Ok(a)),
            Some(Err(e)) => f(e).inner,
            None => None,
        })
    }

    /// Converts the error type using a function.
    ///
    /// Alias for `map_left`.
    #[inline]
    pub fn with_error<E2, F>(self, f: F) -> EitherT<Option<Result<A, E2>>>
    where
        F: FnOnce(E) -> E2,
    {
        self.map_left(f)
    }

    /// Checks if the computation is a success.
    #[inline]
    pub fn is_right(&self) -> bool {
        matches!(&self.inner, Some(Ok(_)))
    }

    /// Checks if the computation is an error.
    #[inline]
    pub fn is_left(&self) -> bool {
        matches!(&self.inner, Some(Err(_)))
    }

    /// Checks if the computation is absent.
    #[inline]
    pub fn is_absent(&self) -> bool {
        self.inner.is_none()
    }
}

impl<A, E> MonadTransformer for EitherT<Option<Result<A, E>>> {
    type BaseMonad = Option<A>;

    #[inline]
    fn lift(base: Option<A>) -> Self {
        EitherT::lift_m(base)
    }
}

// ============================================================================
// EitherT over Result (Result<Result<A, E1>, E2>)
// ============================================================================

impl<A, E1, E2> EitherT<Result<Result<A, E1>, E2>> {
    /// Creates an `EitherT` containing a success value over `Result`.
    #[inline]
    pub fn right_result(value: A) -> Self {
        EitherT::new(Ok(Ok(value)))
    }

    /// Creates an `EitherT` containing an inner error over `Result`.
    #[inline]
    pub fn left_result(error: E1) -> Self {
        EitherT::new(Ok(Err(error)))
    }

    /// Creates an `EitherT` containing an outer error over `Result`.
    #[inline]
    pub fn outer_err(error: E2) -> Self {
        EitherT::new(Err(error))
    }

    /// Maps a function over the success value.
    #[inline]
    pub fn map_result<B, F>(self, f: F) -> EitherT<Result<Result<B, E1>, E2>>
    where
        F: FnOnce(A) -> B,
    {
        EitherT::new(self.inner.map(|res| res.map(f)))
    }

    /// Chains a computation that returns an `EitherT`.
    #[inline]
    pub fn flat_map_result<B, F>(self, f: F) -> EitherT<Result<Result<B, E1>, E2>>
    where
        F: FnOnce(A) -> EitherT<Result<Result<B, E1>, E2>>,
    {
        EitherT::new(match self.inner {
            Ok(Ok(a)) => f(a).inner,
            Ok(Err(e)) => Ok(Err(e)),
            Err(e) => Err(e),
        })
    }
}

// ============================================================================
// EitherT over Vec
// ============================================================================

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
impl<A, E> EitherT<Vec<Result<A, E>>> {
    /// Creates an `EitherT` containing a success value over `Vec`.
    #[inline]
    pub fn right_vec(value: A) -> Self {
        EitherT::new(alloc::vec![Ok(value)])
    }

    /// Creates an `EitherT` containing an error value over `Vec`.
    #[inline]
    pub fn left_vec(error: E) -> Self {
        EitherT::new(alloc::vec![Err(error)])
    }

    /// Creates an `EitherT` from multiple results.
    #[inline]
    pub fn from_vec(values: Vec<Result<A, E>>) -> Self {
        EitherT::new(values)
    }

    /// Maps a function over all success values.
    #[inline]
    pub fn map_vec<B, F>(self, mut f: F) -> EitherT<Vec<Result<B, E>>>
    where
        F: FnMut(A) -> B,
    {
        EitherT::new(self.inner.into_iter().map(|res| res.map(&mut f)).collect())
    }

    /// Chains a computation that returns an `EitherT`.
    #[inline]
    pub fn flat_map_vec<B, F>(self, mut f: F) -> EitherT<Vec<Result<B, E>>>
    where
        F: FnMut(A) -> EitherT<Vec<Result<B, E>>>,
    {
        let results: Vec<Result<B, E>> = self
            .inner
            .into_iter()
            .flat_map(|res| match res {
                Ok(a) => f(a).inner,
                Err(e) => alloc::vec![Err(e)],
            })
            .collect();
        EitherT::new(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_either_t_right() {
        let ok: EitherT<Option<Result<i32, &str>>> = EitherT::right(42);
        assert_eq!(ok.run(), Some(Ok(42)));
    }

    #[test]
    fn test_either_t_left() {
        let err: EitherT<Option<Result<i32, &str>>> = EitherT::left("error");
        assert_eq!(err.run(), Some(Err("error")));
    }

    #[test]
    fn test_either_t_absent() {
        let absent: EitherT<Option<Result<i32, &str>>> = EitherT::absent();
        assert_eq!(absent.run(), None);
    }

    #[test]
    fn test_either_t_map() {
        let ok: EitherT<Option<Result<i32, &str>>> = EitherT::right(21);
        let doubled = ok.map(|x| x * 2);
        assert_eq!(doubled.run(), Some(Ok(42)));

        let err: EitherT<Option<Result<i32, &str>>> = EitherT::left("error");
        let err_mapped = err.map(|x| x * 2);
        assert_eq!(err_mapped.run(), Some(Err("error")));
    }

    #[test]
    fn test_either_t_flat_map() {
        let ok: EitherT<Option<Result<i32, &str>>> = EitherT::right(10);
        let result = ok.flat_map(|x| {
            if x > 5 {
                EitherT::right(x * 2)
            } else {
                EitherT::left("too small")
            }
        });
        assert_eq!(result.run(), Some(Ok(20)));

        let ok2: EitherT<Option<Result<i32, &str>>> = EitherT::right(3);
        let result2 = ok2.flat_map(|x| {
            if x > 5 {
                EitherT::right(x * 2)
            } else {
                EitherT::left("too small")
            }
        });
        assert_eq!(result2.run(), Some(Err("too small")));
    }

    #[test]
    #[allow(clippy::type_complexity)] // spelled-out nested transformer type is the point
    fn test_either_t_apply() {
        let val: EitherT<Option<Result<i32, &str>>> = EitherT::right(21);
        let func: EitherT<Option<Result<fn(i32) -> i32, &str>>> = EitherT::right(|x: i32| x * 2);
        let result = val.apply(func);
        assert_eq!(result.run(), Some(Ok(42)));
    }

    #[test]
    fn test_either_t_lift_m() {
        let some: Option<i32> = Some(42);
        let lifted: EitherT<Option<Result<i32, &str>>> = EitherT::lift_m(some);
        assert_eq!(lifted.run(), Some(Ok(42)));

        let none: Option<i32> = None;
        let lifted_none: EitherT<Option<Result<i32, &str>>> = EitherT::lift_m(none);
        assert_eq!(lifted_none.run(), None);
    }

    #[test]
    fn test_either_t_handle_error() {
        let err: EitherT<Option<Result<i32, &str>>> = EitherT::left("error");
        let recovered = err.handle_error(|_| EitherT::right(0));
        assert_eq!(recovered.run(), Some(Ok(0)));
    }

    // Monad law tests
    #[test]
    fn test_either_t_left_identity() {
        // pure(a).flat_map(f) == f(a)
        let a = 5;
        let f = |x: i32| EitherT::<Option<Result<i32, &str>>>::right(x * 2);

        let left = EitherT::<Option<Result<i32, &str>>>::right(a).flat_map(f);
        let right = f(a);
        assert_eq!(left.run(), right.run());
    }

    #[test]
    fn test_either_t_right_identity() {
        // m.flat_map(pure) == m
        let m: EitherT<Option<Result<i32, &str>>> = EitherT::right(42);
        let result = m.flat_map(EitherT::right);
        assert_eq!(result.run(), Some(Ok(42)));
    }

    #[test]
    fn test_either_t_associativity() {
        // m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
        let m: EitherT<Option<Result<i32, &str>>> = EitherT::right(5);
        let f = |x: i32| EitherT::<Option<Result<i32, &str>>>::right(x + 1);
        let g = |x: i32| EitherT::<Option<Result<i32, &str>>>::right(x * 2);

        let left = m.flat_map(f).flat_map(g);
        let right = EitherT::<Option<Result<i32, &str>>>::right(5).flat_map(|x| f(x).flat_map(g));
        assert_eq!(left.run(), right.run());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_either_t_vec() {
        let ok: EitherT<Vec<Result<i32, &str>>> = EitherT::right_vec(42);
        assert_eq!(ok.run(), alloc::vec![Ok(42)]);

        let mapped: EitherT<Vec<Result<i32, &str>>> = EitherT::right_vec(21);
        let result = mapped.map_vec(|x| x * 2);
        assert_eq!(result.run(), alloc::vec![Ok(42)]);
    }
}
