//! # `ContinuatioT` (Continuation Monad)
//!
//! `ContinuatioT<R, A>` is a plain continuation monad in
//! continuation-passing style. Despite the `T` suffix, it does **not**
//! currently layer over a base monad `M` (there is no `M` type parameter) —
//! it is `Cont`, not a true `ContT`; the name is kept for forward
//! compatibility with a future transformer version.
//!
//! ## Quick Start
//!
//! Control program flow with continuations:
//!
//! ```rust
//! use ordofp_core::transformers::ContinuatioT;
//!
//! // Create a simple continuation
//! let cont = ContinuatioT::<i32, i32>::pure(42);
//!
//! // Run the continuation with an identity function
//! let result = cont.run(|x| x);
//! assert_eq!(result, 42);
//!
//! // Chain continuations with bind
//! let chained = ContinuatioT::<i32, i32>::pure(10)
//!     .bind(|x| ContinuatioT::pure(x + 5))
//!     .bind(|x| ContinuatioT::pure(x * 2));
//!
//! let final_result = chained.run(|x| x);
//! assert_eq!(final_result, 30); // ((10 + 5) * 2)
//! ```
//!
//! ## Core Concepts
//!
//! - **Continuation-Passing Style**: Functions receive an explicit continuation
//! - **Explicit Control Flow**: Implements patterns like early exit or backtracking
//! - **Composable Computations**: Continuations can be composed using monadic operations
//!
//! ## Scholastic Naming
//!
//! Following `OrdoFP`'s Scholastic naming convention:
//! - `ContinuatioT` - Latin form of "continuation transformer"
//! - `exsequi` - Latin for "to execute/run"

use alloc::sync::Arc;
use core::marker::PhantomData;

/// Type alias for the core continuation function type.
pub(crate) type ContFn<R, A> = dyn Fn(Arc<dyn Fn(A) -> R + Send + Sync>) -> R + Send + Sync;

/// The continuation monad transformer.
///
/// `ContinuatioT<R, A>` represents a computation that takes a continuation
/// (a function from `A` to `R`) and produces a result of type `R`.
///
/// # Type Parameters
///
/// * `R` - The final result type
/// * `A` - The intermediate value type
///
/// # Examples
///
/// ```rust
/// use ordofp_core::transformers::ContinuatioT;
///
/// // Create two continuations
/// let cont1 = ContinuatioT::<i32, i32>::pure(5);
/// let cont2 = ContinuatioT::<i32, i32>::pure(-1);
///
/// // Run the continuations
/// let result1 = cont1.run(|x| x);
/// let result2 = cont2.run(|x| x);
///
/// assert_eq!(result1, 5);
/// assert_eq!(result2, -1);
/// ```
pub struct ContinuatioT<R, A> {
    run_cont: Arc<ContFn<R, A>>,
    _phantom: PhantomData<(R, A)>,
}

impl<R, A> Clone for ContinuatioT<R, A> {
    fn clone(&self) -> Self {
        ContinuatioT {
            run_cont: self.run_cont.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<R: 'static, A: 'static> ContinuatioT<R, A> {
    /// Creates a new continuation from a function.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes a continuation and returns a result
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use ordofp_core::transformers::ContinuatioT;
    ///
    /// // Create a continuation that doubles its input and adds 1
    /// let cont = ContinuatioT::new(|k: Arc<dyn Fn(i32) -> i32 + Send + Sync>| {
    ///     let x = 5;
    ///     let doubled = x * 2;
    ///     k(doubled + 1)
    /// });
    ///
    /// // Run with identity
    /// let result = cont.run(|x| x);
    /// assert_eq!(result, 11);
    /// ```
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(Arc<dyn Fn(A) -> R + Send + Sync>) -> R + Send + Sync + 'static,
    {
        ContinuatioT {
            run_cont: Arc::new(f),
            _phantom: PhantomData,
        }
    }

    /// Runs the continuation with the given continuation function.
    ///
    /// Named `exsequi` (Latin for "to execute") in Scholastic style.
    ///
    /// # Arguments
    ///
    /// * `k` - A function that takes a value of type `A` and returns type `R`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::ContinuatioT;
    ///
    /// let cont = ContinuatioT::<i32, i32>::pure(42);
    /// let result = cont.exsequi(|x| x * 2);
    /// assert_eq!(result, 84);
    /// ```
    #[inline]
    pub fn exsequi<F>(&self, k: F) -> R
    where
        F: Fn(A) -> R + Send + Sync + 'static,
    {
        (self.run_cont)(Arc::new(k))
    }

    /// Alias for `exsequi`.
    #[inline]
    pub fn run<F>(&self, k: F) -> R
    where
        F: Fn(A) -> R + Send + Sync + 'static,
    {
        self.exsequi(k)
    }

    /// Creates a continuation that immediately returns the given value.
    ///
    /// This is `return`/`pure` for the continuation monad.
    ///
    /// # Arguments
    ///
    /// * `a` - The value to return
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::ContinuatioT;
    ///
    /// let cont = ContinuatioT::<i32, i32>::pure(42);
    /// let result = cont.run(|x| x * 2);
    /// assert_eq!(result, 84);
    /// ```
    #[inline]
    pub fn pure(a: A) -> Self
    where
        A: Clone + Send + Sync + 'static,
    {
        ContinuatioT::new(move |k| k(a.clone()))
    }

    /// Maps a function over the value inside this continuation.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms `A` into `B`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::ContinuatioT;
    ///
    /// let cont = ContinuatioT::<i32, i32>::pure(42);
    /// let doubled = cont.map(|x| x * 2);
    /// assert_eq!(doubled.run(|x| x), 84);
    /// ```
    #[inline]
    pub fn map<B, F>(self, f: F) -> ContinuatioT<R, B>
    where
        F: Fn(A) -> B + Send + Sync + 'static,
        B: 'static,
    {
        let f = Arc::new(f);
        let run_cont = self.run_cont;
        ContinuatioT::new(move |k| {
            let f = f.clone();
            run_cont(Arc::new(move |a| k(f(a))))
        })
    }

    /// Monadic bind operation for the continuation monad.
    ///
    /// Sequences continuation computations.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that transforms `A` into a new continuation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::transformers::ContinuatioT;
    ///
    /// let cont = ContinuatioT::<i32, i32>::pure(5);
    /// let doubled = cont.bind(|x| ContinuatioT::pure(x * 2));
    /// assert_eq!(doubled.run(|x| x), 10);
    /// ```
    #[inline]
    pub fn bind<B, F>(self, f: F) -> ContinuatioT<R, B>
    where
        F: Fn(A) -> ContinuatioT<R, B> + Send + Sync + 'static,
        B: 'static,
    {
        let f = Arc::new(f);
        let run_cont = self.run_cont;
        ContinuatioT::new(move |k| {
            let f = f.clone();
            let k = k.clone();
            run_cont(Arc::new(move |a| {
                let cont_b = f(a);
                (cont_b.run_cont)(k.clone())
            }))
        })
    }

    /// Alias for `bind`.
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> ContinuatioT<R, B>
    where
        F: Fn(A) -> ContinuatioT<R, B> + Send + Sync + 'static,
        B: 'static,
    {
        self.bind(f)
    }

    /// Call with current continuation (call/cc).
    ///
    /// Captures the current continuation and passes it to the given function.
    /// This enables advanced control flow patterns like early returns.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that receives an escape continuation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use ordofp_core::transformers::ContinuatioT;
    ///
    /// // Use call_cc to implement early return
    /// let computation = ContinuatioT::<i32, i32>::call_cc(|exit| {
    ///     // If condition is met, exit early
    ///     if 5 > 3 {
    ///         exit(10)
    ///     } else {
    ///         ContinuatioT::pure(5)
    ///     }
    /// });
    ///
    /// assert_eq!(computation.run(|x| x), 10);
    /// ```
    #[inline]
    pub fn call_cc<B, F>(f: F) -> ContinuatioT<R, A>
    where
        F: Fn(Arc<dyn Fn(A) -> ContinuatioT<R, B> + Send + Sync>) -> ContinuatioT<R, A>
            + Send
            + Sync
            + 'static,
        A: Clone + Send + Sync + 'static,
        B: 'static,
    {
        ContinuatioT::new(move |k| {
            let k_clone = k.clone();
            let escape = Arc::new(move |a: A| {
                let k_inner = k_clone.clone();
                ContinuatioT::<R, B>::new(move |_ignored| k_inner(a.clone()))
            });
            (f(escape).run_cont)(k)
        })
    }
}

/// Applies a function wrapped in a continuation to a value in another continuation.
impl<R: 'static, A: 'static> ContinuatioT<R, A> {
    /// Applicative apply operation.
    ///
    /// # Arguments
    ///
    /// * `cf` - A continuation containing a function
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use ordofp_core::transformers::ContinuatioT;
    ///
    /// let cont_val = ContinuatioT::<i32, i32>::pure(5);
    /// let cont_fn: ContinuatioT<i32, Arc<dyn Fn(i32) -> i32 + Send + Sync>> =
    ///     ContinuatioT::pure(Arc::new(|x| x * 2) as Arc<dyn Fn(i32) -> i32 + Send + Sync>);
    ///
    /// let result = cont_val.apply(cont_fn).run(|x| x);
    /// assert_eq!(result, 10);
    /// ```
    #[inline]
    pub fn apply<B>(
        self,
        cf: ContinuatioT<R, Arc<dyn Fn(A) -> B + Send + Sync>>,
    ) -> ContinuatioT<R, B>
    where
        B: 'static,
    {
        let run_val = self.run_cont;
        let run_fn = cf.run_cont;
        ContinuatioT::new(move |k| {
            let run_val = run_val.clone();
            let k = Arc::new(k);
            run_fn(Arc::new(move |f| {
                let k = k.clone();
                run_val(Arc::new(move |a| k(f(a))))
            }))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_and_run() {
        let cont = ContinuatioT::<i32, i32>::pure(42);
        assert_eq!(cont.run(|x| x), 42);
        assert_eq!(cont.run(|x| x * 2), 84);
    }

    #[test]
    fn test_map() {
        let cont = ContinuatioT::<i32, i32>::pure(21);
        let doubled = cont.map(|x| x * 2);
        assert_eq!(doubled.run(|x| x), 42);
    }

    #[test]
    fn test_bind_chain() {
        let result = ContinuatioT::<i32, i32>::pure(5)
            .bind(|x| ContinuatioT::pure(x + 3))
            .bind(|x| ContinuatioT::pure(x * 2))
            .run(|x| x);

        assert_eq!(result, 16); // (5 + 3) * 2
    }

    #[test]
    fn test_call_cc_early_exit() {
        let result = ContinuatioT::<i32, i32>::call_cc(|exit| {
            if 10 > 5 {
                exit(100)
            } else {
                ContinuatioT::pure(0)
            }
        })
        .run(|x| x);

        assert_eq!(result, 100);
    }

    #[test]
    fn test_call_cc_no_exit() {
        let result = ContinuatioT::<i32, i32>::call_cc(
            |_exit: Arc<dyn Fn(i32) -> ContinuatioT<i32, i32> + Send + Sync>| {
                ContinuatioT::pure(42)
            },
        )
        .run(|x| x);

        assert_eq!(result, 42);
    }

    #[test]
    fn test_left_identity() {
        // pure(a).bind(f) == f(a)
        let a = 5;
        let f = |x: i32| ContinuatioT::pure(x * 2);

        let left = ContinuatioT::<i32, i32>::pure(a).bind(f);
        let right = f(a);

        assert_eq!(left.run(|x| x), right.run(|x| x));
    }

    #[test]
    fn test_right_identity() {
        // m.bind(pure) == m
        let m = ContinuatioT::<i32, i32>::pure(42);
        let bound = m.clone().bind(ContinuatioT::pure);

        assert_eq!(m.run(|x| x), bound.run(|x| x));
    }

    #[test]
    fn test_associativity() {
        // (m.bind(f)).bind(g) == m.bind(|x| f(x).bind(g))
        let m = ContinuatioT::<i32, i32>::pure(5);

        let left = m
            .clone()
            .bind(|x| ContinuatioT::pure(x + 1))
            .bind(|x| ContinuatioT::pure(x * 2));

        let right = m.bind(|x| {
            let inner = ContinuatioT::pure(x + 1);
            inner.bind(|y| ContinuatioT::pure(y * 2))
        });

        assert_eq!(left.run(|x| x), right.run(|x| x));
    }
}
