//! `StateT` Monad Transformer
//!
//! `StateT` adds stateful computation capabilities to any base monad.
//! It represents computations that can read and modify a state value
//! while supporting the effects of the underlying monad.
//!
//! # Supported base monads
//!
//! `StateT` ships concrete `map`/`flat_map`/`pure` impls for the base monads
//! `Option<(S, A)>`, `Result<(S, A), E>`, and `Vec<(S, A)>`. There is **no**
//! single generic impl over every `Monad` — see the design note below for
//! why.
//!
//! # Design note
//!
//! Unlike [`ReaderT`](crate::transformers::ReaderT), `StateT` does **not**
//! currently expose a single fully generic `impl<S, M: Monad> StateT<S, M>`
//! block. The reason is structural: the `Monad` trait in this crate has a
//! single associated type `Inner` that represents the wrapped value. For
//! `StateT`, the wrapped value is `(S, A)` — a nested tuple — and we need to
//! speak about the *inner* `A` while keeping the state `S` in place. Encoding
//! that pattern as a bound (something like `Monad<Inner = (S, A)>` with `A`
//! projected out) requires a `NestedMonad` / "inner-type-family" typeclass
//! extension, along with HKT-style plumbing, that does not exist in this
//! crate today.
//!
//! Until such an extension lands, the concrete `impl<S, A> StateT<S, Option<(S, A)>>`,
//! `StateT<S, Result<(S, A), E>>`, and `StateT<S, Vec<(S, A)>>` blocks are the
//! implementation mechanism. Each duplicates the shape of `map`/`flat_map`
//! for its specific outer monad. This is a known DRY cost we are carrying
//! deliberately; the alternative — degrading the public API to only support
//! the monads we have now — is worse.
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # fn main() {
//! use ordofp_core::transformers::StateT;
//!
//! // StateT over Option - stateful computation that may fail
//! let computation: StateT<i32, Option<(i32, i32)>> = StateT::new(|state: i32| {
//!     if state >= 0 {
//!         Some((state + 1, state * 2))  // (new_state, value)
//!     } else {
//!         None
//!     }
//! });
//!
//! assert_eq!(computation.run(5), Some((6, 10)));
//! assert_eq!(computation.run(-1), None);
//! # }
//! # #[cfg(not(feature = "alloc"))]
//! # fn main() {}
//! ```

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// The `StateT` monad transformer adds state management to any base monad `M`.
///
/// `StateT<S, M>` represents a computation that takes an initial state of type `S`,
/// and returns a value in the monad `M` containing both the new state and a result value.
/// The monad `M` wraps a tuple `(S, A)` where `S` is the state and `A` is the value.
///
/// # Type Parameters
///
/// - `S`: The state type
/// - `M`: The base monad wrapping `(S, A)` (e.g., `Option<(S, A)>`, `Result<(S, A), E>`)
///
/// # State Convention
///
/// The wrapped monad `M` should contain a tuple `(new_state, value)`:
/// - First element: the updated state
/// - Second element: the computed value
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # fn main() {
/// use ordofp_core::transformers::StateT;
///
/// // A counter that increments state and returns the old value
/// let increment: StateT<i32, Option<(i32, i32)>> = StateT::new(|s: i32| {
///     Some((s + 1, s))
/// });
///
/// assert_eq!(increment.run(0), Some((1, 0)));
/// assert_eq!(increment.run(5), Some((6, 5)));
/// # }
/// # #[cfg(not(feature = "alloc"))]
/// # fn main() {}
/// ```
///
/// ## Chaining Stateful Computations
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # fn main() {
/// use ordofp_core::transformers::StateT;
///
/// // Increment and then double
/// let inc: StateT<i32, Option<(i32, i32)>> = StateT::new(|s: i32| Some((s + 1, s)));
/// let double_state = inc.flat_map(|_old_val| {
///     StateT::new(|s: i32| Some((s * 2, s)))
/// });
///
/// // Start at 5: inc gives (6, 5), then double_state gives (12, 6)
/// assert_eq!(double_state.run(5), Some((12, 6)));
/// # }
/// # #[cfg(not(feature = "alloc"))]
/// # fn main() {}
/// ```
#[cfg(feature = "alloc")]
pub struct StateT<S, M> {
    /// The state transformation function
    run_fn: Box<dyn Fn(S) -> M + Send + Sync>,
}

// ============================================================================
// Core operations (work for any M)
// ============================================================================

#[cfg(feature = "alloc")]
impl<S, M> StateT<S, M> {
    /// Creates a new `StateT` from a function.
    ///
    /// The function takes an initial state and returns a monadic value
    /// containing the new state and result.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::StateT;
    ///
    /// let state_t: StateT<i32, Option<(i32, String)>> = StateT::new(|s: i32| {
    ///     Some((s + 1, format!("was {}", s)))
    /// });
    /// assert_eq!(state_t.run(42), Some((43, "was 42".to_string())));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(S) -> M + Send + Sync + 'static,
    {
        StateT {
            run_fn: Box::new(f),
        }
    }

    /// Runs the stateful computation with an initial state.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::StateT;
    ///
    /// let inc: StateT<i32, Option<(i32, i32)>> = StateT::new(|s: i32| Some((s + 1, s)));
    /// assert_eq!(inc.run(10), Some((11, 10)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn run(&self, state: S) -> M {
        (self.run_fn)(state)
    }
}

// ============================================================================
// StateT over Option — Universalis + convenience
// ============================================================================

#[cfg(feature = "alloc")]
impl<S: 'static, A: 'static> StateT<S, Option<(S, A)>> {
    /// Creates a `StateT` that returns a value without changing state.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::StateT;
    ///
    /// let pure_val: StateT<i32, Option<(i32, &str)>> = StateT::pure("hello");
    /// assert_eq!(pure_val.run(42), Some((42, "hello")));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn pure(value: A) -> Self
    where
        A: Clone + Send + Sync,
    {
        StateT::new(move |s: S| Some((s, value.clone())))
    }

    /// Creates a `StateT` that always fails.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::StateT;
    ///
    /// let fail: StateT<i32, Option<(i32, i32)>> = StateT::none();
    /// assert_eq!(fail.run(42), None);
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn none() -> Self {
        StateT::new(|_: S| None)
    }

    /// Creates a `StateT` that returns the current state without modifying it.
    #[inline]
    pub fn get() -> StateT<S, Option<(S, S)>>
    where
        S: Clone + Send + Sync,
    {
        StateT::new(|s: S| Some((s.clone(), s)))
    }

    /// Creates a `StateT` that sets the state and returns the old state.
    #[inline]
    pub fn put(new_state: S) -> StateT<S, Option<(S, S)>>
    where
        S: Clone + Send + Sync,
    {
        StateT::new(move |old: S| Some((new_state.clone(), old)))
    }

    /// Creates a `StateT` that modifies the state with a function.
    #[inline]
    pub fn modify<F>(f: F) -> StateT<S, Option<(S, ())>>
    where
        F: Fn(S) -> S + Send + Sync + 'static,
    {
        StateT::new(move |s: S| Some((f(s), ())))
    }

    /// Maps a function over the value.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::StateT;
    ///
    /// let inc: StateT<i32, Option<(i32, i32)>> = StateT::new(|s: i32| Some((s + 1, s)));
    /// let doubled = inc.map(|v| v * 2);
    /// assert_eq!(doubled.run(5), Some((6, 10))); // state: 6, value: 5*2=10
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn map<B, F>(self, f: F) -> StateT<S, Option<(S, B)>>
    where
        F: Fn(A) -> B + Send + Sync + 'static,
        B: 'static,
    {
        StateT::new(move |s: S| (self.run_fn)(s).map(|(new_s, a)| (new_s, f(a))))
    }

    /// Chains a computation that returns a `StateT`.
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # fn main() {
    /// use ordofp_core::transformers::StateT;
    ///
    /// let inc: StateT<i32, Option<(i32, i32)>> = StateT::new(|s: i32| Some((s + 1, s)));
    /// let chained = inc.flat_map(|val| {
    ///     StateT::new(move |s: i32| Some((s * 2, val + s)))
    /// });
    /// // Start: 5, inc: (6, 5), then (6*2, 5+6) = (12, 11)
    /// assert_eq!(chained.run(5), Some((12, 11)));
    /// # }
    /// # #[cfg(not(feature = "alloc"))]
    /// # fn main() {}
    /// ```
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> StateT<S, Option<(S, B)>>
    where
        F: Fn(A) -> StateT<S, Option<(S, B)>> + Send + Sync + 'static,
        B: 'static,
    {
        StateT::new(move |s: S| match (self.run_fn)(s) {
            Some((new_s, a)) => f(a).run(new_s),
            None => None,
        })
    }

    /// Runs the computation and returns only the final state.
    #[inline]
    pub fn exec_state(&self, state: S) -> Option<S> {
        self.run(state).map(|(s, _)| s)
    }

    /// Runs the computation and returns only the final value.
    #[inline]
    pub fn eval_state(&self, state: S) -> Option<A> {
        self.run(state).map(|(_, a)| a)
    }

    /// Applies a wrapped function to this value.
    #[inline]
    pub fn apply<B, F>(self, sf: StateT<S, Option<(S, F)>>) -> StateT<S, Option<(S, B)>>
    where
        F: FnOnce(A) -> B + Clone + Send + Sync + 'static,
        B: 'static,
    {
        StateT::new(move |s: S| match sf.run(s) {
            Some((s1, f)) => (self.run_fn)(s1).map(|(s2, a)| (s2, f(a))),
            None => None,
        })
    }

    /// Combines two stateful computations with a function.
    #[inline]
    pub fn map2<B, C, F>(self, other: StateT<S, Option<(S, B)>>, f: F) -> StateT<S, Option<(S, C)>>
    where
        F: Fn(A, B) -> C + Send + Sync + 'static,
        B: 'static,
        C: 'static,
    {
        StateT::new(move |s: S| match (self.run_fn)(s) {
            Some((s1, a)) => other.run(s1).map(|(s2, b)| (s2, f(a, b))),
            None => None,
        })
    }
}

// ============================================================================
// StateT over Result — convenience methods
// ============================================================================

#[cfg(feature = "alloc")]
impl<S: 'static, A: 'static, E: 'static> StateT<S, Result<(S, A), E>> {
    /// Creates a `StateT` that returns a value without changing state.
    #[inline]
    pub fn ok(value: A) -> Self
    where
        A: Clone + Send + Sync,
    {
        StateT::new(move |s: S| Ok((s, value.clone())))
    }

    /// Creates a `StateT` that always returns an error.
    #[inline]
    pub fn err(error: E) -> Self
    where
        E: Clone + Send + Sync,
    {
        StateT::new(move |_: S| Err(error.clone()))
    }

    /// Creates a `StateT` that returns the current state.
    #[inline]
    pub fn get_result() -> StateT<S, Result<(S, S), E>>
    where
        S: Clone + Send + Sync,
    {
        StateT::new(|s: S| Ok((s.clone(), s)))
    }

    /// Creates a `StateT` that sets the state.
    #[inline]
    pub fn put_result(new_state: S) -> StateT<S, Result<(S, S), E>>
    where
        S: Clone + Send + Sync,
    {
        StateT::new(move |old: S| Ok((new_state.clone(), old)))
    }

    /// Creates a `StateT` that modifies the state.
    #[inline]
    pub fn modify_result<F>(f: F) -> StateT<S, Result<(S, ()), E>>
    where
        F: Fn(S) -> S + Send + Sync + 'static,
    {
        StateT::new(move |s: S| Ok((f(s), ())))
    }

    /// Maps a function over the value.
    #[inline]
    pub fn map_ok<B, F>(self, f: F) -> StateT<S, Result<(S, B), E>>
    where
        F: Fn(A) -> B + Send + Sync + 'static,
        B: 'static,
    {
        StateT::new(move |s: S| (self.run_fn)(s).map(|(new_s, a)| (new_s, f(a))))
    }

    /// Maps a function over the error.
    #[inline]
    pub fn map_err<E2, F>(self, f: F) -> StateT<S, Result<(S, A), E2>>
    where
        F: Fn(E) -> E2 + Send + Sync + 'static,
        E2: 'static,
    {
        StateT::new(move |s: S| (self.run_fn)(s).map_err(&f))
    }

    /// Chains a computation that returns a `StateT`.
    #[inline]
    pub fn flat_map_ok<B, F>(self, f: F) -> StateT<S, Result<(S, B), E>>
    where
        F: Fn(A) -> StateT<S, Result<(S, B), E>> + Send + Sync + 'static,
        B: 'static,
    {
        StateT::new(move |s: S| match (self.run_fn)(s) {
            Ok((new_s, a)) => f(a).run(new_s),
            Err(e) => Err(e),
        })
    }

    /// Runs the computation and returns only the final state.
    ///
    /// # Errors
    ///
    /// Returns `Err(e)` when the underlying stateful computation fails on
    /// the given initial `state`; no final state is produced in that case.
    #[inline]
    pub fn exec_state_result(&self, state: S) -> Result<S, E> {
        self.run(state).map(|(s, _)| s)
    }

    /// Runs the computation and returns only the final value.
    ///
    /// # Errors
    ///
    /// Returns `Err(e)` when the underlying stateful computation fails on
    /// the given initial `state`; no value is produced in that case.
    #[inline]
    pub fn eval_state_result(&self, state: S) -> Result<A, E> {
        self.run(state).map(|(_, a)| a)
    }

    /// Combines two stateful computations with a function.
    #[inline]
    pub fn map2_ok<B, C, F>(
        self,
        other: StateT<S, Result<(S, B), E>>,
        f: F,
    ) -> StateT<S, Result<(S, C), E>>
    where
        F: Fn(A, B) -> C + Send + Sync + 'static,
        B: 'static,
        C: 'static,
    {
        StateT::new(move |s: S| match (self.run_fn)(s) {
            Ok((s1, a)) => match other.run(s1) {
                Ok((s2, b)) => Ok((s2, f(a, b))),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        })
    }
}

// ============================================================================
// StateT over Vec — convenience methods
// ============================================================================

#[cfg(feature = "alloc")]
impl<S: Clone + 'static, A: 'static> StateT<S, Vec<(S, A)>> {
    /// Creates a `StateT` that returns a single value.
    #[inline]
    pub fn singleton(value: A) -> Self
    where
        A: Clone + Send + Sync,
    {
        StateT::new(move |s: S| alloc::vec![(s, value.clone())])
    }

    /// Creates a `StateT` that returns no results.
    #[inline]
    pub fn empty_vec() -> Self {
        StateT::new(|_: S| alloc::vec![])
    }

    /// Maps a function over all values.
    #[inline]
    pub fn map_vec<B, F>(self, f: F) -> StateT<S, Vec<(S, B)>>
    where
        F: Fn(A) -> B + Send + Sync + 'static,
        B: 'static,
    {
        StateT::new(move |s: S| {
            (self.run_fn)(s)
                .into_iter()
                .map(|(new_s, a)| (new_s, f(a)))
                .collect()
        })
    }

    /// Chains a computation over all branches.
    #[inline]
    pub fn flat_map_vec<B, F>(self, f: F) -> StateT<S, Vec<(S, B)>>
    where
        F: Fn(A) -> StateT<S, Vec<(S, B)>> + Send + Sync + 'static,
        B: 'static,
    {
        StateT::new(move |s: S| {
            (self.run_fn)(s)
                .into_iter()
                .flat_map(|(new_s, a)| f(a).run(new_s))
                .collect()
        })
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn test_state_t_new_and_run() {
        let state_t: StateT<i32, Option<(i32, i32)>> = StateT::new(|s: i32| Some((s + 1, s)));
        assert_eq!(state_t.run(5), Some((6, 5)));
    }

    #[test]
    fn test_state_t_pure() {
        let pure_val: StateT<i32, Option<(i32, &str)>> = StateT::pure("hello");
        assert_eq!(pure_val.run(42), Some((42, "hello")));
    }

    #[test]
    fn test_state_t_none() {
        let fail: StateT<i32, Option<(i32, i32)>> = StateT::none();
        assert_eq!(fail.run(42), None);
    }

    #[test]
    fn test_state_t_get() {
        let get: StateT<i32, Option<(i32, i32)>> = StateT::<i32, Option<(i32, i32)>>::get();
        assert_eq!(get.run(42), Some((42, 42)));
    }

    #[test]
    fn test_state_t_put() {
        let put: StateT<i32, Option<(i32, i32)>> = StateT::<i32, Option<(i32, i32)>>::put(100);
        assert_eq!(put.run(42), Some((100, 42)));
    }

    #[test]
    fn test_state_t_modify() {
        let modify: StateT<i32, Option<(i32, ())>> =
            StateT::<i32, Option<(i32, ())>>::modify(|s| s * 2);
        assert_eq!(modify.run(21), Some((42, ())));
    }

    #[test]
    fn test_state_t_map() {
        let inc: StateT<i32, Option<(i32, i32)>> = StateT::new(|s: i32| Some((s + 1, s)));
        let doubled = inc.map(|v| v * 2);
        assert_eq!(doubled.run(5), Some((6, 10)));
    }

    #[test]
    fn test_state_t_flat_map() {
        let inc: StateT<i32, Option<(i32, i32)>> = StateT::new(|s: i32| Some((s + 1, s)));
        let chained = inc.flat_map(|val| StateT::new(move |s: i32| Some((s * 2, val + s))));
        // Start: 5, inc: (6, 5), then (6*2, 5+6) = (12, 11)
        assert_eq!(chained.run(5), Some((12, 11)));
    }

    #[test]
    fn test_state_t_exec_state() {
        let inc: StateT<i32, Option<(i32, i32)>> = StateT::new(|s: i32| Some((s + 1, s)));
        assert_eq!(inc.exec_state(5), Some(6));
    }

    #[test]
    fn test_state_t_eval_state() {
        let inc: StateT<i32, Option<(i32, i32)>> = StateT::new(|s: i32| Some((s + 1, s)));
        assert_eq!(inc.eval_state(5), Some(5));
    }

    #[test]
    fn test_state_t_result() {
        let ok_val: StateT<i32, Result<(i32, &str), &str>> = StateT::ok("hello");
        assert_eq!(ok_val.run(42), Ok((42, "hello")));

        let err_val: StateT<i32, Result<(i32, &str), &str>> = StateT::err("error");
        assert_eq!(err_val.run(42), Err("error"));
    }

    #[test]
    fn test_state_t_result_flat_map() {
        let inc: StateT<i32, Result<(i32, i32), &str>> = StateT::new(|s: i32| Ok((s + 1, s)));
        let chained = inc.flat_map_ok(|val| {
            StateT::new(move |s: i32| {
                if s > 0 {
                    Ok((s * 2, val + s))
                } else {
                    Err("negative state")
                }
            })
        });
        assert_eq!(chained.run(5), Ok((12, 11)));
    }

    // Monad law tests
    #[test]
    fn test_state_t_left_identity() {
        // pure(a).flat_map(f) == f(a)
        let a = 5;
        let f = |x: i32| StateT::<i32, Option<(i32, i32)>>::pure(x * 2);

        let left = StateT::<i32, Option<(i32, i32)>>::pure(a).flat_map(f);
        let right = f(a);
        assert_eq!(left.run(10), right.run(10));
    }

    #[test]
    fn test_state_t_right_identity() {
        // m.flat_map(pure) == m
        let m: StateT<i32, Option<(i32, i32)>> = StateT::pure(42);
        let result = m.flat_map(StateT::pure);
        assert_eq!(result.run(10), Some((10, 42)));
    }

    #[test]
    fn test_state_t_associativity() {
        // m.flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
        let f = |x: i32| StateT::<i32, Option<(i32, i32)>>::pure(x + 1);
        let g = |x: i32| StateT::<i32, Option<(i32, i32)>>::pure(x * 2);

        let left = StateT::<i32, Option<(i32, i32)>>::pure(5)
            .flat_map(f)
            .flat_map(g);
        let right = StateT::<i32, Option<(i32, i32)>>::pure(5).flat_map(move |x| f(x).flat_map(g));
        assert_eq!(left.run(10), right.run(10));
    }

    #[test]
    fn test_state_t_vec() {
        let state_t: StateT<i32, Vec<(i32, i32)>> = StateT::singleton(42);
        assert_eq!(state_t.run(10), alloc::vec![(10, 42)]);

        let mapped = StateT::singleton(21).map_vec(|x| x * 2);
        assert_eq!(mapped.run(10), alloc::vec![(10, 42)]);
    }
}
