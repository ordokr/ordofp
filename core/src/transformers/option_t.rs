//! `OptionT` Monad Transformer
//!
//! `OptionT` adds optionality to any base monad. It wraps a computation that
//! returns `M<Option<A>>` and provides a monadic interface that handles the
//! `Option` layer automatically.
//!
//! # Design note
//!
//! `OptionT` is currently implemented via three concrete `impl` blocks — one
//! each for `OptionT<Option<Option<A>>>`, `OptionT<Result<Option<A>, E>>`, and
//! `OptionT<Vec<Option<A>>>`. A single generic `impl<M: Monad> OptionT<M>`
//! block is **not** currently possible because:
//!
//! 1. The transformer must case-split on the *inner* `Option<A>` to implement
//!    `map`/`flat_map` (e.g. `Some(a) -> f(a)`, `None -> None`). The `Monad`
//!    trait in this crate has a single `Inner` associated type that represents
//!    the outer wrapped value — to generalise, we would need a
//!    `Monad<Inner = Option<A>>` constraint pattern with a way to project out
//!    the `A`, which requires a `NestedMonad` / "inner-type-family" typeclass
//!    extension that does not exist in this crate today.
//! 2. The `Vec` variant uses `FnMut` for its function arguments (because
//!    `Vec::into_iter().map(..).collect()` expects `FnMut`), while the
//!    `Option` and `Result` variants use `FnOnce` (they invoke the closure at
//!    most once). Merging these into a single generic impl would require
//!    widening every caller's bound to `FnMut`, which is a strict API
//!    regression for the `Option` and `Result` paths.
//!
//! Until the typeclass extension lands, the three concrete impls are the
//! implementation mechanism. The duplication is a known DRY cost carried
//! deliberately; generalising prematurely would either break public bounds
//! (point 2) or require unstable Rust features (point 1).
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # fn main() {
//! use ordofp_core::transformers::OptionT;
//!
//! // OptionT over Result - combines optionality with error handling
//! let computation: OptionT<Result<Option<i32>, &str>> = OptionT::some(42);
//! let result = computation
//!     .map(|x| x * 2)
//!     .flat_map(|x| if x > 50 { OptionT::some(x) } else { OptionT::none() });
//! assert_eq!(result.run(), Ok(Some(84)));
//! # }
//! # #[cfg(not(feature = "alloc"))]
//! # fn main() {}
//! ```

use super::MonadTransformer;

/// The `OptionT` monad transformer adds optionality to any base monad `M`.
///
/// `OptionT<M>` wraps `M<Option<A>>` and provides a unified interface for working
/// with optional values inside another monadic context.
///
/// # Type Parameters
///
/// - `M`: The base monad wrapping `Option<A>` (e.g., `Result<Option<A>, E>`)
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # fn main() {
/// use ordofp_core::transformers::OptionT;
///
/// // Create an OptionT with a value
/// let opt: OptionT<Result<Option<i32>, &str>> = OptionT::some(42);
/// assert_eq!(opt.run(), Ok(Some(42)));
///
/// // Create an empty OptionT
/// let none: OptionT<Result<Option<i32>, &str>> = OptionT::none();
/// assert_eq!(none.run(), Ok(None));
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
/// use ordofp_core::transformers::OptionT;
///
/// fn safe_div(a: i32, b: i32) -> OptionT<Result<Option<i32>, &'static str>> {
///     if b == 0 {
///         OptionT::none()
///     } else {
///         OptionT::some(a / b)
///     }
/// }
///
/// let result = OptionT::<Result<Option<i32>, &str>>::some(100)
///     .flat_map(|x| safe_div(x, 2))
///     .flat_map(|x| safe_div(x, 5));
/// assert_eq!(result.run(), Ok(Some(10)));
///
/// // Division by zero returns None
/// let result2 = OptionT::<Result<Option<i32>, &str>>::some(100)
///     .flat_map(|x| safe_div(x, 0));
/// assert_eq!(result2.run(), Ok(None));
/// # }
/// # #[cfg(not(feature = "alloc"))]
/// # fn main() {}
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionT<M> {
    /// The wrapped computation
    inner: M,
}

impl<M> OptionT<M> {
    /// Creates a new `OptionT` from a wrapped computation.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let inner: Result<Option<i32>, &str> = Ok(Some(42));
    /// let opt = OptionT::new(inner);
    /// assert_eq!(opt.run(), Ok(Some(42)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn new(inner: M) -> Self {
        OptionT { inner }
    }

    /// Runs the transformer, extracting the inner computation.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let opt: OptionT<Result<Option<i32>, &str>> = OptionT::some(42);
    /// let result: Result<Option<i32>, &str> = opt.run();
    /// assert_eq!(result, Ok(Some(42)));
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
// OptionT over Option
// ============================================================================

impl<A> OptionT<Option<Option<A>>> {
    /// Creates an `OptionT` containing `Some(value)` over `Option`.
    #[inline]
    pub fn some_option(value: A) -> Self {
        OptionT::new(Some(Some(value)))
    }

    /// Creates an empty `OptionT` over `Option`.
    #[inline]
    pub fn none_option() -> Self {
        OptionT::new(Some(None))
    }

    /// Maps a function over the inner value.
    #[inline]
    pub fn map<B, F>(self, f: F) -> OptionT<Option<Option<B>>>
    where
        F: FnOnce(A) -> B,
    {
        OptionT::new(self.inner.map(|opt| opt.map(f)))
    }

    /// Chains a computation that returns an `OptionT`.
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> OptionT<Option<Option<B>>>
    where
        F: FnOnce(A) -> OptionT<Option<Option<B>>>,
    {
        OptionT::new(match self.inner {
            Some(Some(a)) => f(a).inner,
            Some(None) => Some(None),
            None => None,
        })
    }

    /// Applies a wrapped function to this value.
    #[inline]
    pub fn apply<B, F>(self, f: OptionT<Option<Option<F>>>) -> OptionT<Option<Option<B>>>
    where
        F: FnOnce(A) -> B,
    {
        OptionT::new(match (self.inner, f.inner) {
            (Some(Some(a)), Some(Some(func))) => Some(Some(func(a))),
            (Some(None), _) | (_, Some(None)) => Some(None),
            (None, _) | (_, None) => None,
        })
    }
}

impl<A> MonadTransformer for OptionT<Option<Option<A>>> {
    type BaseMonad = Option<A>;

    #[inline]
    fn lift(base: Option<A>) -> Self {
        OptionT::new(Some(base))
    }
}

// ============================================================================
// OptionT over Result
// ============================================================================

impl<A, E> OptionT<Result<Option<A>, E>> {
    /// Creates an `OptionT` containing `Some(value)` over `Result`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let opt: OptionT<Result<Option<i32>, &str>> = OptionT::some(42);
    /// assert_eq!(opt.run(), Ok(Some(42)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn some(value: A) -> Self {
        OptionT::new(Ok(Some(value)))
    }

    /// Creates an empty `OptionT` over `Result`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let opt: OptionT<Result<Option<i32>, &str>> = OptionT::none();
    /// assert_eq!(opt.run(), Ok(None));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn none() -> Self {
        OptionT::new(Ok(None))
    }

    /// Creates an `OptionT` representing an error.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let opt: OptionT<Result<Option<i32>, &str>> = OptionT::err("error");
    /// assert_eq!(opt.run(), Err("error"));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn err(e: E) -> Self {
        OptionT::new(Err(e))
    }

    /// Lifts a `Result` value into `OptionT`, wrapping the success case in `Some`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let result: Result<i32, &str> = Ok(42);
    /// let opt: OptionT<Result<Option<i32>, &str>> = OptionT::lift_m(result);
    /// assert_eq!(opt.run(), Ok(Some(42)));
    ///
    /// let err_result: Result<i32, &str> = Err("error");
    /// let opt_err: OptionT<Result<Option<i32>, &str>> = OptionT::lift_m(err_result);
    /// assert_eq!(opt_err.run(), Err("error"));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn lift_m(result: Result<A, E>) -> Self {
        OptionT::new(result.map(Some))
    }

    /// Maps a function over the inner value.
    ///
    /// If the computation is `Ok(Some(a))`, applies `f` to `a`.
    /// If the computation is `Ok(None)` or `Err(e)`, propagates unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let opt: OptionT<Result<Option<i32>, &str>> = OptionT::some(21);
    /// let doubled = opt.map(|x| x * 2);
    /// assert_eq!(doubled.run(), Ok(Some(42)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn map<B, F>(self, f: F) -> OptionT<Result<Option<B>, E>>
    where
        F: FnOnce(A) -> B,
    {
        OptionT::new(self.inner.map(|opt| opt.map(f)))
    }

    /// Maps a function over the error value.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let opt: OptionT<Result<Option<i32>, i32>> = OptionT::err(1);
    /// let mapped = opt.map_err(|e| e * 2);
    /// assert_eq!(mapped.run(), Err(2));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn map_err<E2, F>(self, f: F) -> OptionT<Result<Option<A>, E2>>
    where
        F: FnOnce(E) -> E2,
    {
        OptionT::new(self.inner.map_err(f))
    }

    /// Chains a computation that returns an `OptionT`.
    ///
    /// Also known as `bind` or `>>=`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let opt: OptionT<Result<Option<i32>, &str>> = OptionT::some(10);
    /// let result = opt.flat_map(|x| {
    ///     if x > 5 { OptionT::some(x * 2) }
    ///     else { OptionT::none() }
    /// });
    /// assert_eq!(result.run(), Ok(Some(20)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> OptionT<Result<Option<B>, E>>
    where
        F: FnOnce(A) -> OptionT<Result<Option<B>, E>>,
    {
        OptionT::new(match self.inner {
            Ok(Some(a)) => f(a).inner,
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        })
    }

    /// Applies a wrapped function to this value.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let val: OptionT<Result<Option<i32>, &str>> = OptionT::some(21);
    /// let func: OptionT<Result<Option<fn(i32) -> i32>, &str>> =
    ///     OptionT::some(|x: i32| x * 2);
    /// let result = val.apply(func);
    /// assert_eq!(result.run(), Ok(Some(42)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn apply<B, F>(self, f: OptionT<Result<Option<F>, E>>) -> OptionT<Result<Option<B>, E>>
    where
        F: FnOnce(A) -> B,
    {
        OptionT::new(match (self.inner, f.inner) {
            (Ok(Some(a)), Ok(Some(func))) => Ok(Some(func(a))),
            (Ok(None), _) | (_, Ok(None)) => Ok(None),
            (Err(e), _) | (_, Err(e)) => Err(e),
        })
    }

    /// Combines two `OptionT` values using a function.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let a: OptionT<Result<Option<i32>, &str>> = OptionT::some(10);
    /// let b: OptionT<Result<Option<i32>, &str>> = OptionT::some(20);
    /// let combined = a.map2(b, |x, y| x + y);
    /// assert_eq!(combined.run(), Ok(Some(30)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn map2<B, C, F>(
        self,
        other: OptionT<Result<Option<B>, E>>,
        f: F,
    ) -> OptionT<Result<Option<C>, E>>
    where
        F: FnOnce(A, B) -> C,
    {
        OptionT::new(match (self.inner, other.inner) {
            (Ok(Some(a)), Ok(Some(b))) => Ok(Some(f(a, b))),
            (Ok(None), _) | (_, Ok(None)) => Ok(None),
            (Err(e), _) | (_, Err(e)) => Err(e),
        })
    }

    /// Returns `self` if it contains a value, otherwise returns `other`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let a: OptionT<Result<Option<i32>, &str>> = OptionT::none();
    /// let b: OptionT<Result<Option<i32>, &str>> = OptionT::some(42);
    /// assert_eq!(a.or_else(|| b).run(), Ok(Some(42)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        OptionT::new(match self.inner {
            Ok(Some(a)) => Ok(Some(a)),
            Ok(None) => f().inner,
            Err(e) => Err(e),
        })
    }

    /// Extracts the value or returns a default.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::OptionT;
    ///
    /// let some: OptionT<Result<Option<i32>, &str>> = OptionT::some(42);
    /// assert_eq!(some.get_or_else(|| 0), Ok(42));
    ///
    /// let none: OptionT<Result<Option<i32>, &str>> = OptionT::none();
    /// assert_eq!(none.get_or_else(|| 0), Ok(0));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    ///
    /// # Errors
    ///
    /// Propagates the underlying `Err(e)` unchanged; the default is only
    /// consulted for the `Ok(None)` case.
    #[inline]
    pub fn get_or_else<F>(self, default: F) -> Result<A, E>
    where
        F: FnOnce() -> A,
    {
        self.inner.map(|opt| opt.unwrap_or_else(default))
    }

    /// Converts `OptionT<Result<Option<A>, E>>` to `Result<Option<A>, E>`.
    ///
    /// This is an alias for `run()` with a more descriptive name.
    ///
    /// # Errors
    ///
    /// Returns the underlying `Err(e)` unchanged when the wrapped
    /// computation failed; this unwrapping introduces no new error
    /// condition.
    #[inline]
    pub fn to_result(self) -> Result<Option<A>, E> {
        self.inner
    }

    /// Checks if the computation contains a value.
    #[inline]
    pub fn is_some(&self) -> bool {
        matches!(&self.inner, Ok(Some(_)))
    }

    /// Checks if the computation is empty (None).
    #[inline]
    pub fn is_none(&self) -> bool {
        matches!(&self.inner, Ok(None))
    }

    /// Checks if the computation is an error.
    #[inline]
    pub fn is_err(&self) -> bool {
        self.inner.is_err()
    }
}

impl<A, E> MonadTransformer for OptionT<Result<Option<A>, E>> {
    type BaseMonad = Result<A, E>;

    #[inline]
    fn lift(base: Result<A, E>) -> Self {
        OptionT::lift_m(base)
    }
}

// ============================================================================
// OptionT over Vec
// ============================================================================

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
impl<A> OptionT<Vec<Option<A>>> {
    /// Creates an `OptionT` containing `Some(value)` over `Vec`.
    #[inline]
    pub fn some_vec(value: A) -> Self {
        OptionT::new(alloc::vec![Some(value)])
    }

    /// Creates an empty `OptionT` over `Vec`.
    #[inline]
    pub fn none_vec() -> Self {
        OptionT::new(alloc::vec![None])
    }

    /// Creates an `OptionT` from multiple optional values.
    #[inline]
    pub fn from_vec(values: Vec<Option<A>>) -> Self {
        OptionT::new(values)
    }

    /// Maps a function over all inner values.
    #[inline]
    pub fn map<B, F>(self, mut f: F) -> OptionT<Vec<Option<B>>>
    where
        F: FnMut(A) -> B,
    {
        OptionT::new(self.inner.into_iter().map(|opt| opt.map(&mut f)).collect())
    }

    /// Chains a computation that returns an `OptionT`.
    #[inline]
    pub fn flat_map<B, F>(self, mut f: F) -> OptionT<Vec<Option<B>>>
    where
        F: FnMut(A) -> OptionT<Vec<Option<B>>>,
    {
        let results: Vec<Option<B>> = self
            .inner
            .into_iter()
            .flat_map(|opt| match opt {
                Some(a) => f(a).inner,
                None => alloc::vec![None],
            })
            .collect();
        OptionT::new(results)
    }
}

#[cfg(feature = "alloc")]
impl<A> MonadTransformer for OptionT<Vec<Option<A>>> {
    type BaseMonad = Vec<A>;

    #[inline]
    fn lift(base: Vec<A>) -> Self {
        OptionT::new(base.into_iter().map(Some).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_t_some() {
        let opt: OptionT<Result<Option<i32>, &str>> = OptionT::some(42);
        assert_eq!(opt.run(), Ok(Some(42)));
    }

    #[test]
    fn test_option_t_none() {
        let opt: OptionT<Result<Option<i32>, &str>> = OptionT::none();
        assert_eq!(opt.run(), Ok(None));
    }

    #[test]
    fn test_option_t_err() {
        let opt: OptionT<Result<Option<i32>, &str>> = OptionT::err("error");
        assert_eq!(opt.run(), Err("error"));
    }

    #[test]
    fn test_option_t_map() {
        let opt: OptionT<Result<Option<i32>, &str>> = OptionT::some(21);
        let result = opt.map(|x| x * 2);
        assert_eq!(result.run(), Ok(Some(42)));

        let none: OptionT<Result<Option<i32>, &str>> = OptionT::none();
        let result_none = none.map(|x| x * 2);
        assert_eq!(result_none.run(), Ok(None));
    }

    #[test]
    fn test_option_t_flat_map() {
        let opt: OptionT<Result<Option<i32>, &str>> = OptionT::some(10);
        let result = opt.flat_map(|x| {
            if x > 5 {
                OptionT::some(x * 2)
            } else {
                OptionT::none()
            }
        });
        assert_eq!(result.run(), Ok(Some(20)));

        let opt2: OptionT<Result<Option<i32>, &str>> = OptionT::some(3);
        let result2 = opt2.flat_map(|x| {
            if x > 5 {
                OptionT::some(x * 2)
            } else {
                OptionT::none()
            }
        });
        assert_eq!(result2.run(), Ok(None));
    }

    #[test]
    #[allow(clippy::type_complexity)] // spelled-out nested transformer type is the point
    fn test_option_t_apply() {
        let val: OptionT<Result<Option<i32>, &str>> = OptionT::some(21);
        let func: OptionT<Result<Option<fn(i32) -> i32>, &str>> = OptionT::some(|x: i32| x * 2);
        let result = val.apply(func);
        assert_eq!(result.run(), Ok(Some(42)));
    }

    #[test]
    fn test_option_t_lift_m() {
        let ok: Result<i32, &str> = Ok(42);
        let lifted: OptionT<Result<Option<i32>, &str>> = OptionT::lift_m(ok);
        assert_eq!(lifted.run(), Ok(Some(42)));

        let err: Result<i32, &str> = Err("error");
        let lifted_err: OptionT<Result<Option<i32>, &str>> = OptionT::lift_m(err);
        assert_eq!(lifted_err.run(), Err("error"));
    }

    #[test]
    fn test_option_t_map2() {
        let a: OptionT<Result<Option<i32>, &str>> = OptionT::some(10);
        let b: OptionT<Result<Option<i32>, &str>> = OptionT::some(20);
        let combined = a.map2(b, |x, y| x + y);
        assert_eq!(combined.run(), Ok(Some(30)));
    }

    #[test]
    fn test_option_t_or_else() {
        let a: OptionT<Result<Option<i32>, &str>> = OptionT::none();
        let b = || OptionT::some(42);
        assert_eq!(a.or_else(b).run(), Ok(Some(42)));
    }

    // Monad law tests
    #[test]
    fn test_option_t_left_identity() {
        // pure(a).flat_map(f) == f(a)
        let a = 5;
        let f = |x: i32| OptionT::<Result<Option<i32>, &str>>::some(x * 2);

        let left = OptionT::<Result<Option<i32>, &str>>::some(a).flat_map(f);
        let right = f(a);
        assert_eq!(left.run(), right.run());
    }

    #[test]
    fn test_option_t_right_identity() {
        // m.flat_map(pure) == m
        let m: OptionT<Result<Option<i32>, &str>> = OptionT::some(42);
        let result = m.flat_map(OptionT::some);
        assert_eq!(result.run(), Ok(Some(42)));
    }

    #[test]
    fn test_option_t_associativity() {
        // m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
        let m: OptionT<Result<Option<i32>, &str>> = OptionT::some(5);
        let f = |x: i32| OptionT::<Result<Option<i32>, &str>>::some(x + 1);
        let g = |x: i32| OptionT::<Result<Option<i32>, &str>>::some(x * 2);

        let left = m.flat_map(f).flat_map(g);
        let right = OptionT::<Result<Option<i32>, &str>>::some(5).flat_map(|x| f(x).flat_map(g));
        assert_eq!(left.run(), right.run());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_option_t_vec() {
        let opt: OptionT<Vec<Option<i32>>> = OptionT::some_vec(42);
        assert_eq!(opt.run(), alloc::vec![Some(42)]);

        let mapped = OptionT::some_vec(21).map(|x| x * 2);
        assert_eq!(mapped.run(), alloc::vec![Some(42)]);
    }
}
